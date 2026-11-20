use std::{
    thread,
    time::{Duration, Instant},
};

use regnum_game::{Game, TICK_RATE};

fn main() {
    let mut game = Game::new();

    let step = Duration::from_secs_f64(1.0 / TICK_RATE as f64);
    let mut next_tick = Instant::now();

    loop {
        game.fixed_update(step.as_secs_f32());

        if game.tick() % TICK_RATE as u64 == 0 {
            println!("tick {}", game.tick());
        }

        next_tick += step;
        let now = Instant::now();
        if next_tick > now {
            thread::sleep(next_tick - now);
        } else {
            next_tick = now;
        }
    }
}
