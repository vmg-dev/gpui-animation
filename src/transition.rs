use std::collections::HashSet;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{sync::Arc, time::Duration};

use dashmap::DashMap;
use gpui::*;
use smol::channel::{self, Receiver, Sender};

use crate::animation::Event;
use crate::interpolate::State;

pub mod color;
pub mod general;
pub mod position;

pub trait Transition: Send + Sync + 'static {
    fn run(&self, start: std::time::Instant, duration: std::time::Duration) -> f32 {
        let t = (start.elapsed().as_secs_f32() / duration.as_secs_f32()).min(1.0);
        self.calculate(t)
    }

    fn calculate(&self, t: f32) -> f32;
}

pub trait IntoArcTransition<T: Transition + 'static> {
    fn into_arc(self) -> Arc<T>;
}

impl<T: Transition + 'static> IntoArcTransition<T> for T {
    fn into_arc(self) -> Arc<T> {
        Arc::new(self)
    }
}

impl<T: Transition + 'static> IntoArcTransition<T> for Arc<T> {
    fn into_arc(self) -> Arc<T> {
        self
    }
}

pub(crate) struct ActiveAnimation {
    duration: Duration,
    transition: Arc<dyn Transition>,
    ver: usize,
}

pub(crate) struct TransitionRegistry {
    initialized: AtomicBool,
    states: DashMap<ElementId, State<StyleRefinement>>,
    active_animations: DashMap<ElementId, ActiveAnimation>,
    active_events: DashMap<ElementId, HashSet<Event>>,
    wakeup_tx: Sender<()>,
    wakeup_rx: Receiver<()>,
}

pub(crate) static TRANSITION_REGISTRY: LazyLock<TransitionRegistry> = LazyLock::new(|| {
    let (tx, rx) = channel::unbounded();

    TransitionRegistry {
        initialized: AtomicBool::new(false),
        states: Default::default(),
        active_animations: Default::default(),
        active_events: Default::default(),
        wakeup_tx: tx,
        wakeup_rx: rx,
    }
});

impl TransitionRegistry {
    pub fn init(cx: &mut App) {
        if !TRANSITION_REGISTRY.initialized.swap(true, Ordering::SeqCst) {
            cx.spawn(Self::animation_tick).detach();
        }
    }

    pub fn background_animated_task(
        id: ElementId,
        duration: Duration,
        transition: Arc<dyn Transition>,
        ver: usize,
    ) {
        TRANSITION_REGISTRY.active_animations.insert(
            id,
            ActiveAnimation {
                duration,
                transition,
                ver,
            },
        );

        TRANSITION_REGISTRY.wakeup_tx.try_send(()).ok();
    }

    pub fn modifier_permit(id: &ElementId) -> bool {
        TRANSITION_REGISTRY.active_events.get(id).is_none()
    }

    pub fn add_animation_event(id: ElementId, event: Event) {
        TRANSITION_REGISTRY
            .active_events
            .entry(id.clone())
            .or_insert_with(HashSet::new)
            .insert(event);
    }

    pub fn remove_animation_event(id: &ElementId, event: &Event) {
        if let Some(mut events) = TRANSITION_REGISTRY.active_events.get_mut(id) {
            events.remove(event);

            if events.is_empty() {
                drop(events);
                TRANSITION_REGISTRY.active_events.remove(id);
            }
        }
    }

    pub async fn animation_tick(cx: &mut AsyncApp) {
        // least 120TPS
        let frame_duration = Duration::from_secs_f32(1. / 120.);
        let registry = &*TRANSITION_REGISTRY;

        loop {
            let mut changed = false;

            {
                registry.active_animations.retain(|id, active| {
                    if let Some(mut state) = registry.states.get_mut(id) {
                        changed = true;
                        !state.animated(active.ver, active.duration, &active.transition)
                    } else {
                        false
                    }
                });
            }

            if changed {
                cx.update(|cx| cx.refresh_windows()).ok();
            }

            if registry.active_animations.is_empty() {
                registry.wakeup_rx.recv().await.ok();
            } else {
                smol::Timer::after(frame_duration).await;
            }
        }
    }

    #[inline]
    pub fn with_state_default<R>(
        id: ElementId,
        default: &StyleRefinement,
        f: impl FnOnce(&mut State<StyleRefinement>) -> R,
    ) -> R {
        let mut state = TRANSITION_REGISTRY
            .states
            .entry(id)
            .or_insert_with(|| State::new(default.clone()));

        f(&mut *state)
    }

    #[inline]
    pub fn state_mut(
        id: ElementId,
    ) -> Option<dashmap::mapref::one::RefMut<'static, ElementId, State<StyleRefinement>>> {
        if !TRANSITION_REGISTRY
            .initialized
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return None;
        }

        TRANSITION_REGISTRY.states.get_mut(&id)
    }
}
