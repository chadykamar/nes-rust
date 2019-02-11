use rom::Rom;

use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("Usage: nes <rom-path>")
    }

    let path = &args[1];
    println!("{:?}", path);

    let rom = Rom::load(&mut File::open(&Path::new(path)).unwrap()).unwrap();

    start(rom);
}
