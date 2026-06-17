use sal_sync::sync::channel::Sender;
use std::{sync::{atomic::{AtomicUsize, Ordering}}, thread};
use crate::{
    domain::FnOutRef,
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
use lazy_static::lazy_static;

///
/// Function | Displaying values of the inputs on the diagram
/// - 'x' - input of the x-values, default current time
/// - 'any input' - y-values, name of input displayed in the legend
/// - 'legend' - legend wil be displayed if true
/// - 'enable' - enables functionality
/// - Returns Ok(None)
/// 
/// **Note !** To activate fn Plot use:
/// - `cargo test --features=plot` or 
/// - `cargo run --features=plot`
/// 
#[derive(Debug)]
pub struct FnPlot {
    kind: FnKind,
    x: Option<FnOutRef>,
    inputs: Vec<(String, FnOutRef)>,
    plot_send: Sender<(String, egui::accesskit::Point)>,
    id: String,
}
//
// 
impl FnPlot {
    ///
    /// Creates new instance of the FnPlot
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, x: Option<FnOutRef>, inputs: impl IntoIterator<Item = (String, FnOutRef)>) -> Self {
        let id = format!("{}/FnPlot{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self { 
            kind: FnKind::Fn,
            x,
            inputs: inputs.into_iter().collect(),
            plot_send: UI_PLOT.clone(),
            id,
        }
    }    
}
//
// 
impl FnOut for FnPlot { 
    //
    fn id(&self) -> String {
        self.id.clone()
    }
    //
    fn kind(&self) -> FnKind {
        self.kind
    }
    //
    fn inputs(&self) -> Vec<String> {
        let mut inputs = vec![];
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
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let flow = FlowContext::new();
        let inputs: Vec<(&String, Result<Option<FnFlow>, String>)> = self.inputs.iter()
            .map(|(k, f)| (k, f.borrow_mut().out()))
            .collect();
        for (name, input) in inputs {
            match flow.ignore(input) {
                Ok(Some(value)) => {
                    log::trace!("{}.out | value: {:?}", self.id, value);
                    let d = value.ts();
                    let secs = d.timestamp() as f64 ;
                    let nanos = (d.timestamp_subsec_nanos() as f64) / 1_000_000_000.0;
                    let x = secs + nanos;
                    let send = (name.to_owned(), egui::accesskit::Point::new(x, value.to_double().as_double().value));
                    if let Err(err) = self.plot_send.send(send) {
                        log::error!("{}.out | Send error: {:#?}", self.id, err);
                    }
                }
                Ok(None) => log::error!("{}.out | None on input '{}'", self.id, name),
                Err(err) => log::error!("{}.out | Error on input '{}': {:?}", self.id, name, err),
            }
        }        
        Ok(None)
    }
    //
    //
    fn reset(&mut self) {
        if let Some(x) = &self.x {
            x.borrow_mut().reset();
        }
        for (_, input) in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnPlot instances
static COUNT: AtomicUsize = AtomicUsize::new(1);

lazy_static! {
    static ref UI_PLOT: Sender<(String, egui::accesskit::Point)> = ui_plot();
}
// #[cfg(not(feature = "plot"))]
#[cfg(feature = "plot")]
fn ui_plot() -> Sender<(String, egui::accesskit::Point)> {
    use sal_sync::sync::channel;
    let (send, recv) = channel::unbounded();
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
// #[cfg(feature = "plot")]
#[cfg(not(feature = "plot"))]
fn ui_plot() -> Sender<(String, egui::accesskit::Point)> {
    use sal_sync::sync::channel;
    let (send, recv) = channel::unbounded();
    println!(
        "fn_plot.ui_plot | To activate fn Plot use: \n\t`cargo test --features=plot` or \n\t`cargo run --features=plot`",
    );
    thread::spawn(move || {
        loop {
            if let Err(err) = recv.recv_timeout(sal_sync::services::RECV_TIMEOUT) {
                use sal_sync::sync::channel::RecvTimeoutError;
                match err {
                    RecvTimeoutError::Timeout => {},
                    _ => break,
                }
            }
        }
    });
    send
}
