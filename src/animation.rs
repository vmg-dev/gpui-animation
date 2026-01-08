use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};

use gpui::{prelude::FluentBuilder, *};

use crate::transition::{
    IntoArcTransition, State, Transition, TransitionRegistry, general::Linear,
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
    E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static,
{
    style: StyleRefinement,
    children: Vec<AnyElement>,
    id: ElementId,
    child: E,
    transitions: HashMap<Event, (Duration, Arc<dyn Transition>)>,
    on_hover: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    hover_modifier: Option<Rc<dyn Fn(&bool, State<StyleRefinement>) -> State<StyleRefinement>>>,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    click_modifier:
        Option<Rc<dyn Fn(&ClickEvent, State<StyleRefinement>) -> State<StyleRefinement>>>,

    init_modifiers: Vec<Rc<dyn Fn(State<StyleRefinement>) -> State<StyleRefinement>>>,
}

impl<E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static>
    AnimatedWrapper<E>
{
    pub fn transition_on_hover<T, I>(
        mut self,
        duration: Duration,
        transition: I,
        modifier: impl Fn(&bool, State<StyleRefinement>) -> State<StyleRefinement> + 'static,
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
        modifier: impl Fn(&ClickEvent, State<StyleRefinement>) -> State<StyleRefinement> + 'static,
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

    /**
     * Changes made via .when(), .when_else(), etc., do not automatically trigger the animation cycle. Unlike event-based listeners that hold and manage the App context, these declarative methods do not pass the context to the animation controller. You must manually invoke a refresh or re-render to start the transition.
     */
    pub fn when(
        mut self,
        condition: bool,
        then: impl Fn(State<StyleRefinement>) -> State<StyleRefinement> + 'static,
    ) -> Self {
        if condition {
            self.init_modifiers.push(Rc::new(then));
        }

        self
    }

    /**
     * Changes made via .when(), .when_else(), etc., do not automatically trigger the animation cycle. Unlike event-based listeners that hold and manage the App context, these declarative methods do not pass the context to the animation controller. You must manually invoke a refresh or re-render to start the transition.
     */
    pub fn when_else(
        mut self,
        condition: bool,
        then: impl Fn(State<StyleRefinement>) -> State<StyleRefinement> + 'static,
        else_fn: impl Fn(State<StyleRefinement>) -> State<StyleRefinement> + 'static,
    ) -> Self {
        if condition {
            self.init_modifiers.push(Rc::new(then));
        } else {
            self.init_modifiers.push(Rc::new(else_fn))
        }

        self
    }

    /**
     * Changes made via .when(), .when_else(), etc., do not automatically trigger the animation cycle. Unlike event-based listeners that hold and manage the App context, these declarative methods do not pass the context to the animation controller. You must manually invoke a refresh or re-render to start the transition.
     */
    pub fn when_some<T>(
        mut self,
        option: Option<T>,
        then: impl Fn(State<StyleRefinement>) -> State<StyleRefinement> + 'static,
    ) -> Self {
        if option.is_some() {
            self.init_modifiers.push(Rc::new(then));
        }

        self
    }

    /**
     * Changes made via .when(), .when_else(), etc., do not automatically trigger the animation cycle. Unlike event-based listeners that hold and manage the App context, these declarative methods do not pass the context to the animation controller. You must manually invoke a refresh or re-render to start the transition.
     */
    pub fn when_none<T>(
        mut self,
        option: &Option<T>,
        then: impl Fn(State<StyleRefinement>) -> State<StyleRefinement> + 'static,
    ) -> Self {
        if option.is_none() {
            self.init_modifiers.push(Rc::new(then));
        }

        self
    }
}

impl<E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static>
    AnimatedWrapper<E>
{
    fn with_transition(mut child: E, id: impl Into<ElementId>) -> Self {
        Self {
            style: child.style().clone(),
            children: Vec::new(),
            id: id.into(),
            child,
            transitions: HashMap::new(),
            on_click: None,
            on_hover: None,
            hover_modifier: None,
            click_modifier: None,
            init_modifiers: Vec::new(),
        }
    }
}

impl<E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static>
    Styled for AnimatedWrapper<E>
{
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl<E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static>
    ParentElement for AnimatedWrapper<E>
{
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl<E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static>
    RenderOnce for AnimatedWrapper<E>
{
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let registry = cx.default_global::<TransitionRegistry>();
        let mut state = registry
            .0
            .entry(self.id.clone())
            .or_insert_with(|| State::new(self.style.clone()))
            .clone();

        let initial_state = state.clone();
        for modifier in &self.init_modifiers {
            state = modifier(state);
        }

        if initial_state != state {
            let registry = cx.global_mut::<TransitionRegistry>();
            if let Some(reg_state) = registry.0.get_mut(&self.id) {
                *reg_state = state.clone();

                let transition = self
                    .transitions
                    .values()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| (Duration::from_millis(200), Arc::new(Linear)));

                let (vers, dt) = reg_state.pre_animated(transition.0);
                let id = self.id.clone();

                cx.spawn(async move |app| {
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

        let style = &state.cur;

        let id_for_hover = self.id.clone();
        let on_hover_cb = self.on_hover;
        let hover_mod = self.hover_modifier;
        let hover_transition = self
            .transitions
            .get(&Event::HOVER)
            .cloned()
            .unwrap_or_else(|| (Duration::default(), Arc::new(Linear)));

        let id_for_click = self.id.clone();
        let on_click_cb = self.on_click;
        let click_mod = self.click_modifier;
        let click_transition = self
            .transitions
            .get(&Event::CLICK)
            .cloned()
            .unwrap_or_else(|| (Duration::default(), Arc::new(Linear)));

        let mut root = self.child;

        root.style().refine(style);

        root.on_hover(move |hovered, window, app| {
            Self::animated_handle(
                hovered,
                window,
                app,
                id_for_hover.clone(),
                on_hover_cb.clone(),
                hover_mod.clone(),
                hover_transition.clone(),
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
            );
        })
        .children(self.children)
    }
}

impl<E: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static>
    AnimatedWrapper<E>
{
    fn animated_handle<T>(
        data: &T,
        window: &mut Window,
        app: &mut App,
        id: ElementId,
        callback: Option<Rc<dyn Fn(&T, &mut Window, &mut App)>>,
        modifier: Option<Rc<dyn Fn(&T, State<StyleRefinement>) -> State<StyleRefinement>>>,
        transition: (Duration, Arc<dyn Transition>),
    ) {
        if let Some(cb) = callback {
            cb(data, window, app);
        }

        let registry = app.global_mut::<TransitionRegistry>();

        let state = registry.0.get_mut(&id).unwrap();

        let state_snapshot = state.clone();
        if let Some(modifier) = modifier {
            *state = modifier(data, state.clone());
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

pub trait TransitionExt:
    IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static
{
    fn with_transition(self, id: impl Into<ElementId>) -> AnimatedWrapper<Self> {
        AnimatedWrapper::with_transition(self, id)
    }
}

impl<T: IntoElement + StatefulInteractiveElement + ParentElement + FluentBuilder + Styled + 'static>
    TransitionExt for T
{
}
