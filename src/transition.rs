use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use gpui::{ElementId, Global, Hsla, Rgba, rgba};

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

pub(crate) trait Interpolatable: Clone {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

impl Interpolatable for Hsla {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        let rgb_from = self.to_rgb();
        let rgb_to = other.to_rgb();

        let r = rgb_from.r + (rgb_to.r - rgb_from.r) * t;
        let g = rgb_from.g + (rgb_to.g - rgb_from.g) * t;
        let b = rgb_from.b + (rgb_to.b - rgb_from.b) * t;
        let a = self.a + (other.a - self.a) * t;

        Rgba { r, g, b, a }.into()
    }
}

#[derive(Clone)]
pub(crate) struct TransitionStates {
    pub bg_transition: (Duration, Arc<dyn Transition>),
    pub bg_from: Hsla,
    pub bg_to: Hsla,
    pub bg_cur: Hsla,
    pub bg_progress: f32,
    pub bg_start_at: Instant,
    pub bg_version: usize,
}

impl Default for TransitionStates {
    fn default() -> Self {
        Self {
            bg_transition: (Duration::default(), Arc::new(Linear)),
            bg_from: Hsla::default(),
            bg_to: Hsla::default(),
            bg_cur: rgba(0x000000ff).into(),
            bg_progress: 1.,
            bg_start_at: Instant::now(),
            bg_version: 0,
        }
    }
}

#[derive(Default)]
pub(crate) struct TransitionRegistry(pub HashMap<ElementId, TransitionStates>);

impl Global for TransitionRegistry {}
