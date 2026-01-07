use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};

use gpui::*;

use crate::transition::{
    IntoArcTransition, State, Transition, TransitionRegistry, TransitionStates, general::Linear,
};

#[derive(Clone, Hash, PartialEq, std::cmp::Eq)]
pub enum Event {
    NONE,
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
    pub(crate) on_hover: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    pub(crate) hover_modifier: Option<Rc<dyn Fn(&bool, &mut TransitionStates)>>,
    pub(crate) on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    pub(crate) click_modifier: Option<Rc<dyn Fn(&ClickEvent, &mut TransitionStates)>>,
    pub(crate) bg: Rgba,
    pub(crate) text_bg: Rgba,
    pub(crate) text_color: Rgba,
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

    pub fn text_color(mut self, color: impl Into<Rgba>) -> Self {
        self.text_color = color.into();

        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;

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
        let (cur_bg, cur_text_bg, cur_text_clr, cur_opacity) =
            if let Some(st) = registry.0.get(&self.id) {
                (st.bg.cur, st.text_bg.cur, st.text_color.cur, st.opacity.cur)
            } else {
                (self.bg, self.text_bg, self.text_color, self.opacity)
            };

        let id_for_hover = self.id.clone();
        let on_hover_cb = self.on_hover;
        let hover_mod = self.hover_modifier;
        let hover_bg = self.bg;
        let hover_text_bg = self.text_bg;
        let hover_text_color = self.text_color;
        let hover_opacity = self.opacity;
        let hover_transition = self
            .transitions
            .get(&Event::HOVER)
            .cloned()
            .unwrap_or_else(|| (Duration::default(), Arc::new(Linear)));

        let id_for_click = self.id.clone();
        let on_click_cb = self.on_click;
        let click_mod = self.click_modifier;
        let click_bg = self.bg;
        let click_text_bg = self.text_bg;
        let click_text_color = self.text_color;
        let click_opacity = self.opacity;
        let click_transition = self
            .transitions
            .get(&Event::CLICK)
            .cloned()
            .unwrap_or_else(|| (Duration::default(), Arc::new(Linear)));

        let mut root = div();
        root.style().refine(&self.style);

        root.id(self.id.clone())
            .bg(cur_bg)
            .text_bg(cur_text_bg)
            .text_color(cur_text_clr)
            .opacity(cur_opacity)
            .on_hover(move |hovered, window, app| {
                Self::animated_handle(
                    hovered,
                    window,
                    app,
                    id_for_hover.clone(),
                    on_hover_cb.clone(),
                    hover_mod.clone(),
                    hover_transition.clone(),
                    hover_bg,
                    hover_text_bg,
                    hover_text_color,
                    hover_opacity,
                );
            })
            .on_click(move |event, window, app| {
                Self::animated_handle(
                    event,
                    window,
                    app,
                    id_for_click.clone(),
                    on_click_cb.clone(),
                    click_mod.clone(),
                    click_transition.clone(),
                    click_bg,
                    click_text_bg,
                    click_text_color,
                    click_opacity,
                );
            })
            .child(self.child.children(self.children))
    }
}

impl<E: IntoElement + ParentElement + 'static> AnimatedWrapper<E> {
    fn animated_handle<T>(
        data: &T,
        window: &mut Window,
        app: &mut App,
        id: ElementId,
        callback: Option<Rc<dyn Fn(&T, &mut Window, &mut App)>>,
        modifier: Option<Rc<dyn Fn(&T, &mut TransitionStates)>>,
        transition: (Duration, Arc<dyn Transition>),
        bg: Rgba,
        text_bg: Rgba,
        text_color: Rgba,
        opacity: f32,
    ) {
        if let Some(cb) = callback {
            cb(data, window, app);
        }

        let registry = app.global_mut::<TransitionRegistry>();

        let state = registry
            .0
            .entry(id.clone())
            .or_insert_with(|| TransitionStates {
                bg: State::new(bg),
                text_bg: State::new(text_bg),
                text_color: State::new(text_color),
                opacity: State::new(opacity),
            });

        let state_snapshot = state.clone();
        if let Some(modifier) = modifier {
            modifier(data, state);
        }

        if state_snapshot.ne(state) {
            let (vers, dt) = state.pre_animated(transition.0);

            app.spawn(async move |app| {
                loop {
                    let finished = app
                        .update_global::<TransitionRegistry, bool>(|reg, _| {
                            reg.0
                                .get_mut(&id)
                                .map(|s| s.animated(vers.clone(), dt, transition.1.clone()))
                                .unwrap_or(true)
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
            text_color: Rgba::default(),
            opacity: 1.,
        }
    }
}

impl<T: IntoElement + ParentElement + 'static> TransitionExt for T {}
