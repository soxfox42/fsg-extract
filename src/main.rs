use std::{env, process};

fn main() {
    if env::args().len() != 2 {
        eprintln!("Usage: fsg-extract <file>");
        process::exit(1);
    }

    println!("Extracting {}", env::args().nth(1).unwrap());
}
