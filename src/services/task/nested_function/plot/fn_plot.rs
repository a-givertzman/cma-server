use chrono::Utc;
use eframe::EventLoopBuilderHook;
use egui::ViewportBuilder;
use indexmap::IndexMap;
use log::{error, trace};
use sal_sync::services::{entity::{cot::Cot, point::{point::Point, point_hlr::PointHlr, point_tx_id::PointTxId}, status::status::Status}, types::bool::Bool};
use winit::platform::x11::EventLoopBuilderExtX11;
use std::{mem::MaybeUninit, sync::{atomic::{AtomicUsize, Ordering}, mpsc::Sender, Once}, thread};
use crate::{
    core_::types::fn_in_out_ref::FnInOutRef,
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    },
};

use super::ui_plot::UiPlot;
///
/// Function | Displaying values of the inputs on the diagram
/// - 'x' - input of the x-values, default current time
/// - 'any input' - y-values, name of input displayed in the legend
/// - 'legend' - legend wil be displayed if true
/// - 'enable' - enables functionality
/// - Returns value from 'enable' input
#[derive(Debug)]
pub struct FnPlot {
    id: String,
    tx_id: usize,
    kind: FnKind,
    enable: Option<FnInOutRef>,
    x: Option<FnInOutRef>,
    inputs: IndexMap<String, FnInOutRef>,
    plot_send: Sender<(String, egui::accesskit::Point)>
}
//
// 
impl FnPlot {
    ///
    /// Creates new instance of the FnPlot
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, enable: Option<FnInOutRef>, x: Option<FnInOutRef>, inputs: IndexMap<String, FnInOutRef>) -> Self {
        let id = format!("{}/FnPlot{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let tx_id = PointTxId::from_str(&id);
        Self { 
            plot_send: ui_plot(id.clone()).clone(),
            id,
            tx_id,
            kind: FnKind::Fn,
            enable,
            x,
            inputs,
        }
    }    
}
//
// 
impl FnIn for FnPlot {}
//
// 
impl FnOut for FnPlot { 
    //
    fn id(&self) -> String {
        self.id.clone()
    }
    //
    fn kind(&self) -> &FnKind {
        &self.kind
    }
    //
    fn inputs(&self) -> Vec<String> {
        let mut inputs = vec![];
        if let Some(enable) = &self.enable {
            inputs.append(&mut enable.borrow().inputs());
        }
        if let Some(x) = &self.x {
            inputs.append(&mut x.borrow().inputs());
        }
        for (_, input) in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    //
    fn out(&mut self) -> FnResult<Point, String> {
        let mut inputs = self.inputs.iter();
        let enable = match &self.enable {
            Some(enable) => {
                let enable = enable.borrow_mut().out();
                match enable {
                    FnResult::Ok(enable) => enable.to_bool().as_bool().value.0,
                    FnResult::None => return FnResult::None,
                    FnResult::Err(err) => return FnResult::Err(err),
                }
            }
            None => true,
        };
        let mut value: Point;
        while let Some((name, input)) = inputs.next() {
            let input = input.borrow_mut().out();
            match input {
                FnResult::Ok(input) => {
                    value = input.clone();
                    trace!("{}.out | value: {:#?}", self.id, value);
                    let d = value.timestamp();
                    let secs = d.timestamp() as f64 ;
                    let nanos = (d.timestamp_subsec_nanos() as f64) / 1_000_000_000.0;
                    let x = secs + nanos;
                    let send = (name.to_owned(), egui::accesskit::Point::new(x, value.to_double().as_double().value));
                    if let Err(err) = self.plot_send.send(send) {
                        error!("{}.out | Send error: {:#?}", self.id, err);
                    }
                }
                FnResult::None => {}
                FnResult::Err(err) => {
                    error!("{}.out | Error on input '{}': {:#?}", self.id, name, err);
                }
            }
        }        
        FnResult::Ok(Point::Bool(
            PointHlr::new(
                self.tx_id,
                &self.id,
                Bool(enable),
                Status::Ok,
                Cot::Inf,
                Utc::now(),
            )
        ))
    }
    //
    //
    fn reset(&mut self) {
        if let Some(enable) = &self.enable {
            enable.borrow_mut().reset();
        }
        if let Some(x) = &self.x {
            x.borrow_mut().reset();
        }
        for (_, input) in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
//
// 
impl FnInOut for FnPlot {}
///
/// Global static counter of FnPlot instances
static COUNT: AtomicUsize = AtomicUsize::new(1);


fn ui_plot(parent: String) -> &'static Sender<(String, egui::accesskit::Point)> {
    // Create an uninitialized static
    static mut SINGLETON: MaybeUninit<Sender<(String, egui::accesskit::Point)>> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();
    unsafe {
        ONCE.call_once(|| {
            let (send, recv) = std::sync::mpsc::channel();
            let singleton = send.clone();
            thread::spawn(|| {
                let event_loop_builder: Option<EventLoopBuilderHook> = Some(Box::new(|event_loop_builder| {
                    event_loop_builder.with_any_thread(true);
                }));
                eframe::run_native(
                    "TaskPlot", 
                    eframe::NativeOptions {
                        // fullscreen: true,
                        // maximized: true,
                        event_loop_builder,
                        viewport: ViewportBuilder::default()
                            .with_min_inner_size([ 1920.0, 840.0]),
                        ..Default::default()
                    }, 
                    Box::new(|cc| Ok(Box::new(
                        UiPlot::new(
                            parent,
                            cc,
                            recv,
                        ),
                    ))),
                ).unwrap();    
            });
            // debug!("{}.ui_plot | Ui ready", self.id, err);
            SINGLETON.write(singleton);
        });
        // Now we give out a shared reference to the data, which is safe to use
        // concurrently.
        SINGLETON.assume_init_ref()
    }
}