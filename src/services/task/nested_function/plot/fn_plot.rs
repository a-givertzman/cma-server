use chrono::Utc;
use indexmap::IndexMap;
use sal_sync::services::{entity::{cot::Cot, point::{point::Point, point_hlr::PointHlr, point_tx_id::PointTxId}, status::status::Status}, types::bool::Bool};
use std::{sync::{atomic::{AtomicUsize, Ordering}, mpsc::Sender}, thread};
use crate::{
    core_::types::fn_in_out_ref::FnInOutRef,
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    },
};
use lazy_static::lazy_static;
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
            plot_send: UI_PLOT.clone(),
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
                    log::trace!("{}.out | value: {:#?}", self.id, value);
                    let d = value.timestamp();
                    let secs = d.timestamp() as f64 ;
                    let nanos = (d.timestamp_subsec_nanos() as f64) / 1_000_000_000.0;
                    let x = secs + nanos;
                    let send = (name.to_owned(), egui::accesskit::Point::new(x, value.to_double().as_double().value));
                    if let Err(err) = self.plot_send.send(send) {
                        log::error!("{}.out | Send error: {:#?}", self.id, err);
                    }
                }
                FnResult::None => {}
                FnResult::Err(err) => {
                    log::error!("{}.out | Error on input '{}': {:#?}", self.id, name, err);
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

lazy_static! {
    static ref UI_PLOT: Sender<(String, egui::accesskit::Point)> = ui_plot();
}
#[cfg(feature = "plot")]
fn ui_plot() -> Sender<(String, egui::accesskit::Point)> {
    let (send, recv) = std::sync::mpsc::channel();
    thread::spawn(|| {
        let event_loop_builder: Option<eframe::EventLoopBuilderHook> = Some(Box::new(|event_loop_builder| {
            // event_loop_builder.build().unwrap();
            winit::platform::x11::EventLoopBuilderExtX11::with_any_thread(event_loop_builder, true);
        }));
        eframe::run_native(
            "TaskPlot", 
            eframe::NativeOptions {
                // fullscreen: true,
                // maximized: true,
                event_loop_builder,
                viewport: egui::ViewportBuilder::default()
                    .with_min_inner_size([ 1920.0, 840.0]),
                ..Default::default()
            }, 
            Box::new(|cc| Ok(Box::new(
                super::ui_plot::UiPlot::new(
                    "", //parent,
                    cc,
                    recv,
                ),
            ))),
        ).unwrap();    
    });
    send
}
#[cfg(not(feature = "plot"))]
fn ui_plot() -> Sender<(String, egui::accesskit::Point)> {
    let (send, recv) = std::sync::mpsc::channel();
    log::info!(
        "fn_plot.ui_plot | To activate fn Plot use: \n\t`cargo test --features=plot` or \n\t`cargo run --features=plot`",
    );
    thread::spawn(move || {
        loop {
            if let Err(err) = recv.recv_timeout(sal_sync::services::service::RECV_TIMEOUT) {
                match err {
                    std::sync::mpsc::RecvTimeoutError::Timeout => {},
                    std::sync::mpsc::RecvTimeoutError::Disconnected => {
                        break;
                    },
                }
            }
        }
    });
    send
}
