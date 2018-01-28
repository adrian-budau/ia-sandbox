use std::thread;
use std::time::{Duration, Instant};

const NUM_THREADS: usize = 4;

fn main() {
    let threads: Vec<_> = (0..NUM_THREADS)
        .map(|_| {
            thread::spawn(move || {
                let now = Instant::now();

                let mut steps = 0;
                loop {
                    steps += 1;
                    if steps < 100000 {
                        continue;
                    }
                    steps = 0;
                    if now.elapsed() > Duration::from_millis((500 / NUM_THREADS) as u64) {
                        break;
                    }
                }
            })
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }
}
