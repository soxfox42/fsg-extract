mod reader;

use reader::FsgReader;
use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::process;

fn open(path: &Path) -> FsgReader {
    let mut reader = FsgReader::new(File::open(path).unwrap());

    if path.extension().map(|s| s == "part0").unwrap_or(false) {
        let stem = path.file_stem().unwrap();
        for i in 1u8.. {
            let path = format!("{}.part{}", stem.to_str().unwrap(), i);
            let path = Path::new(&path);
            if !path.is_file() {
                break;
            }
            reader.add(File::open(path).unwrap());
        }
    }

    reader
}

fn main() {
    if env::args().len() != 2 {
        eprintln!("Usage: fsg-extract <file>");
        process::exit(1);
    }

    let path = env::args().nth(1).unwrap();
    let path = Path::new(&path);

    if !path.is_file() {
        eprintln!("File not found: {}", path.display());
        process::exit(1);
    }

    let mut reader = open(path);

    let mut header = [0; 16];
    reader.read_exact(&mut header).unwrap();

    if header != "FSG-FILE-SYSTEM\x00".as_bytes() {
        eprintln!("Invalid file header, is this actually an FSG image?");
        process::exit(1);
    }

    reader.seek(SeekFrom::Current(8)).unwrap();
}
