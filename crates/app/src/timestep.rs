const MAX_STEPS: u32 = 8;

#[derive(Default)]
pub(crate) struct FixedTimestep {
    accumulator: f32,
}

impl FixedTimestep {
    pub(crate) fn advance(&mut self, dt: f32, step: f32) -> u32 {
        self.accumulator += dt;

        let steps = ((self.accumulator / step) as u32).min(MAX_STEPS);
        self.accumulator -= steps as f32 * step;

        if self.accumulator >= step {
            self.accumulator = 0.0;
        }

        steps
    }
}
