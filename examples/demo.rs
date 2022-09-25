use std::time::Duration;

use eframe::egui;
use egui::style::Margin;
use egui::{Color32, Direction, Frame, Pos2, RichText, Widget};

use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

/// Identifier for a custom toast kind
const MY_CUSTOM_TOAST: u32 = 0;

fn main() {
    eframe::run_native(
        "egui-toast demo",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(Demo::default())),
    );
}

struct Demo {
    i: usize,
    anchor: Pos2,
    duration_sec: f32,
    direction: Direction,
    align_to_end: bool,
    kind: ToastKind,
    show_icon: bool,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            i: 0,
            duration_sec: 2.0,
            anchor: Pos2::new(10.0, 10.0),
            direction: Direction::TopDown,
            align_to_end: false,
            kind: ToastKind::Info,
            show_icon: true,
        }
    }
}

impl eframe::App for Demo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut toasts = Toasts::new()
            .anchor(self.anchor)
            .direction(self.direction)
            .align_to_end(self.align_to_end)
            .custom_contents(MY_CUSTOM_TOAST, my_custom_toast_contents);

        self.options_window(ctx, &mut toasts);

        toasts.show(ctx);

        ctx.request_repaint();
    }
}

impl Demo {
    fn options_window(&mut self, ctx: &egui::Context, toasts: &mut Toasts) {
        egui::Window::new("")
            .default_pos((100.0, 100.0))
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    egui::DragValue::new(&mut self.anchor.x).ui(ui);
                    ui.label("Anchor X");
                });

                ui.horizontal(|ui| {
                    egui::DragValue::new(&mut self.anchor.y).ui(ui);
                    ui.label("Anchor Y");
                });

                ui.checkbox(&mut self.align_to_end, "Align to right/bottom");

                egui::ComboBox::from_label("Direction")
                    .selected_text(format!("{:?}", self.direction))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.direction, Direction::TopDown, "TopDown");
                        ui.selectable_value(&mut self.direction, Direction::BottomUp, "BottomUp");
                        ui.selectable_value(
                            &mut self.direction,
                            Direction::RightToLeft,
                            "RightToLeft",
                        );
                        ui.selectable_value(
                            &mut self.direction,
                            Direction::LeftToRight,
                            "LeftToRight",
                        );
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    egui::DragValue::new(&mut self.duration_sec)
                        .fixed_decimals(1)
                        .speed(0.1)
                        .clamp_range(0..=100)
                        .suffix("s")
                        .ui(ui);
                    ui.label("Duration");
                });

                egui::ComboBox::from_label("Kind")
                    .selected_text(format!("{:?}", self.kind))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.kind, ToastKind::Info, "Info");
                        ui.selectable_value(&mut self.kind, ToastKind::Warning, "Warning");
                        ui.selectable_value(&mut self.kind, ToastKind::Error, "Error");
                        ui.selectable_value(&mut self.kind, ToastKind::Success, "Success");
                    });

                ui.checkbox(&mut self.show_icon, "Show icon");

                ui.separator();

                let duration = if self.duration_sec < 0.01 {
                    None
                } else {
                    Some(Duration::from_secs_f32(self.duration_sec))
                };

                let options = ToastOptions {
                    show_icon: self.show_icon,
                    ..ToastOptions::with_duration(duration)
                };

                if ui.button("Give me a toast").clicked() {
                    // You can also use toasts.add(). These are just for convenience.

                    match self.kind {
                        ToastKind::Warning => {
                            toasts.warning(format!("Hello, I am a toast {}", self.i), options);
                        }
                        ToastKind::Error => {
                            toasts.error(format!("Hello, I am a toast {}", self.i), options);
                        }
                        ToastKind::Success => {
                            toasts.success(format!("Hello, I am a toast {}", self.i), options);
                        }
                        _ => {
                            toasts.info(format!("Hello, I am a toast {}", self.i), options);
                        }
                    }

                    self.i += 1;
                }

                if ui.button("Give me a custom toast").clicked() {
                    toasts.add(Toast {
                        text: format!("Hello, I am a custom toast {}", self.i).into(),
                        kind: ToastKind::Custom(MY_CUSTOM_TOAST),
                        options,
                    });

                    self.i += 1;
                }
            });
    }
}

fn my_custom_toast_contents(ui: &mut egui::Ui, toast: &mut Toast) -> egui::Response {
    Frame::default()
        .fill(Color32::from_rgb(33, 150, 243))
        .inner_margin(Margin::same(12.0))
        .rounding(4.0)
        .show(ui, |ui| {
            ui.label(toast.text.clone().color(Color32::WHITE).monospace());

            if egui::Button::new(RichText::new("Close").color(Color32::WHITE))
                .fill(Color32::from_rgb(33, 150, 243))
                .stroke((1.0, Color32::WHITE))
                .ui(ui)
                .clicked()
            {
                toast.close();
            }
        })
        .response
}
