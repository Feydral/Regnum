use std::{
    thread,
    time::{Duration, Instant},
};

const TICK_RATE: u32 = 20;

fn main() {
    let step = Duration::from_secs_f64(1.0 / TICK_RATE as f64);
    let mut next_tick = Instant::now();
    let mut tick: u64 = 0;

    loop {
        tick += 1;

        if tick % TICK_RATE as u64 == 0 {
            println!("tick {tick}");
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
