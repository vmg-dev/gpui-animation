use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_animation::{
    animation::TransitionExt,
    transition::{self},
};

struct Hover;

impl Render for Hover {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let linear = std::sync::Arc::new(transition::general::Linear);

        div()
            .flex()
            .flex_col()
            .gap_3()
            .size(px(500.0))
            .justify_center()
            .items_center()
            .child(
                div()
                    .child("Hover over rectangle")
                    .with_transition("Hoverable")
                    .text_color(gpui::red())
                    .flex()
                    .text_xl()
                    .justify_center()
                    .items_center()
                    .transition_on_hover(
                        std::time::Duration::from_millis(250),
                        linear.clone(),
                        |hovered, state| {
                            if *hovered {
                                state.text_bg(gpui::blue()).text_color(gpui::yellow());
                            } else {
                                state.text_bg(gpui::white()).text_color(gpui::black());
                            }
                        },
                    ),
            )
            .child(
                div().flex().gap_2().child(
                    div()
                        .size_16()
                        .with_transition("Hoverable1")
                        .transition_on_hover(
                            std::time::Duration::from_millis(250),
                            linear.clone(),
                            |hovered, state| {
                                if *hovered {
                                    state.bg(gpui::yellow()).opacity(0.);
                                } else {
                                    state.bg(gpui::red()).opacity(1.);
                                }
                            },
                        )
                        .bg(gpui::red()),
                ),
            )
            .with_transition("Hoverable2")
            .transition_on_hover(
                std::time::Duration::from_millis(250),
                linear.clone(),
                |hovered, state| {
                    state.bg(if *hovered {
                        rgb(0xffffff)
                    } else {
                        rgb(0x505050)
                    });
                },
            )
            .bg(rgb(0x505050))
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
