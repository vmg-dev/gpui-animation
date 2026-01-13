use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    size,
};
use gpui_animation::{
    animation::TransitionExt,
    transition::{self},
};

struct Hover;

impl Render for Hover {
    fn render(&mut self, _window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let linear = std::sync::Arc::new(transition::general::Linear);

        div()
            .id("Hoverable2")
            .flex()
            .flex_col()
            .gap_3()
            .size(px(500.0))
            .justify_center()
            .items_center()
            .child(
                div()
                    .id("Hoverable")
                    .child("Hover over rectangle")
                    .bg(gpui::white())
                    .text_color(gpui::red())
                    .flex()
                    .text_xl()
                    .h_10()
                    .justify_center()
                    .items_center(),
            )
            .child(
                div().flex().gap_2().child(
                    div()
                        .id("Hoverable1")
                        .size_16()
                        .with_transition("Hoverable1")
                        .transition_on_hover(
                            std::time::Duration::from_millis(250),
                            linear.clone(),
                            |hovered, state| {
                                if *hovered {
                                    state.size_32().ml_24()
                                } else {
                                    state.size_16().ml_0()
                                }
                            },
                        )
                        .bg(gpui::red())
                        .ml_0(),
                ),
            )
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
