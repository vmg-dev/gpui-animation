use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use gpui::{ElementId, Global, Rgba};

use general::Linear;

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

pub trait Interpolatable: Clone {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

impl Interpolatable for Rgba {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        let r = self.r + (other.r - self.r) * t;
        let g = self.g + (other.g - self.g) * t;
        let b = self.b + (other.b - self.b) * t;
        let a = self.a + (other.a - self.a) * t;

        Rgba { r, g, b, a }
    }
}

#[derive(Clone)]
pub struct State<T: Interpolatable + Default> {
    pub(crate) transition: (Duration, Arc<dyn Transition>),
    pub(crate) from: T,
    pub(crate) to: T,
    pub(crate) cur: T,
    pub(crate) progress: f32,
    pub(crate) start_at: Instant,
    pub(crate) version: usize,
}

impl<T: Interpolatable + Default> Default for State<T> {
    fn default() -> Self {
        Self {
            transition: (Duration::default(), Arc::new(Linear)),
            from: T::default(),
            to: T::default(),
            cur: T::default(),
            progress: 1.,
            start_at: Instant::now(),
            version: 0,
        }
    }
}

#[derive(Clone, Default)]
pub struct TransitionStates {
    pub(crate) bg: State<Rgba>,
}

impl TransitionStates {
    pub fn bg(&mut self, color: impl Into<Rgba>) -> &Self {
        self.bg.to = color.into();

        self
    }
}

#[derive(Default)]
pub(crate) struct TransitionRegistry(pub HashMap<ElementId, TransitionStates>);

impl Global for TransitionRegistry {}
