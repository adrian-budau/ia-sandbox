extern crate libc;

fn main() {
    unsafe { libc::sleep(2) };
}
