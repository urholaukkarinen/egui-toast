use egui::{Color32, WidgetText};
use std::time::Duration;

#[derive(Default, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ToastKind {
    #[default]
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

#[derive(Clone, Default)]
pub struct Toast {
    pub kind: ToastKind,
    pub text: WidgetText,
    pub options: ToastOptions,
    pub style: ToastStyle,
}

impl Toast {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn kind(mut self, kind: ToastKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn text(mut self, text: impl Into<WidgetText>) -> Self {
        self.text = text.into();
        self
    }

    pub fn options(mut self, options: ToastOptions) -> Self {
        self.options = options;
        self
    }

    pub fn style(mut self, style: ToastStyle) -> Self {
        self.style = style;
        self
    }

    /// Close the toast immediately
    pub fn close(&mut self) {
        self.options.ttl_sec = 0.0;
    }
}

#[derive(Clone)]
pub struct ToastStyle {
    pub info_icon: WidgetText,
    pub warning_icon: WidgetText,
    pub error_icon: WidgetText,
    pub success_icon: WidgetText,
    pub close_button_text: WidgetText,
}

impl Default for ToastStyle {
    fn default() -> Self {
        Self {
            info_icon: WidgetText::from("ℹ").color(Color32::from_rgb(0, 155, 255)),
            warning_icon: WidgetText::from("⚠").color(Color32::from_rgb(255, 212, 0)),
            error_icon: WidgetText::from("❗").color(Color32::from_rgb(255, 32, 0)),
            success_icon: WidgetText::from("✔").color(Color32::from_rgb(0, 255, 32)),
            close_button_text: WidgetText::from("🗙"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ToastOptions {
    /// Whether the toast should include an icon.
    pub show_icon: bool,
    /// Whether the toast should visualize the remaining time
    pub show_progress: bool,
    /// The toast is removed when this reaches zero.
    pub(crate) ttl_sec: f64,
    /// Initial value of ttl_sec, used for progress
    pub(crate) initial_ttl_sec: f64,
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            show_icon: true,
            show_progress: true,
            ttl_sec: f64::INFINITY,
            initial_ttl_sec: f64::INFINITY,
        }
    }
}

impl ToastOptions {
    /// Set duration of the toast. [None] duration means the toast never expires.
    pub fn duration(mut self, duration: impl Into<Option<Duration>>) -> Self {
        self.ttl_sec = duration
            .into()
            .map_or(f64::INFINITY, |duration| duration.as_secs_f64());
        self.initial_ttl_sec = self.ttl_sec;
        self
    }

    /// Set duration of the toast in milliseconds.
    pub fn duration_in_millis(self, millis: u64) -> Self {
        self.duration(Duration::from_millis(millis))
    }

    /// Set duration of the toast in seconds.
    pub fn duration_in_seconds(self, secs: f64) -> Self {
        self.duration(Duration::from_secs_f64(secs))
    }

    /// Visualize remaining time using a progress bar.
    pub fn show_progress(mut self, show_progress: bool) -> Self {
        self.show_progress = show_progress;
        self
    }

    /// Show type icon in the toast.
    pub fn show_icon(mut self, show_icon: bool) -> Self {
        self.show_icon = show_icon;
        self
    }

    /// Remaining time of the toast between 1..0
    pub fn progress(self) -> f64 {
        if self.ttl_sec.is_finite() && self.initial_ttl_sec > 0.0 {
            self.ttl_sec / self.initial_ttl_sec
        } else {
            0.0
        }
    }
}
