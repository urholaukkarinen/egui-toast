use std::time::Duration;

use eframe::egui;
use eframe::epaint::Margin;
use egui::{Align2, Color32, Direction, Frame, Pos2, RichText, Widget};

use egui_toast::{Toast, ToastKind, ToastOptions, ToastStyle, Toasts};

/// Identifier for a custom toast kind
const MY_CUSTOM_TOAST: u32 = 0;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "egui-toast demo",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::<Demo>::default())),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "canvas",
            web_options,
            Box::new(|_cc| Box::<Demo>::default()),
        )
        .await
        .expect("failed to start eframe");
    });
}

struct Demo {
    i: usize,
    offset: Pos2,
    alignment: Align2,
    duration_sec: f64,
    direction: Direction,
    kind: ToastKind,
    show_icon: bool,
    show_progress: bool,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            i: 0,
            duration_sec: 2.0,
            offset: Pos2::new(10.0, 10.0),
            alignment: Align2::LEFT_TOP,
            direction: Direction::TopDown,
            kind: ToastKind::Info,
            show_icon: true,
            show_progress: true,
        }
    }
}

impl eframe::App for Demo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Recreate the toasts in case the demo options have changed.
        let mut toasts = Toasts::new()
            .anchor(self.alignment, self.offset)
            .direction(self.direction)
            .custom_contents(MY_CUSTOM_TOAST, my_custom_toast_contents);

        // Show the options window
        self.options_window(ctx, &mut toasts);

        // Draw and update the toasts
        toasts.show(ctx);
    }
}

impl Demo {
    fn options_window(&mut self, ctx: &egui::Context, toasts: &mut Toasts) {
        let Self {
            i,
            offset: position,
            alignment,
            duration_sec,
            direction,
            kind,
            show_icon,
            show_progress,
        } = self;

        egui::Window::new("Demo options")
            .default_pos((200.0, 200.0))
            .default_width(250.0)
            .show(ctx, |ui| {
                egui::ComboBox::from_label("Anchor")
                    .selected_text(align2_to_str(*alignment))
                    .show_ui(ui, |ui| {
                        alignment_selection(ui, alignment, Align2::LEFT_BOTTOM);
                        alignment_selection(ui, alignment, Align2::LEFT_CENTER);
                        alignment_selection(ui, alignment, Align2::LEFT_TOP);
                        alignment_selection(ui, alignment, Align2::CENTER_BOTTOM);
                        alignment_selection(ui, alignment, Align2::CENTER_CENTER);
                        alignment_selection(ui, alignment, Align2::CENTER_TOP);
                        alignment_selection(ui, alignment, Align2::RIGHT_BOTTOM);
                        alignment_selection(ui, alignment, Align2::RIGHT_CENTER);
                        alignment_selection(ui, alignment, Align2::RIGHT_TOP);
                    });

                ui.horizontal(|ui| {
                    egui::DragValue::new(&mut position.x).ui(ui);
                    ui.label("Anchor X offset");
                });

                ui.horizontal(|ui| {
                    egui::DragValue::new(&mut position.y).ui(ui);
                    ui.label("Anchor Y offset");
                });

                egui::ComboBox::from_label("Direction")
                    .selected_text(format!("{:?}", direction))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(direction, Direction::TopDown, "TopDown");
                        ui.selectable_value(direction, Direction::BottomUp, "BottomUp");
                        ui.selectable_value(direction, Direction::RightToLeft, "RightToLeft");
                        ui.selectable_value(direction, Direction::LeftToRight, "LeftToRight");
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    egui::DragValue::new(duration_sec)
                        .fixed_decimals(1)
                        .speed(0.1)
                        .range(0..=100)
                        .suffix("s")
                        .ui(ui);
                    ui.label("Duration");
                });

                egui::ComboBox::from_label("Kind")
                    .selected_text(format!("{:?}", kind))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(kind, ToastKind::Info, "Info");
                        ui.selectable_value(kind, ToastKind::Warning, "Warning");
                        ui.selectable_value(kind, ToastKind::Error, "Error");
                        ui.selectable_value(kind, ToastKind::Success, "Success");
                    });

                ui.checkbox(show_icon, "Show icon");
                ui.checkbox(show_progress, "Show progress");

                ui.separator();

                let duration = if *duration_sec < 0.01 {
                    None
                } else {
                    Some(Duration::from_secs_f64(*duration_sec))
                };

                let options = ToastOptions::default()
                    .show_icon(*show_icon)
                    .show_progress(*show_progress)
                    .duration(duration);

                let style = ToastStyle::default();

                if ui.button("Give me a toast").clicked() {
                    toasts.add(
                        Toast::default()
                            .kind(*kind)
                            .text(format!("Hello, I am a toast {}", i))
                            .options(options)
                            .style(style.clone()),
                    );

                    *i += 1;
                }

                if ui.button("Give me a custom toast").clicked() {
                    toasts.add(
                        Toast::default()
                            .kind(ToastKind::Custom(MY_CUSTOM_TOAST))
                            .text(format!("Hello, I am a custom toast {}", i))
                            .options(options)
                            .style(style.clone()),
                    );

                    *i += 1;
                }
            });
    }
}

fn my_custom_toast_contents(ui: &mut egui::Ui, toast: &mut Toast) -> egui::Response {
    Frame::default()
        .fill(Color32::from_rgb(33, 150, 243))
        .inner_margin(Margin::same(12))
        .corner_radius(4.0)
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

fn alignment_selection(ui: &mut egui::Ui, current_value: &mut Align2, alignment: Align2) {
    ui.selectable_value(current_value, alignment, align2_to_str(alignment));
}

fn align2_to_str(align: Align2) -> &'static str {
    match align {
        Align2::LEFT_BOTTOM => "LeftBottom",
        Align2::LEFT_CENTER => "LeftCenter",
        Align2::LEFT_TOP => "LeftTop",
        Align2::CENTER_BOTTOM => "CenterBottom",
        Align2::CENTER_CENTER => "CenterCenter",
        Align2::CENTER_TOP => "CenterTop",
        Align2::RIGHT_BOTTOM => "RightBottom",
        Align2::RIGHT_CENTER => "RightCenter",
        Align2::RIGHT_TOP => "RightTop",
    }
}
