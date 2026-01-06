use std::{
    collections::HashMap,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use gpui::*;

use crate::transition::{
    Interpolatable, Transition, TransitionRegistry, TransitionStates, color::Linear,
};

#[derive(Hash, PartialEq, std::cmp::Eq)]
pub enum Event {
    HOVER,
    CLICK,
}

#[derive(IntoElement)]
pub struct AnimatedWrapper<E>
where
    E: IntoElement + ParentElement + 'static,
{
    pub(crate) style: StyleRefinement,
    pub(crate) children: Vec<AnyElement>,
    pub(crate) id: ElementId,
    pub(crate) child: E,
    pub(crate) transitions: HashMap<Event, (Duration, Arc<dyn Transition>)>,
    pub(crate) on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    pub(crate) on_hover: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    pub(crate) bg: Hsla,
    pub(crate) bg_on_hover: Hsla,
    pub(crate) bg_on_click: Hsla,
    pub(crate) text_bg: Hsla,
}

impl<E: IntoElement + ParentElement + 'static> AnimatedWrapper<E> {
    pub fn transition_on_hover<T: Transition>(mut self, duration: Duration, transition: T) -> Self {
        self.transitions
            .insert(Event::HOVER, (duration, Arc::new(transition)));

        self
    }

    pub fn transition_on_click<T: Transition>(mut self, duration: Duration, transition: T) -> Self {
        self.transitions
            .insert(Event::CLICK, (duration, Arc::new(transition)));

        self
    }

    pub fn on_hover(mut self, listener: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Rc::new(listener));

        self
    }

    pub fn on_click(
        mut self,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(listener));

        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = color.into();

        self
    }

    pub fn bg_on_hover(mut self, color: impl Into<Hsla>) -> Self {
        self.bg_on_hover = color.into();

        self
    }

    pub fn bg_on_click(mut self, color: impl Into<Hsla>) -> Self {
        self.bg_on_click = color.into();

        self
    }

    pub fn text_bg(mut self, color: impl Into<Hsla>) -> Self {
        self.text_bg = color.into();

        self
    }
}

impl<E: IntoElement + ParentElement + 'static> Styled for AnimatedWrapper<E> {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl<E: IntoElement + ParentElement + 'static> ParentElement for AnimatedWrapper<E> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl<E: IntoElement + ParentElement + 'static> RenderOnce for AnimatedWrapper<E> {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let registry = cx.default_global::<TransitionRegistry>();
        let id = self.id.clone();
        let states = registry
            .0
            .get(&self.id)
            .cloned()
            .unwrap_or_else(|| TransitionStates {
                bg_cur: self.bg,
                ..Default::default()
            });

        let mut root = div();

        root.style().refine(&self.style);

        root.id(self.id.clone())
            .size_full()
            .bg(states.bg_cur)
            .on_hover(move |hovered, window, app| {
                if let Some(cb) = self.on_hover.clone() {
                    cb(hovered, window, app);
                }

                let registry = app.global_mut::<TransitionRegistry>();

                let state = registry
                    .0
                    .entry(id.clone())
                    .or_insert_with(|| TransitionStates {
                        bg_transition: self
                            .transitions
                            .get(&Event::HOVER)
                            .map(|v| v.clone())
                            .unwrap_or((Duration::default(), Arc::new(Linear))),
                        bg_cur: self.bg,
                        bg_from: self.bg,
                        bg_to: self.bg,
                        ..Default::default()
                    });

                let target_bg = if *hovered {
                    self.bg_on_hover.clone()
                } else {
                    self.bg.clone()
                };
                if state.bg_to != target_bg {
                    state.bg_version += 1;
                    let version = state.bg_version;
                    state.bg_from = state.bg_cur;
                    state.bg_to = target_bg;
                    state.bg_start_at = Instant::now();

                    let bg_duration = state.bg_transition.0.mul_f32(state.bg_progress);

                    state.bg_progress = 0.;

                    let id = id.clone();

                    app.spawn(async move |app| {
                        loop {
                            let finished = app
                                .update_global::<TransitionRegistry, bool>(|reg, _| {
                                    if let Some(state) = reg.0.get_mut(&id) {
                                        if version != state.bg_version {
                                            return true;
                                        }

                                        state.bg_progress = state
                                            .bg_transition
                                            .1
                                            .run(state.bg_start_at, bg_duration);
                                        state.bg_cur = state
                                            .bg_from
                                            .interpolate(&state.bg_to, state.bg_progress);

                                        state.bg_progress >= 1.
                                    } else {
                                        true
                                    }
                                })
                                .unwrap_or(true);

                            let _ = app.refresh();

                            app.background_executor()
                                .timer(Duration::from_millis(8))
                                .await;

                            if finished {
                                break;
                            }
                        }
                    })
                    .detach();
                }
            })
            .child(self.child.children(self.children))
    }
}

pub trait TransitionExt: IntoElement + ParentElement + 'static {
    fn with_transition(self, id: impl Into<ElementId>) -> AnimatedWrapper<Self> {
        AnimatedWrapper {
            style: StyleRefinement::default(),
            children: Vec::new(),
            id: id.into(),
            child: self,
            transitions: HashMap::new(),
            on_click: None,
            on_hover: None,
            bg: Hsla::default(),
            bg_on_hover: Hsla::default(),
            bg_on_click: Hsla::default(),
            text_bg: Hsla::default(),
        }
    }
}

impl<T: IntoElement + ParentElement + 'static> TransitionExt for T {}
