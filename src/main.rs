mod reader;

use byteorder::{BigEndian, ReadBytesExt};
use reader::FsgReader;
use std::env;
use std::fs::File;
use std::io::Read;
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

    reader.read_u32::<BigEndian>().unwrap();
    reader.read_u32::<BigEndian>().unwrap();
    
    let num_sectors = reader.read_u32::<BigEndian>().unwrap();
    println!("Num sectors: {}", num_sectors);
    
    reader.read_u32::<BigEndian>().unwrap();
    
    let base_offset = reader.read_u32::<BigEndian>().unwrap();
    println!("Base offset: {}", base_offset);

    reader.read_u32::<BigEndian>().unwrap();
    reader.read_u32::<BigEndian>().unwrap();
    
    let num_files = reader.read_u32::<BigEndian>().unwrap();
    println!("Number of files: {}", num_files);
    
    reader.read_u32::<BigEndian>().unwrap();
    reader.read_u32::<BigEndian>().unwrap();
}
