use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use gpui::{ElementId, Global, Rgba};

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

impl Interpolatable for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        *self + (*other - *self) * t
    }
}

#[derive(Clone)]
pub struct State<T: Interpolatable + Default + PartialEq> {
    pub(crate) from: T,
    pub(crate) to: T,
    pub(crate) cur: T,
    pub(crate) progress: f32,
    pub(crate) start_at: Instant,
    pub(crate) version: usize,
}

impl<T: Interpolatable + Default + PartialEq> PartialEq for State<T> {
    fn eq(&self, other: &Self) -> bool {
        self.to.eq(&other.to)
    }

    fn ne(&self, other: &Self) -> bool {
        self.to.ne(&other.to)
    }
}

impl<T: Interpolatable + Default + PartialEq> Default for State<T> {
    fn default() -> Self {
        Self {
            from: T::default(),
            to: T::default(),
            cur: T::default(),
            progress: 1.,
            start_at: Instant::now(),
            version: 0,
        }
    }
}

impl<T: Interpolatable + Default + PartialEq> State<T> {
    pub fn new(init: T) -> Self {
        Self {
            cur: init.clone(),
            from: init.clone(),
            to: init,
            ..Default::default()
        }
    }

    pub fn pre_animated(&mut self, dt: Duration) -> (usize, Duration) {
        self.version += 1;

        let is_reversing = self.to == self.from;

        let actual_duration = if is_reversing {
            dt.mul_f32(self.progress)
        } else {
            dt
        };

        self.from = self.cur.clone();
        self.start_at = Instant::now();

        self.progress = 0.;

        (self.version, actual_duration)
    }

    pub fn animated(
        &mut self,
        ss_ver: usize,
        dt: Duration,
        transition: Arc<dyn Transition>,
    ) -> bool {
        if ss_ver.ne(&self.version) {
            return true;
        }

        self.progress = transition.run(self.start_at, dt);
        self.cur = self.from.interpolate(&self.to, self.progress);

        self.progress >= 1.
    }
}

#[derive(Clone)]
pub struct VersionSnapshot {
    bg: usize,
    text_bg: usize,
    text_color: usize,
    opacity: usize,
}

#[derive(Clone, Default, PartialEq)]
pub struct TransitionStates {
    pub(crate) bg: State<Rgba>,
    pub(crate) text_bg: State<Rgba>,
    pub(crate) text_color: State<Rgba>,
    pub(crate) opacity: State<f32>,
}

impl TransitionStates {
    pub(crate) fn pre_animated(&mut self, dt: Duration) -> (VersionSnapshot, Duration) {
        let (bg_ver, duration) = self.bg.pre_animated(dt);
        let (text_bg_ver, _) = self.text_bg.pre_animated(dt);
        let (text_color_ver, _) = self.text_color.pre_animated(dt);
        let (opacity_ver, _) = self.opacity.pre_animated(dt);

        (
            VersionSnapshot {
                bg: bg_ver,
                text_bg: text_bg_ver,
                text_color: text_color_ver,
                opacity: opacity_ver,
            },
            duration,
        )
    }

    pub(crate) fn animated(
        &mut self,
        versions: VersionSnapshot,
        dt: Duration,
        transition: Arc<dyn Transition>,
    ) -> bool {
        let b_done = self.bg.animated(versions.bg, dt, transition.clone());
        let t_done = self
            .text_bg
            .animated(versions.text_bg, dt, transition.clone());
        let tc_done = self
            .text_color
            .animated(versions.text_color, dt, transition.clone());
        let o_done = self.opacity.animated(versions.opacity, dt, transition);

        b_done && t_done && tc_done && o_done
    }

    pub fn bg(&mut self, color: impl Into<Rgba>) -> &mut Self {
        self.bg.to = color.into();

        self
    }

    pub fn text_bg(&mut self, color: impl Into<Rgba>) -> &mut Self {
        self.text_bg.to = color.into();

        self
    }

    pub fn text_color(&mut self, color: impl Into<Rgba>) -> &mut Self {
        self.text_color.to = color.into();

        self
    }

    pub fn opacity(&mut self, value: f32) -> &mut Self {
        self.opacity.to = value;

        self
    }
}

#[derive(Default)]
pub(crate) struct TransitionRegistry(pub HashMap<ElementId, TransitionStates>);

impl Global for TransitionRegistry {}
