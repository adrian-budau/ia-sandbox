fn main() {
    let mut vec = vec![0u8; 20_000_000];
    for (i, elem) in vec.iter_mut().enumerate() {
        *elem = (i % 8) as u8;
    }

    println!("{}", vec.len());
}
