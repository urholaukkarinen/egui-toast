//! This crate provides a convenient interface for showing toast notifications with
//! the [egui](https://github.com/emilk/egui) library.
//!
//! For a complete example, see <https://github.com/urholaukkarinen/egui-toast/tree/main/demo>.
//!
//! # Usage
//!
//! To get started, create a `Toasts` instance in your rendering code and specify the anchor position and
//! direction for the notifications. Toast notifications will show up starting from the specified
//! anchor position and stack up in the specified direction.
//! ```
//! # use std::time::Duration;
//! use egui::Align2;
//! # use egui_toast::{Toasts, ToastKind, ToastOptions, Toast};
//! # egui_toast::__run_test_ui(|ui, ctx| {
//! let mut toasts = Toasts::new()
//!     .anchor(Align2::LEFT_TOP, (10.0, 10.0))
//!     .direction(egui::Direction::TopDown);
//!
//! toasts.add(Toast {
//!     text: "Hello, World".into(),
//!     kind: ToastKind::Info,
//!     options: ToastOptions::with_duration_in_seconds(3.0)
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
//!     options: ToastOptions::default(),
//! });
//!
//! # })
//! ```
//!
#![deny(clippy::all)]

use std::collections::HashMap;
use std::time::Duration;

use egui::{
    Align2, Area, Color32, Context, Direction, Id, Order, Pos2, Response, RichText, Ui, WidgetText,
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
    /// The toast is removed when this reaches zero.
    pub ttl_sec: f64,
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            show_icon: true,
            ttl_sec: f64::INFINITY,
        }
    }
}

impl ToastOptions {
    pub fn with_duration(duration: impl Into<Option<Duration>>) -> Self {
        Self {
            ttl_sec: duration
                .into()
                .map_or(f64::INFINITY, |duration| duration.as_secs_f64()),
            ..Default::default()
        }
    }

    pub fn with_duration_in_millis(millis: u64) -> Self {
        Self::with_duration(Duration::from_millis(millis))
    }

    pub fn with_duration_in_seconds(secs: f64) -> Self {
        Self::with_duration(Duration::from_secs_f64(secs))
    }
}

impl Toast {
    pub fn close(&mut self) {
        self.options.ttl_sec = 0.0;
    }
}

pub type ToastContents = dyn Fn(&mut Ui, &mut Toast) -> Response;

pub struct Toasts {
    id: Id,
    align: Align2,
    offset: Pos2,
    direction: Direction,
    custom_toast_contents: HashMap<ToastKind, Box<ToastContents>>,
    /// Toasts added since the last draw call. These are moved to the
    /// egui context's memory, so you are free to recreate the [`Toasts`] instance every frame.
    added_toasts: Vec<Toast>,
}

impl Default for Toasts {
    fn default() -> Self {
        Self {
            id: Id::new("__toasts"),
            align: Align2::LEFT_TOP,
            offset: Pos2::new(10.0, 10.0),
            direction: Direction::TopDown,
            custom_toast_contents: HashMap::new(),
            added_toasts: Vec::new(),
        }
    }
}

impl Toasts {
    pub fn new() -> Self {
        Self::default()
    }

    /// Position where the toasts show up.
    ///
    /// The toasts will start from this position and stack up
    /// in the direction specified with [`Self::direction`].
    pub fn position(mut self, position: impl Into<Pos2>) -> Self {
        self.offset = position.into();
        self
    }

    /// Anchor for the toasts.
    ///
    /// For instance, if you set this to (10.0, 10.0) and [`Align2::LEFT_TOP`],
    /// then (10.0, 10.0) will be the top-left corner of the first toast.
    pub fn anchor(mut self, anchor: Align2, offset: impl Into<Pos2>) -> Self {
        self.align = anchor;
        self.offset = offset.into();
        self
    }

    /// Direction where the toasts stack up
    pub fn direction(mut self, direction: impl Into<Direction>) -> Self {
        self.direction = direction.into();
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

    /// Add a new toast
    pub fn add(&mut self, toast: Toast) -> &mut Self {
        self.added_toasts.push(toast);
        self
    }

    /// Show and update all toasts
    pub fn show(&mut self, ctx: &Context) {
        let Self {
            id,
            align,
            mut offset,
            direction,
            ..
        } = *self;

        let dt = ctx.input(|i| i.unstable_dt) as f64;

        let mut toasts: Vec<Toast> = ctx.data_mut(|d| d.get_temp(id).unwrap_or_default());
        toasts.extend(std::mem::take(&mut self.added_toasts));
        toasts.retain(|toast| toast.options.ttl_sec > 0.0);

        for (i, toast) in toasts.iter_mut().enumerate() {
            let response = Area::new(id.with("toast").with(i))
                .anchor(align, offset.to_vec2())
                .order(Order::Foreground)
                .interactable(true)
                .show(ctx, |ui| {
                    if let Some(add_contents) = self.custom_toast_contents.get_mut(&toast.kind) {
                        add_contents(ui, toast)
                    } else {
                        default_toast_contents(ui, toast)
                    };
                })
                .response;

            if !response.hovered() {
                toast.options.ttl_sec -= dt;
                if toast.options.ttl_sec.is_finite() {
                    ctx.request_repaint_after(Duration::from_secs_f64(
                        toast.options.ttl_sec.max(0.0),
                    ));
                }
            }

            match direction {
                Direction::LeftToRight => {
                    offset.x += response.rect.width() + 10.0;
                }
                Direction::RightToLeft => {
                    offset.x -= response.rect.width() + 10.0;
                }
                Direction::TopDown => {
                    offset.y += response.rect.height() + 10.0;
                }
                Direction::BottomUp => {
                    offset.y -= response.rect.height() + 10.0;
                }
            }
        }

        ctx.data_mut(|d| d.insert_temp(id, toasts));
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
