#![feature(asm)]
extern crate libc;

use std::env;
use std::process;
use std::ptr;

fn main() {
    unsafe { libc::signal(libc::SIGFPE, libc::SIG_DFL) };
    let signal = env::args().skip(1).next().unwrap().trim().parse().unwrap();
    process::exit(match signal {
        8 => unsafe {
            asm!("xor %ebx, %ebx\n\
                   mov $$0x200, %eax\n\
                   div %ebx"
                ::: "eax", "ebx", "edx");
            1
        },
        11 => unsafe { libc::strcmp(ptr::null(), ptr::null()) },
        _ => 1
    });
}
