use std::{rc::Rc, time::Duration};

use gpui::{prelude::FluentBuilder, *};
use gpui_animation::{
    animation::TransitionExt,
    transition::general::{self, Linear},
};

#[derive(IntoElement)]
struct Button {
    id: ElementId,
    style: StyleRefinement,
    children: Vec<AnyElement>,
    on_hover: Option<Rc<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    selected: Option<bool>,
    disabled: Option<bool>,
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: Default::default(),
            children: Default::default(),
            on_hover: None,
            on_click: None,
            selected: None,
            disabled: None,
        }
    }

    #[allow(dead_code)]
    pub fn on_hover(mut self, f: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Rc::new(f));

        self
    }

    #[allow(dead_code)]
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));

        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);

        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);

        self
    }
}

impl RenderOnce for Button {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut root = div()
            .id(self.id.clone())
            .flex()
            .items_center()
            .justify_center()
            .w_full()
            .border_1()
            .border_color(rgb(0x262626))
            .rounded_md()
            .h_8()
            .bg(rgb(0x0a0a0a));

        root.style().refine(&self.style);

        root.children(self.children)
            .with_transition(self.id)
            .when_else(
                self.disabled.unwrap_or_default(),
                |this| this.bg(rgb(0x333333)).cursor_not_allowed(),
                |this| {
                    this.when_some(self.on_hover, |this, on_hover| {
                        this.on_hover(move |h, w, a| {
                            (on_hover)(h, w, a);
                        })
                    })
                    .when_some(self.on_click, |this, on_click| {
                        this.on_click(move |e, w, a| {
                            (on_click)(e, w, a);
                        })
                    })
                    .transition_when_else(
                        self.selected.unwrap_or_default(),
                        Duration::from_millis(250),
                        Linear,
                        |this| this.bg(rgb(0x1a1a1a)),
                        |this| this.bg(rgb(0x0a0a0a)),
                    )
                    .transition_on_hover(Duration::from_millis(250), Linear, |hovered, this| {
                        if *hovered {
                            this.bg(rgb(0x1a1a1a))
                        } else {
                            this
                        }
                    })
                    .when(!self.selected.unwrap_or_default(), |this| {
                        this.transition_on_click(
                            Duration::from_millis(150),
                            general::EaseInExpo,
                            |_, this| this.bg(rgb(0x262626)),
                        )
                    })
                },
            )
    }
}

struct BasicBackground;

impl Render for BasicBackground {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let text_color = rgb(0xffffff);

        div()
            .flex()
            .flex_col()
            .gap_3()
            .size(px(500.0))
            .justify_center()
            .items_center()
            .bg(rgb(0x0a0a0a))
            .child(
                Button::new("Button1")
                    .child("Button1")
                    .text_color(text_color),
            )
            .child(
                Button::new("Button2")
                    .child("Button2")
                    .selected(true)
                    .text_color(text_color),
            )
            .child(
                Button::new("Button3")
                    .child("Button3")
                    .disabled(true)
                    .text_color(text_color),
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
            |_, cx| cx.new(|_| BasicBackground),
        )
        .unwrap();
    });
}
