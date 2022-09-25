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
//! # use egui_toast::{Toasts, ToastKind, ToastOptions, Toast};
//! # egui_toast::__run_test_ui(|ui, ctx| {
//! let mut toasts = Toasts::new()
//!     .anchor((300.0, 300.0))
//!     .direction(egui::Direction::BottomUp)
//!     .align_to_end(true);
//!
//! toasts.info("Hello, World!", Duration::from_secs(5));
//! // or
//! toasts.info("Hello, World!", ToastOptions {
//!     show_icon: true,
//!     ..ToastOptions::with_duration(Duration::from_secs(5))
//! });
//! // or
//! toasts.add(Toast {
//!     text: "Hello, World".into(),
//!     kind: ToastKind::Info,
//!     options: Duration::from_secs(5).into()
//! });

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
//! fn custom_toast_contents(ui: &mut egui::Ui, toast: &mut Toast) -> egui::Response {
//!     egui::Frame::window(ui.style()).show(ui, |ui| {
//!         ui.label(toast.text.clone());
//!     }).response
//! }
//!
//! let mut toasts = Toasts::new()
//!     .custom_contents(MY_CUSTOM_TOAST, &custom_toast_contents)
//!     .custom_contents(ToastKind::Info, |ui, toast| {
//!         ui.label(toast.text.clone())
//!     });
//!
//! // Add a custom toast that never expires
//! toasts.add(Toast {
//!     text: "Hello, World".into(),
//!     kind: ToastKind::Custom(MY_CUSTOM_TOAST),
//!     options: ToastOptions::with_duration(None)
//! });
//!
//! # })
//! ```
//!
#![deny(clippy::all)]

use std::collections::HashMap;
use std::ops::Add;
use std::time::{Duration, Instant};

use egui::{
    Align, Area, Color32, Context, Direction, Id, Layout, Order, Pos2, Rect, Response, RichText,
    Ui, Vec2, WidgetText,
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

pub type ToastContents = dyn Fn(&mut Ui, &mut Toast) -> Response;

pub struct Toasts {
    id: Id,
    anchor: Pos2,
    direction: Direction,
    align_to_end: bool,
    custom_toast_contents: HashMap<ToastKind, Box<ToastContents>>,
    toasts: Vec<Toast>,
}

impl Default for Toasts {
    fn default() -> Self {
        Self::new()
    }
}

impl Toasts {
    pub fn new() -> Self {
        Self {
            id: Id::new("__toasts"),
            anchor: Pos2::new(0.0, 0.0),
            direction: Direction::TopDown,
            align_to_end: false,
            custom_toast_contents: HashMap::new(),
            toasts: Vec::new(),
        }
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
        add_contents: impl Fn(&mut Ui, &mut Toast) -> Response + 'static,
    ) -> Self {
        self.custom_toast_contents
            .insert(kind.into(), Box::new(add_contents));
        self
    }

    /// Adds a new info toast
    pub fn info(
        &mut self,
        text: impl Into<WidgetText>,
        options: impl Into<ToastOptions>,
    ) -> &mut Self {
        self.add(Toast {
            text: text.into(),
            kind: ToastKind::Info,
            options: options.into(),
        })
    }

    /// Adds a new warning toast
    pub fn warning(
        &mut self,
        text: impl Into<WidgetText>,
        options: impl Into<ToastOptions>,
    ) -> &mut Self {
        self.add(Toast {
            text: text.into(),
            kind: ToastKind::Warning,
            options: options.into(),
        })
    }

    /// Adds a new error toast
    pub fn error(
        &mut self,
        text: impl Into<WidgetText>,
        options: impl Into<ToastOptions>,
    ) -> &mut Self {
        self.add(Toast {
            text: text.into(),
            kind: ToastKind::Error,
            options: options.into(),
        })
    }

    /// Adds a new success toast
    pub fn success(
        &mut self,
        text: impl Into<WidgetText>,
        options: impl Into<ToastOptions>,
    ) -> &mut Self {
        self.add(Toast {
            text: text.into(),
            kind: ToastKind::Success,
            options: options.into(),
        })
    }

    /// Adds a new toast
    pub fn add(&mut self, toast: Toast) -> &mut Self {
        self.toasts.push(toast);
        self
    }

    /// Shows and updates all toasts
    pub fn show(&mut self, ctx: &Context) {
        let Self {
            id,
            anchor,
            align_to_end,
            direction,
            ..
        } = *self;

        let mut toasts: Vec<Toast> = ctx.data().get_temp(id).unwrap_or_default();
        toasts.extend(std::mem::take(&mut self.toasts));

        let screen_area = ctx.available_rect();

        let area_pos: Pos2 = ctx.data().get_temp(id.with("pos")).unwrap_or_default();

        Area::new(id.with("area"))
            .fixed_pos(area_pos)
            .order(Order::Foreground)
            .interactable(true)
            .movable(false)
            .show(ctx, |ui| {
                let now = Instant::now();

                let rect = match (direction, align_to_end) {
                    (Direction::LeftToRight | Direction::TopDown, false) => {
                        Rect::from_min_size(anchor, screen_area.size() - anchor.to_vec2())
                    }
                    (Direction::RightToLeft | Direction::BottomUp, true) => {
                        Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(anchor.x, anchor.y))
                    }
                    (Direction::BottomUp, false) | (Direction::LeftToRight, true) => {
                        Rect::from_min_size(
                            Pos2::new(anchor.x, 0.0),
                            Vec2::new(screen_area.width() - anchor.x, anchor.y),
                        )
                    }
                    (Direction::RightToLeft, false) | (Direction::TopDown, true) => {
                        Rect::from_min_size(
                            Pos2::new(0.0, anchor.y),
                            Vec2::new(anchor.x, screen_area.height() - anchor.y),
                        )
                    }
                };

                let cross_align = if align_to_end { Align::Max } else { Align::Min };

                let mut next_area_pos = Pos2::new(f32::MAX, f32::MAX);

                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.with_layout(
                        Layout::from_main_dir_and_cross_align(direction, cross_align),
                        |ui| {
                            ui.spacing_mut().item_spacing = Vec2::splat(5.0);

                            for toast in toasts.iter_mut() {
                                let toast_response = if let Some(add_contents) =
                                    self.custom_toast_contents.get_mut(&toast.kind)
                                {
                                    add_contents(ui, toast)
                                } else {
                                    default_toast_contents(ui, toast)
                                };

                                next_area_pos = next_area_pos.min(toast_response.rect.min);
                            }

                            if toasts.is_empty() {
                                next_area_pos = anchor;
                            }

                            ctx.data().insert_temp(id.with("pos"), next_area_pos);

                            toasts.retain(|toast| {
                                toast
                                    .options
                                    .expires_at
                                    .filter(|&expires_at| expires_at <= now)
                                    .is_none()
                            });

                            ctx.data().insert_temp(id, toasts);
                        },
                    );
                });
            });
    }
}

fn default_toast_contents(ui: &mut Ui, toast: &mut Toast) -> Response {
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
        })
        .response
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
