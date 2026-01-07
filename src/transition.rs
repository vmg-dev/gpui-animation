use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use gpui::{
    Background, ElementId, Fill, Global, Hsla, LinearColorStop, Rgba, StyleRefinement, Styled,
    TextStyleRefinement,
};

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

impl Interpolatable for Hsla {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Rgba::from(*self).interpolate(&Rgba::from(*other), t).into()
    }
}

impl Interpolatable for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        *self + (*other - *self) * t
    }
}

macro_rules! refine_interp {
    ($self:expr, $other:expr, $field:ident, $t:expr) => {
        // 用户访问到的是State<StyleRefinement>,other由self.clone得到,因此self存在指定属性则other也存在,这样可以减少分支来提高性能
        $self
            .$field
            .as_ref()
            .map(|a| {
                let b = $other.$field.as_ref().unwrap();
                a.interpolate(b, $t)
            })
            .or_else(|| $other.$field.clone())
    };
}

impl Interpolatable for TextStyleRefinement {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        if t <= 0.0 {
            return self.clone();
        }
        if t >= 1.0 {
            return other.clone();
        }

        Self {
            color: refine_interp!(self, other, color, t),
            background_color: refine_interp!(self, other, background_color, t),

            ..other.clone()
        }
    }
}

#[repr(C)]
pub struct ShadowBackground {
    pad0: [u8; 8],
    pub solid: Hsla,
    pub gradient_angle_or_pattern_height: f32,
    pub colors: [LinearColorStop; 2],
    pad1: u32,
}

impl ShadowBackground {
    pub fn from(bg: &Background) -> &Self {
        unsafe { &*(bg as *const Background as *const Self) }
    }
}

impl Interpolatable for Fill {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        let Fill::Color(bg_start) = self;
        let Fill::Color(bg_end) = other;

        Fill::from(
            ShadowBackground::from(bg_start)
                .solid
                .interpolate(&ShadowBackground::from(bg_end).solid, t),
        )
    }
}

impl Interpolatable for StyleRefinement {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        if t <= 0.0 {
            return self.clone();
        }
        if t >= 1.0 {
            return other.clone();
        }

        StyleRefinement {
            text: refine_interp!(self, other, text, t),
            background: refine_interp!(self, other, background, t),
            opacity: refine_interp!(self, other, opacity, t),

            ..other.clone()
        }
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

impl Styled for State<StyleRefinement> {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.to
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

#[derive(Default)]
pub(crate) struct TransitionRegistry(pub HashMap<ElementId, State<StyleRefinement>>);

impl Global for TransitionRegistry {}
