//! This crate provides a convenient interface for showing toast notifications with
//! the [egui](https://github.com/emilk/egui) library.
//!
//! For a complete example, see <https://github.com/urholaukkarinen/egui-toast/tree/main/examples>.
//!
//! # Usage
//!
//! To get started, create a `Toasts` instance in your rendering code and specify the anchor position and
//! direction for the notifications. Toast notifications will show up starting from the specified
//! anchor position and stack up in the specified direction.
//!
//! To add a toast, you can use one of the convenience methods for different [ToastKinds](ToastKind),
//! e.g. [`Toasts::info()`] for info notifications. You can also use [`Toasts::add()`] if you would like to specify the toast kind
//! manually.
//!
//! ```
//! # use std::time::Duration;
//! # use egui_toast::{Toasts, ToastKind, ToastOptions};
//! # egui_toast::__run_test_ui(|ui, ctx| {
//! let mut toasts = Toasts::new()
//!     .anchor((300.0, 300.0))
//!     .direction(egui::Direction::BottomUp)
//!     .align_to_end(true);
//!
//! toasts.info(ui, "Hello, World!", Duration::from_secs(5));
//! // or
//! toasts.info(ui, "Hello, World!", ToastOptions {
//!     show_icon: true,
//!     ..ToastOptions::with_duration(Duration::from_secs(5))
//! });
//! // or
//! toasts.add(ui, "Hello, World!", ToastKind::Info, Duration::from_secs(5));
//!
//! // Show all toasts
//! toasts.show(ctx);
//! # })
//! ```
//!
//! Look of the notifications can be fully customized by specifying a custom rendering function for a specific toast kind
//! with [`Toasts::custom_contents`]. [`ToastKind::Custom`] can be used if the default kinds are not sufficient.
//!
//! ```
//! # use std::time::Duration;
//! # use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
//! # egui_toast::__run_test_ui(|ui, ctx| {
//! const MY_CUSTOM_TOAST: u32 = 0;
//!
//! fn custom_toast_contents(ui: &mut egui::Ui, toast: &mut Toast) {
//!     egui::Frame::window(ui.style()).show(ui, |ui| {
//!         ui.label(toast.text.clone());
//!     });
//! }
//!
//! let mut toasts = Toasts::new()
//!     .custom_contents(MY_CUSTOM_TOAST, &custom_toast_contents)
//!     .custom_contents(ToastKind::Info, &|ui, toast| {
//!         ui.label(toast.text.clone());
//!     });
//!
//! // Add a custom toast that never expires
//! toasts.add(ui, "Hello, World", MY_CUSTOM_TOAST, ToastOptions::with_duration(None));
//!
//! # })
//! ```
//!
#![deny(clippy::all)]

use std::collections::HashMap;
use std::ops::Add;
use std::time::{Duration, Instant};

use egui::{
    Align, Area, Color32, Context, Direction, Id, Layout, Order, Pos2, Rect, RichText, Ui, Vec2,
    WidgetText,
};

pub const INFO_COLOR: Color32 = Color32::from_rgb(0, 155, 255);
pub const WARNING_COLOR: Color32 = Color32::from_rgb(255, 212, 0);
pub const ERROR_COLOR: Color32 = Color32::from_rgb(255, 32, 0);
pub const SUCCESS_COLOR: Color32 = Color32::from_rgb(0, 255, 32);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ToastKind {
    Info,
    Warning,
    Error,
    Success,
    Custom(u32),
}

impl From<u32> for ToastKind {
    fn from(value: u32) -> ToastKind {
        ToastKind::Custom(value)
    }
}

#[derive(Clone)]
pub struct Toast {
    pub kind: ToastKind,
    pub text: WidgetText,
    pub options: ToastOptions,
}

#[derive(Copy, Clone)]
pub struct ToastOptions {
    /// This can be used to show or hide the toast type icon.
    pub show_icon: bool,
    /// If defined, the toast is removed when it expires.
    pub expires_at: Option<Instant>,
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            show_icon: true,
            expires_at: None,
        }
    }
}

impl From<Duration> for ToastOptions {
    fn from(duration: Duration) -> Self {
        ToastOptions::with_duration(duration)
    }
}

impl ToastOptions {
    pub fn with_duration(duration: impl Into<Option<Duration>>) -> Self {
        Self {
            expires_at: duration.into().map(|duration| Instant::now().add(duration)),
            ..Default::default()
        }
    }
}

impl Toast {
    pub fn close(&mut self) {
        self.options.expires_at = Some(Instant::now());
    }
}

pub struct Toasts<'a> {
    id: Id,
    anchor: Pos2,
    direction: Direction,
    align_to_end: bool,
    custom_toast_contents: HashMap<ToastKind, &'a dyn Fn(&mut Ui, &mut Toast)>,
}

impl<'a> Default for Toasts<'a> {
    fn default() -> Self {
        Self {
            id: Id::new("__toasts"),
            anchor: Pos2::new(0.0, 0.0),
            direction: Direction::TopDown,
            align_to_end: false,
            custom_toast_contents: HashMap::new(),
        }
    }
}

impl<'a> Toasts<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starting position for the toasts
    pub fn anchor(mut self, anchor: impl Into<Pos2>) -> Self {
        self.anchor = anchor.into();
        self
    }

    /// Direction where the toasts stack up
    pub fn direction(mut self, direction: impl Into<Direction>) -> Self {
        self.direction = direction.into();
        self
    }

    /// Whether to align toasts to right/bottom depending on the direction
    pub fn align_to_end(mut self, align_to_end: bool) -> Self {
        self.align_to_end = align_to_end;
        self
    }

    /// Can be used to specify a custom rendering function for toasts for given kind
    pub fn custom_contents(
        mut self,
        kind: impl Into<ToastKind>,
        add_contents: &'a dyn Fn(&mut Ui, &mut Toast),
    ) -> Self {
        self.custom_toast_contents.insert(kind.into(), add_contents);
        self
    }

    /// Adds a new info toast
    pub fn info(
        &mut self,
        ui: &mut Ui,
        text: impl Into<WidgetText>,
        options: impl Into<ToastOptions>,
    ) -> &mut Self {
        self.add(ui, text, ToastKind::Info, options)
    }

    /// Adds a new warning toast
    pub fn warning(
        &mut self,
        ui: &mut Ui,
        text: impl Into<WidgetText>,
        options: impl Into<ToastOptions>,
    ) -> &mut Self {
        self.add(ui, text, ToastKind::Warning, options)
    }

    /// Adds a new error toast
    pub fn error(
        &mut self,
        ui: &mut Ui,
        text: impl Into<WidgetText>,
        options: impl Into<ToastOptions>,
    ) -> &mut Self {
        self.add(ui, text, ToastKind::Error, options)
    }

    /// Adds a new success toast
    pub fn success(
        &mut self,
        ui: &mut Ui,
        text: impl Into<WidgetText>,
        options: impl Into<ToastOptions>,
    ) -> &mut Self {
        self.add(ui, text, ToastKind::Success, options)
    }

    /// Adds a new toast
    pub fn add(
        &mut self,
        ui: &mut Ui,
        text: impl Into<WidgetText>,
        kind: impl Into<ToastKind>,
        options: impl Into<ToastOptions>,
    ) -> &mut Self {
        ui.ctx()
            .data()
            .get_temp_mut_or_default::<Vec<Toast>>(self.id)
            .push(Toast {
                kind: kind.into(),
                text: text.into(),
                options: options.into(),
            });
        self
    }

    /// Shows and updates all toasts
    pub fn show(mut self, ctx: &Context) {
        Area::new("__toasts")
            .default_pos((0.0, 0.0))
            .order(Order::Background)
            .interactable(true)
            .movable(false)
            .show(ctx, |ui| {
                let now = Instant::now();

                let rect = match (self.direction, self.align_to_end) {
                    (Direction::LeftToRight | Direction::TopDown, false) => Rect::from_min_size(
                        self.anchor,
                        ui.available_size() - self.anchor.to_vec2(),
                    ),
                    (Direction::RightToLeft | Direction::BottomUp, true) => Rect::from_min_size(
                        Pos2::new(0.0, 0.0),
                        Vec2::new(self.anchor.x, self.anchor.y),
                    ),
                    (Direction::BottomUp, false) | (Direction::LeftToRight, true) => {
                        Rect::from_min_size(
                            Pos2::new(self.anchor.x, 0.0),
                            Vec2::new(ui.available_width() - self.anchor.x, self.anchor.y),
                        )
                    }
                    (Direction::RightToLeft, false) | (Direction::TopDown, true) => {
                        Rect::from_min_size(
                            Pos2::new(0.0, self.anchor.y),
                            Vec2::new(self.anchor.x, ui.available_height() - self.anchor.y),
                        )
                    }
                };

                let cross_align = if self.align_to_end {
                    Align::Max
                } else {
                    Align::Min
                };

                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.with_layout(
                        Layout::from_main_dir_and_cross_align(self.direction, cross_align),
                        |ui| {
                            ui.spacing_mut().item_spacing = Vec2::splat(5.0);

                            let mut toasts: Vec<Toast> =
                                ui.ctx().data().get_temp(self.id).unwrap_or_default();

                            for toast in toasts.iter_mut() {
                                if let Some(add_contents) =
                                    self.custom_toast_contents.get_mut(&toast.kind)
                                {
                                    add_contents(ui, toast);
                                } else {
                                    default_toast_contents(ui, toast);
                                }
                            }

                            toasts.retain(|toast| {
                                toast
                                    .options
                                    .expires_at
                                    .filter(|&expires_at| expires_at <= now)
                                    .is_none()
                            });

                            ui.ctx().data().insert_temp(self.id, toasts);
                        },
                    );
                });
            });
    }
}

fn default_toast_contents(ui: &mut Ui, toast: &mut Toast) {
    egui::Frame::window(ui.style())
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (icon, color) = match toast.kind {
                    ToastKind::Warning => ("⚠", WARNING_COLOR),
                    ToastKind::Error => ("❗", ERROR_COLOR),
                    ToastKind::Success => ("✔", SUCCESS_COLOR),
                    _ => ("ℹ", INFO_COLOR),
                };

                let a = |ui: &mut Ui, toast: &mut Toast| {
                    if toast.options.show_icon {
                        ui.label(RichText::new(icon).color(color));
                    }
                };
                let b = |ui: &mut Ui, toast: &mut Toast| ui.label(toast.text.clone());
                let c = |ui: &mut Ui, toast: &mut Toast| {
                    if ui.button("🗙").clicked() {
                        toast.close();
                    }
                };

                // Draw the contents in the reverse order on right-to-left layouts
                // to keep the same look.
                if ui.layout().prefer_right_to_left() {
                    c(ui, toast);
                    b(ui, toast);
                    a(ui, toast);
                } else {
                    a(ui, toast);
                    b(ui, toast);
                    c(ui, toast);
                }
            });
        });
}

pub fn __run_test_ui(mut add_contents: impl FnMut(&mut Ui, &Context)) {
    let ctx = Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            add_contents(ui, ctx);
        });
    });
}

pub fn __run_test_ui_with_toasts(mut add_contents: impl FnMut(&mut Ui, &mut Toasts)) {
    let ctx = Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut toasts = Toasts::new();
            add_contents(ui, &mut toasts);
        });
    });
}
