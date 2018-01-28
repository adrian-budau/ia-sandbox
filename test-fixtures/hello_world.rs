use std::io::Write;

fn main() {
    println!("Hello World!");
    writeln!(&mut ::std::io::stderr(), "Hello stderr!").unwrap();
}
