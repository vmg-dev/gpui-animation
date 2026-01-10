use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};

use gpui::{prelude::FluentBuilder, *};

use crate::transition::{
    IntoArcTransition, State, Transition, TransitionRegistry, general::Linear,
};

#[derive(Debug, Clone, Hash, PartialEq, std::cmp::Eq)]
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
    pub fn transition_when<T, I>(
        self,
        condition: bool,
        duration: Duration,
        transition: I,
        then: impl FnOnce(State<StyleRefinement>) -> State<StyleRefinement> + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if condition && TransitionRegistry::modifier_permit(&self.id) {
            Self::animated_handle_without_event(
                self.id.clone(),
                then,
                (duration, transition.into_arc()),
            );
        }

        self
    }

    /**
     * Changes made via .when(), .when_else(), etc., do not automatically trigger the animation cycle. Unlike event-based listeners that hold and manage the App context, these declarative methods do not pass the context to the animation controller. You must manually invoke a refresh or re-render to start the transition.
     */
    pub fn transition_when_else<T, I>(
        self,
        condition: bool,
        duration: Duration,
        transition: I,
        then: impl Fn(State<StyleRefinement>) -> State<StyleRefinement> + 'static,
        else_fn: impl Fn(State<StyleRefinement>) -> State<StyleRefinement> + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if TransitionRegistry::modifier_permit(&self.id) {
            if condition {
                Self::animated_handle_without_event(
                    self.id.clone(),
                    then,
                    (duration, transition.into_arc()),
                );
            } else {
                Self::animated_handle_without_event(
                    self.id.clone(),
                    else_fn,
                    (duration, transition.into_arc()),
                );
            }
        }

        self
    }

    /**
     * Changes made via .when(), .when_else(), etc., do not automatically trigger the animation cycle. Unlike event-based listeners that hold and manage the App context, these declarative methods do not pass the context to the animation controller. You must manually invoke a refresh or re-render to start the transition.
     */
    pub fn transition_when_some<T, I, O>(
        self,
        option: Option<O>,
        duration: Duration,
        transition: I,
        then: impl Fn(State<StyleRefinement>) -> State<StyleRefinement> + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if option.is_some() && TransitionRegistry::modifier_permit(&self.id) {
            Self::animated_handle_without_event(
                self.id.clone(),
                then,
                (duration, transition.into_arc()),
            );
        }

        self
    }

    /**
     * Changes made via .when(), .when_else(), etc., do not automatically trigger the animation cycle. Unlike event-based listeners that hold and manage the App context, these declarative methods do not pass the context to the animation controller. You must manually invoke a refresh or re-render to start the transition.
     */
    pub fn transition_when_none<T, I, O>(
        self,
        option: &Option<O>,
        duration: Duration,
        transition: I,
        then: impl Fn(State<StyleRefinement>) -> State<StyleRefinement> + 'static,
    ) -> Self
    where
        T: Transition + 'static,
        I: IntoArcTransition<T>,
    {
        if option.is_none() && TransitionRegistry::modifier_permit(&self.id) {
            Self::animated_handle_without_event(
                self.id.clone(),
                then,
                (duration, transition.into_arc()),
            );
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
        TransitionRegistry::init(cx);

        let mut root = self.child;

        TransitionRegistry::with_state_default(self.id.clone(), &self.style, |state| {
            root.style().refine(&state.cur);
        });

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

        root.on_hover(move |hovered, window, app| {
            if let Some(cb) = &on_hover_cb {
                cb(hovered, window, app);
            }

            if *hovered {
                TransitionRegistry::add_animation_event(id_for_hover.clone(), Event::HOVER);
            } else {
                TransitionRegistry::remove_animation_event(&id_for_hover, &Event::HOVER);
            }

            Self::animated_handle(
                hovered,
                id_for_hover.clone(),
                hover_mod.clone(),
                hover_transition.clone(),
            );
        })
        .on_click(move |event, window, app| {
            if let Some(cb) = &on_click_cb {
                cb(event, window, app);
            }

            Self::animated_handle(
                event,
                id_for_click.clone(),
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
        id: ElementId,
        modifier: Option<Rc<dyn Fn(&T, State<StyleRefinement>) -> State<StyleRefinement>>>,
        transition: (Duration, Arc<dyn Transition>),
    ) {
        let mut should_start_task = None;

        {
            if let Some(mut state) = TransitionRegistry::state_mut(id.clone()) {
                let state_snapshot = state.clone();

                if let Some(modifier) = modifier {
                    *state = modifier(data, state.clone());
                }

                if state_snapshot.ne(&*state) {
                    let (ver, dt) = state.pre_animated(transition.0);
                    should_start_task = Some((ver, dt));
                }
            } else {
                should_start_task = None;
            }
        }

        if let Some((ver, dt)) = should_start_task {
            TransitionRegistry::background_animated_task(id, dt, transition.1, ver);
        }
    }
    fn animated_handle_without_event(
        id: ElementId,
        modifier: impl FnOnce(State<StyleRefinement>) -> State<StyleRefinement> + 'static,
        transition: (Duration, Arc<dyn Transition>),
    ) {
        let mut should_start_task = None;

        {
            if let Some(mut state) = TransitionRegistry::state_mut(id.clone()) {
                let state_snapshot = state.clone();

                *state = modifier(state.clone());

                if state_snapshot.ne(&*state) {
                    let (ver, dt) = state.pre_animated(transition.0);
                    should_start_task = Some((ver, dt));
                }
            } else {
                should_start_task = None;
            }
        }

        if let Some((ver, dt)) = should_start_task {
            TransitionRegistry::background_animated_task(id, dt, transition.1, ver);
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
