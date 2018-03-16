extern crate libc;

use std::mem;
use std::thread;

const NUM_THREADS: usize = 4;

fn main() {
    let threads: Vec<_> = (0..NUM_THREADS)
        .map(|_| {
            thread::spawn(move || {
                let mut steps = 0;
                loop {
                    steps += 1;
                    if steps < 100000 {
                        continue;
                    }
                    steps = 0;
                    let mut usage: libc::rusage = unsafe { mem::zeroed() };
                    unsafe {
                        libc::getrusage(libc::RUSAGE_THREAD, &mut usage);
                    }
                    let us = usage.ru_utime.tv_sec * 1000000 + usage.ru_utime.tv_usec;
                    if us >= 500000 / NUM_THREADS as i64 {
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
