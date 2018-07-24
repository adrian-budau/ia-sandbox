extern crate libc;

use std::mem;

fn main() {
    let mut steps = 0;
    loop {
        steps += 1;
        if steps < 100000 {
            continue;
        }
        steps = 0;
        let mut usage: libc::timespec = unsafe { mem::zeroed() };
        unsafe {
            libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut usage);
        }
        let us = i64::from(usage.tv_sec) * 1_000_000_000 + i64::from(usage.tv_nsec);
        if us >= 500_000_000 {
            break;
        }
    }
}
