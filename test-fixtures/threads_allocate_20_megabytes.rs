use std::sync::mpsc;
use std::thread::Builder;

const NUM_THREADS: usize = 4;

fn main() {
    let (tx_checkpoint, rx_checkpoint) = mpsc::channel();
    let (tx, rx) = mpsc::sync_channel(0);

    let threads: Vec<_> = (0..NUM_THREADS)
        .map(|index| {
            let sender_checkpoint = tx_checkpoint.clone();
            let sender = tx.clone();

            Builder::new()
                .stack_size(32 * 1024)
                .spawn(move || {
                    let mut vec = vec![0u8; 20_000_000 / NUM_THREADS];
                    for i in 0..vec.len() {
                        vec[i] = ((i + index) % 8) as u8;
                    }
                    sender_checkpoint.send(()).unwrap();
                    sender.send(()).unwrap();
                    vec.len()
                })
                .unwrap()
        })
        .collect();

    for _ in 0..NUM_THREADS {
        rx_checkpoint.recv().unwrap();
    }

    for _ in 0..NUM_THREADS {
        rx.recv().unwrap();
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
