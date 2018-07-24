use std::thread;
use std::time::Duration;

const NUM_THREADS: usize = 4;
fn main() {
    let threads: Vec<_> = (0..NUM_THREADS)
        .map(|_| {
            thread::spawn(|| {
                thread::sleep(Duration::from_secs(1));
            })
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }
}
