use std::{cell::RefCell, rc::Rc, sync::mpsc::Receiver};
use eframe::CreationContext;
use egui_plot::{Line, Plot, Points};
use hsl::HSL;
use indexmap::IndexMap;
use log::{error, info, warn};
use egui::{
    accesskit::Point, vec2, Align2, Color32, FontFamily, FontId, TextStyle 
};
use crate::core_::state::change_notify::ChangeNotify;
///
/// Plot the point values
pub struct UiPlot {
    id: String,
    // renderDelay: Duration,
    real_input_min_y: f64,
    real_input_max_y: f64,
    real_input_autoscale_y: bool,
    show_events: bool,
    input: Receiver<(String, Point)>,
    plot_style: IndexMap<String, Rc<RefCell<PlotStyle>>>,
    points: IndexMap<String, Vec<[f64; 2]>>,
    events: Vec<String>,
    #[allow(unused)]
    status: Rc<RefCell<ChangeNotify<UiStatus, String>>>,
}
//
//
impl UiPlot {
    ///
    /// 
    pub fn new(
        parent: impl Into<String>,
        cc: &CreationContext,
        recv: Receiver<(String, Point)>,
        // renderDelay: Duration,
    ) -> Self {
        Self::setup_custom_fonts(&cc.egui_ctx);
        Self::configure_text_styles(&cc.egui_ctx);
        let self_id = format!("{}/UiPlot", parent.into());
        let status = Rc::new(RefCell::new(ChangeNotify::new(
            &self_id,
            UiStatus::Ok,
            vec![
                (UiStatus::Ok,  Box::new(|message| info!("{}", message))),
                (UiStatus::Err, Box::new(|message| warn!("{}", message))),
            ],
        )));
        Self {
            id: self_id,
            real_input_min_y: -10.0,
            real_input_max_y: 200.0,
            // real_input_len: 1024,
            // realInputAutoscroll: true,
            real_input_autoscale_y: false,
            show_events: false,
            // send,
            input: recv,
            plot_style: IndexMap::new(),
            points: IndexMap::new(),
            events: vec![],
            status,
        }
    }
    ///
    /// 
    fn setup_custom_fonts(ctx: &egui::Context) {
        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            "Icons".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "./../../../../../assets/fonts/icons.ttf"
            )),
        );

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Icons".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("Icons".to_owned());

        // Tell egui to use these fonts:
        ctx.set_fonts(fonts);
    }
    ///
    fn configure_text_styles(ctx: &egui::Context) {
        use FontFamily::{Monospace, Proportional};
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(24.0, Proportional)),
            // (heading2(), FontId::new(22.0, Proportional)),
            // (heading3(), FontId::new(19.0, Proportional)),
            (TextStyle::Body, FontId::new(16.0, Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, Monospace)),
            (TextStyle::Button, FontId::new(16.0, Proportional)),
            (TextStyle::Small, FontId::new(8.0, Proportional)),
        ].into();
        ctx.set_style(style);
    }
    ///
    /// Generates different color
    fn different_color(&self, index: usize) -> Color32 {
        let colors = self.points.len() as f64;
        let h = ((index as f64) * (360.0 / colors)) % 360.0;
        let rgb = HSL { h, s: 1.0, l: 0.5 }.to_rgb();
        Color32::from_rgb(rgb.0, rgb.1, rgb.2)
    }
}

///
///
impl eframe::App for UiPlot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let window_size = match ctx.input(|i| i.viewport().inner_rect) {
            Some(rect) => rect,
            None => ctx.input(|i: &egui::InputState| i.screen_rect),
        };
        let head_hight = 34.0;
        while let Ok((name, value)) = self.input.try_recv() {
            self.points.entry(name.clone())
                .or_insert(vec![[value.x, value.y]])
                .push([value.x, value.y]);
            self.events.push(format!("{}: {:.3} ", name, value.y));
            let color = self.different_color(self.plot_style.len());
            self.plot_style.entry(name)
                .or_insert(Rc::new(RefCell::new(PlotStyle { show: true, square: true, width: 1.0, color, line: PlotLineStyle::Dots, scale: Scale::default() })));
        }
        // match self.input.try_recv() {
        //     Ok((name, value)) => {
        //         self.points.entry(name.clone())
        //             .or_insert(vec![[value.x, value.y]])
        //             .push([value.x, value.y]);
        //         self.events.push(format!("{}: {:.3} ", name, value.y));
        //         let color = self.different_color(self.plot_style.len());
        //         self.plot_style.entry(name)
        //             .or_insert(Rc::new(RefCell::new(PlotStyle { show: true, square: true, width: 1.0, color, line: PlotLineStyle::Dots, scale: Scale::default() })));
        //     }
        //     Err(err) => {
        //         self.status.borrow_mut().add(UiStatus::Err, &format!("{}.update | self.input.recv error: {:?}", self.id, err));
        //     }
        // };
        egui::Window::new("Settings")
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .default_size(vec2(0.4 * window_size.width(), 0.5 * (window_size.height() - head_hight)))
            .show(ctx, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .column(egui_extras::Column::initial(32.0))
                    .column(egui_extras::Column::initial(250.0))
                    .column(egui_extras::Column::initial(72.0))
                    .column(egui_extras::Column::initial(72.0))
                    .column(egui_extras::Column::initial(72.0))
                    .column(egui_extras::Column::initial(72.0))
                    .header(20.0, |mut header| {
                        header.col(|ui| {ui.label("-");});
                        header.col(|ui| {ui.label("Name");});
                        header.col(|ui| {ui.label("Line");});
                        header.col(|ui| {ui.label("Bold");});
                        header.col(|ui| {ui.label("Square");});
                        header.col(|ui| {ui.label("Y-Scale");});
                    })
                    .body(|mut body| {
                        for (i, (name, style)) in self.plot_style.iter().enumerate() {
                            let color = style.borrow().color;
                            body.row(32.0, |mut row| {
                                row.col(|ui| {
                                    ui.checkbox(&mut (*style.borrow_mut()).show, "");
                                });
                                row.col(|ui| {
                                    ui.label(egui::RichText::new(format!("{:?}\t|\t{:?}", i, name)).color(color));
                                });
                                row.col(|ui| {
                                    let mut is_line = style.borrow().line == PlotLineStyle::Line;
                                    ui.add(egui::Checkbox::without_text(&mut is_line));
                                    // ui.checkbox(&mut is_line, "");
                                    (*style.borrow_mut()).line = if is_line {PlotLineStyle::Line} else {PlotLineStyle::Dots};
                                });
                                row.col(|ui| {
                                    let mut is_bold = style.borrow().width == 2.0;
                                    ui.add(egui::Checkbox::without_text(&mut is_bold));
                                    (*style.borrow_mut()).width = if is_bold {2.0} else {1.0};
                                });
                                row.col(|ui| {
                                    ui.add(egui::Checkbox::without_text(&mut (*style.borrow_mut()).square));
                                });
                                row.col(|ui| {
                                    let mut y_scale = style.borrow().scale.y.to_string();
                                    if ui.add(egui::TextEdit::singleline(&mut y_scale)).changed() {
                                        if let Ok(value) = y_scale.parse() {
                                            (*style.borrow_mut()).scale.y = value
                                        };
                                    };                          
                                });
                            });
                        }
                    });
            });
        if self.show_events {
            egui::Window::new("Events")
                .anchor(Align2::RIGHT_BOTTOM, [0.0, 0.0])
                .default_size(vec2(0.4 * window_size.width(), 0.5 * (window_size.height() - head_hight)))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for (i, event) in self.events.iter().enumerate() {
                            ui.label(format!("{:?}\t|\t{:?}", i, event));
                            ui.separator();
                        }
                    });
                });
        }
        egui::Window::new(self.id.clone())
            // .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .default_size(vec2(0.8 * window_size.width(), 0.8 * window_size.height() - head_hight))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [200.0, 16.0], 
                        egui::Label::new(format!("window width: {:?}", window_size.width())),
                    );
                    ui.label(format!("max length: {}", 0));
                    ui.separator();
                    ui.checkbox(&mut self.show_events, "Events");
                    ui.separator();
                    ui.checkbox(&mut self.real_input_autoscale_y, "Autoscale Y");
                    ui.separator();
                });
                ui.separator();
                let mut min = format!("{}", self.real_input_min_y);
                let mut max = format!("{}", self.real_input_max_y);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [32.0, 16.0 * 2.0 + 6.0], 
                        egui::Label::new(format!("↕")), //⇔⇕   ↔
                    );
                    ui.separator();
                    ui.vertical(|ui| {
                        if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut max)).changed() {
                            if !self.real_input_autoscale_y {
                                self.real_input_max_y = match max.parse() {Ok(value) => {value}, Err(_) => {self.real_input_max_y}};
                            }
                        };
                        if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut min)).changed() {
                            if !self.real_input_autoscale_y {
                                self.real_input_min_y = match min.parse() {Ok(value) => {value}, Err(_) => {self.real_input_min_y}};
                            }
                        };
                    });        
                });
                let mut plot = Plot::new(self.id.clone());
                if !self.real_input_autoscale_y {
                    plot = plot.include_y(self.real_input_min_y);
                    plot = plot.include_y(self.real_input_max_y);
                }
                plot.show(ui, |plot_ui| {
                    let mut prev = None;
                    for (_i, (label, points)) in self.points.iter().enumerate() {
                        match self.plot_style.get(label) {
                            Some(plot_style) => {
                                if plot_style.borrow().show {
                                    match plot_style.borrow().line {
                                        PlotLineStyle::Dots => {
                                            let scale = plot_style.borrow().scale.clone();
                                            plot_ui.points(
                                                Points::new(
                                                    points.iter().map(|p| [p[0], scale.scale_y(p[1])] ).collect::<Vec<[f64; 2]>>()
                                                )
                                                    .name(label)
                                                    .color(plot_style.borrow().color)
                                                    .radius(plot_style.borrow().width)
                                                    .filled(true),
                                            );
                                        }
                                        PlotLineStyle::Line => {
                                            let square = plot_style.borrow().square;
                                            let scale = plot_style.borrow().scale.clone();
                                            plot_ui.line(
                                                Line::new(
                                                    points.iter().fold(Vec::<[f64; 2]>::new(), |mut acc, p| {
                                                        if square {
                                                            if let Some(prev_) = &prev {
                                                                if p[1] != *prev_ {acc.push([p[0], scale.scale_y(*prev_)])}
                                                            };
                                                            prev = Some(p[1]);
                                                        }
                                                        acc.push([p[0], scale.scale_y(p[1])]);
                                                        acc
                                                    })
                                                    // points.iter().map(|p| [p[0], plot_style.borrow().scale.scale_y(p[1])] ).collect::<Vec<[f64; 2]>>()
                                                )
                                                    .color(plot_style.borrow().color),
                                            );
                                        }
                                    }
                                }
                            }
                            None => error!("{}.update | Unknown plot key '{}'", self.id, label),
                        }
                    }
                });
            });
        // std::thread::sleep(self.renderDelay);
        ctx.request_repaint();
    }
}
///
/// 
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum UiStatus {
    Ok,
    Err,
}
///
/// 
#[derive(Clone, Debug, PartialEq)]
struct PlotStyle {
    show: bool,
    square: bool,
    width: f32,
    line: PlotLineStyle,
    color: Color32,
    scale: Scale,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum PlotLineStyle {
    Dots,
    Line,
}
#[derive(Clone, Debug, PartialEq)]
struct Scale {
    x: f64,
    y: f64,
}
impl Default for Scale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}
impl Scale {
    #[allow(unused)]
    pub fn scale_x(&self, x: f64) -> f64 {
        x * self.x
    }
    pub fn scale_y(&self, y: f64) -> f64 {
        y * self.y
    }
}

// pub trait ExtendedColors {
//     const orange: Color32 = Color32::from_rgb(255, 152, 0);
//     const orangeAccent: Color32 = Color32::from_rgb(255, 152, 0);
//     const lightGreen10: Color32 = Color32::from_rgba_premultiplied(0x90, 0xEE, 0x90, 10);
//     fn with_opacity(&self, opacity: u8) -> Self;
// }

// impl ExtendedColors for Color32 {
//     fn with_opacity(&self, opacity: u8) -> Self {
//         let [r, g, b, _] = self.to_array();
//         Color32::from_rgba_premultiplied(r, g, b, opacity)
//     }
// }
