use std::time::{Duration, Instant};

fn main() {
    let now = Instant::now();

    let mut steps = 0;
    loop {
        steps += 1;
        if steps < 100000 {
            continue;
        }
        steps = 0;
        if now.elapsed() > Duration::from_millis(500) {
            break;
        }
    }
}
