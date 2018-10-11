extern crate nes;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("Usage: nes <rom-path>")
    }

    let path = &args[1];
    println!("{:?}", path);
}