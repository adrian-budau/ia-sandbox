use std::thread::{self, Builder};
use std::time::Duration;

const NUM_THREADS: usize = 4;

fn main() {
    let threads: Vec<_> = (0..NUM_THREADS)
        .map(|index| {
            Builder::new()
                .stack_size(32 * 1024)
                .spawn(move || {
                    let mut vec = vec![0u8; 20_000_000 / NUM_THREADS];
                    for i in 0..vec.len() {
                        vec[i] = ((i + index) % 8) as u8;
                    }
                    thread::sleep(Duration::from_millis(100));
                    vec.len()
                })
                .unwrap()
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }
}
