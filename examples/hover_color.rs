use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, linear_color_stop,
    linear_gradient, prelude::*, px, rgb, size,
};
use gpui_animation::{
    animation::TransitionExt,
    transition::{self},
};

struct Hover {
    hovered: bool,
}

impl Render for Hover {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let linear = std::sync::Arc::new(transition::general::Linear);
        let gradient1 = linear_gradient(
            30.,
            linear_color_stop(rgb(0xfccf31), 0.6),
            linear_color_stop(rgb(0xf55555), 0.4),
        );
        let gradient2 = linear_gradient(
            230.,
            linear_color_stop(rgb(0xeead92), 0.6),
            linear_color_stop(rgb(0x6018dc), 0.4),
        );

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
                    .with_transition("Hoverable")
                    .bg(gpui::white())
                    .text_color(gpui::red())
                    .flex()
                    .text_xl()
                    .h_10()
                    .justify_center()
                    .items_center()
                    .transition_on_hover(
                        std::time::Duration::from_millis(250),
                        linear.clone(),
                        |hovered, state| {
                            if *hovered {
                                state
                                    .text_bg(gpui::blue())
                                    .text_color(gpui::yellow())
                                    .text_lg()
                            } else {
                                state
                                    .text_bg(gpui::white())
                                    .text_color(gpui::black())
                                    .text_xl()
                            }
                        },
                    ),
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
                                    state.bg(gpui::yellow()).opacity(0.)
                                } else {
                                    state.bg(gpui::red()).opacity(1.)
                                }
                            },
                        )
                        .opacity(1.)
                        .bg(gpui::red()),
                ),
            )
            .with_transition("Hoverable2")
            .on_hover(cx.listener(|this, hovered, _, cx| {
                this.hovered = *hovered;

                // Changes made via .when(), .when_else(), etc., do not automatically trigger the animation cycle.
                // Unlike event-based listeners that hold and manage the App context, these declarative methods do not pass the context to the animation controller.
                // You must manually invoke a refresh or re-render to start the transition.
                cx.notify();
            }))
            .transition_when_else(
                self.hovered,
                std::time::Duration::from_millis(500),
                transition::general::EaseInExpo,
                move |this| this.bg(gradient2),
                move |this| this.bg(gradient1),
            )
            .transition_on_hover(
                std::time::Duration::from_millis(500),
                transition::general::EaseInExpo,
                |hovered, state| {
                    if *hovered {
                        state.bg(gpui::yellow())
                    } else {
                        state.bg(gpui::red())
                    }
                },
            )
            .bg(gradient1)
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
            |_, cx| cx.new(|_| Hover { hovered: false }),
        )
        .unwrap();
    });
}
