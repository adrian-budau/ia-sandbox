#![feature(alloc_jemalloc)]
fn main() {
    let mut vec = vec![0u8; 20_000_000];
    for i in 0..vec.len() {
        vec[i] = (i % 8) as u8;
    }

    println!("{}", vec.len());
}
