pub const TICK_RATE: u32 = 20;

#[derive(Default)]
pub struct Game {
    tick: u64,
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn fixed_update(&mut self, _dt: f32) {
        self.tick += 1;
    }
}
