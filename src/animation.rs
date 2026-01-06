use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};

use gpui::*;

use crate::transition::{
    IntoArcTransition, State, Transition, TransitionRegistry, TransitionStates, general::Linear,
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
    pub(crate) hover_modifier: Option<Rc<dyn Fn(&bool, &mut TransitionStates)>>,
    pub(crate) click_modifier: Option<Rc<dyn Fn(&ClickEvent, &mut TransitionStates)>>,
    pub(crate) bg: Rgba,
    pub(crate) text_bg: Rgba,
    pub(crate) opacity: f32,
}

impl<E: IntoElement + ParentElement + 'static> AnimatedWrapper<E> {
    pub fn transition_on_hover<T, I>(
        mut self,
        duration: Duration,
        transition: I,
        modifier: impl Fn(&bool, &mut TransitionStates) + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        self.transitions
            .insert(Event::HOVER, (duration, transition.into_arc()));
        self.hover_modifier = Some(Rc::new(modifier));

        self
    }

    pub fn transition_on_click<T, I>(
        mut self,
        duration: Duration,
        transition: I,
        modifier: impl Fn(&ClickEvent, &mut TransitionStates) + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        self.transitions
            .insert(Event::CLICK, (duration, transition.into_arc()));
        self.click_modifier = Some(Rc::new(modifier));

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

    pub fn bg(mut self, color: impl Into<Rgba>) -> Self {
        self.bg = color.into();

        self
    }

    pub fn text_bg(mut self, color: impl Into<Rgba>) -> Self {
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
                bg: State {
                    cur: self.bg,
                    ..Default::default()
                },
                opacity: State {
                    cur: self.opacity,
                    ..Default::default()
                },
            });

        let mut root = div();

        root.style().refine(&self.style);

        root.id(self.id.clone())
            .size_full()
            .bg(states.bg.cur)
            .opacity(states.opacity.cur)
            .on_hover(move |hovered, window, app| {
                if let Some(cb) = self.on_hover.clone() {
                    cb(hovered, window, app);
                }

                let registry = app.global_mut::<TransitionRegistry>();

                let state = registry
                    .0
                    .entry(id.clone())
                    .or_insert_with(|| TransitionStates {
                        bg: State {
                            transition: self
                                .transitions
                                .get(&Event::HOVER)
                                .map(|v| v.clone())
                                .unwrap_or((Duration::default(), Arc::new(Linear))),
                            cur: self.bg,
                            from: self.bg,
                            to: self.bg,
                            ..Default::default()
                        },
                        opacity: State {
                            cur: self.opacity,
                            from: self.opacity,
                            to: self.opacity,
                            ..Default::default()
                        },
                    });

                let state_snapshot = state.clone();
                if let Some(hover_modifier) = self.hover_modifier.clone() {
                    hover_modifier(hovered, state);
                }

                if state_snapshot.ne(state) {
                    let (version, bg_duration) = state.bg.pre_animated();
                    let _ = state.opacity.pre_animated();

                    let id = id.clone();

                    app.spawn(async move |app| {
                        loop {
                            let finished = app
                                .update_global::<TransitionRegistry, bool>(|reg, _| {
                                    if let Some(state) = reg.0.get_mut(&id) {
                                        if version != state.bg.version {
                                            return true;
                                        }

                                        let _ = state.opacity.animated(bg_duration);
                                        state.bg.animated(bg_duration)
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
            hover_modifier: None,
            click_modifier: None,
            bg: Rgba::default(),
            text_bg: Rgba::default(),
            opacity: 1.,
        }
    }
}

impl<T: IntoElement + ParentElement + 'static> TransitionExt for T {}
