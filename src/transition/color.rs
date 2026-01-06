use crate::transition::Transition;

pub struct Linear;

impl Transition for Linear {
    fn run(&self, start: std::time::Instant, duration: std::time::Duration) -> f32 {
        let elapsed = start.elapsed().as_secs_f32();
        let total = duration.as_secs_f32();

        (elapsed / total).min(1.)
    }
}
