# egui-toast

[![Latest version](https://img.shields.io/crates/v/egui-toast.svg)](https://crates.io/crates/egui-toast)
[![Documentation](https://docs.rs/egui-toast/badge.svg)](https://docs.rs/egui-toast)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Toast notifications for the [egui](https://github.com/emilk/egui) library.

![Toast types](toasts.png)

## Quick start

`cargo run --example demo`

```rust
let mut toasts = Toasts::new()
    .anchor((300.0, 300.0))
    .direction(egui::Direction::BottomUp)
    .align_to_end(true);

toasts.info(ui, "Hello, World!", Duration::from_secs(5));
// or
toasts.warning(ui, "Hello, World!", ToastOptions {
    show_icon: true,
    ..ToastOptions::with_duration(Duration::from_secs(5))
});
// or
toasts.add(ui, "Hello, World!", ToastKind::Error, Duration::from_secs(5));

// Show all toasts
toasts.show(ctx);
```

## Customization

Look of the notifications can be fully customized.

```rust
const MY_CUSTOM_TOAST: u32 = 0;

fn my_custom_toast_contents(ui: &mut Ui, toast: &mut Toast) {
    egui::Frame::default()
        .fill(Color32::from_rgb(33, 150, 243))
        .inner_margin(Margin::same(12.0))
        .rounding(4.0)
        .show(ui, |ui| {
            ui.label(toast.text.clone().color(Color32::WHITE));

            if ui.button("Close me").clicked() {
                toast.close();
            }
        });
}

let mut toasts = Toasts::new()
    .custom_contents(MY_CUSTOM_TOAST, &my_custom_toast_contents);

if ui.button("Add toast").clicked() {
    toasts.add(ui, "Hello, World!", MY_CUSTOM_TOAST, ToastOptions::default());
}

toasts.show(ctx);
```