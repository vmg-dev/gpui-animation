use std::sync::Arc;

use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_animation::{animation::TransitionExt, transition::Linear};

struct Hover;

impl Render for Hover {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size(px(500.0))
            .justify_center()
            .items_center()
            .text_xl()
            .child("Hover over rectangle")
            .child(
                div().flex().gap_2().child(
                    div()
                        .size_16()
                        .with_transition("Hoverable1")
                        .transition_on_hover(
                            std::time::Duration::from_millis(250),
                            Arc::new(Linear),
                        )
                        .bg(gpui::red())
                        .bg_on_hover(gpui::yellow()),
                ),
            )
            .with_transition("Hoverable2")
            .transition_on_hover(std::time::Duration::from_millis(250), Arc::new(Linear))
            .bg(rgb(0x505050))
            .bg_on_hover(rgb(0xffffff))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Hover),
        )
        .unwrap();
    });
}
