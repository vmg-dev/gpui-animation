use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};

use gpui::*;

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
    E: IntoElement + ParentElement + 'static,
{
    pub(crate) style: StyleRefinement,
    pub(crate) children: Vec<AnyElement>,
    pub(crate) id: ElementId,
    pub(crate) child: E,
    pub(crate) transitions: HashMap<Event, (Duration, Arc<dyn Transition>)>,
    pub(crate) on_hover: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    pub(crate) hover_modifier:
        Option<Rc<dyn Fn(&bool, State<StyleRefinement>) -> State<StyleRefinement>>>,
    pub(crate) on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    pub(crate) click_modifier:
        Option<Rc<dyn Fn(&ClickEvent, State<StyleRefinement>) -> State<StyleRefinement>>>,
}

impl<E: IntoElement + ParentElement + 'static> AnimatedWrapper<E> {
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
        let _ = registry
            .0
            .entry(self.id.clone())
            .or_insert_with(|| State::new(self.style.clone()));

        let style = if let Some(st) = registry.0.get(&self.id) {
            &st.cur
        } else {
            &self.style
        };

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

        let mut root = div().size_full();
        root.style().refine(style);

        root.id(self.id.clone())
            .on_hover(move |hovered, window, app| {
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

pub trait TransitionExt: IntoElement + ParentElement + Styled + 'static {
    fn with_transition(mut self, id: impl Into<ElementId>) -> AnimatedWrapper<Self> {
        AnimatedWrapper {
            style: self.style().clone(),
            children: Vec::new(),
            id: id.into(),
            child: self,
            transitions: HashMap::new(),
            on_click: None,
            on_hover: None,
            hover_modifier: None,
            click_modifier: None,
        }
    }
}

impl<T: IntoElement + ParentElement + Styled + 'static> TransitionExt for T {}
