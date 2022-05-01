mod reader;

use byteorder::{BigEndian, ReadBytesExt};
use reader::PartReader;
use std::env;
use std::io::Read;
use std::path::Path;
use std::process;

fn open(path: String) -> PartReader {
    let mut paths = vec![path.clone()];
    let path = Path::new(&path);

    if !path.is_file() {
        eprintln!("File not found: {}", path.display());
        process::exit(1);
    }

    if path.extension().map(|s| s == "part0").unwrap_or(false) {
        let stem = path.file_stem().unwrap();
        for i in 1.. {
            let part_path = format!("{}.part{}", stem.to_str().unwrap(), i);
            if !Path::new(&part_path).is_file() {
                break;
            }
            paths.push(part_path);
        }
    }

    PartReader::new(paths)
}

fn main() {
    if env::args().len() != 2 {
        eprintln!("Usage: fsg-extract <file>");
        process::exit(1);
    }

    let path = env::args().nth(1).unwrap();
    let mut reader = open(path);

    let mut header = [0; 16];
    reader.read_exact(&mut header).unwrap();

    if header != "FSG-FILE-SYSTEM\x00".as_bytes() {
        eprintln!("Invalid file header, is this actually an FSG image?");
        process::exit(1);
    }

    reader.read_u32::<BigEndian>().unwrap(); // Unknown
    reader.read_u32::<BigEndian>().unwrap(); // Header Length
    reader.read_u32::<BigEndian>().unwrap(); // Number of Sectors
    reader.read_u32::<BigEndian>().unwrap(); // Sector Map Offset

    let base_offset = reader.read_u32::<BigEndian>().unwrap();
    println!("Base offset: 0x{:08x}", base_offset);

    reader.read_u32::<BigEndian>().unwrap(); // Unknown
    reader.read_u32::<BigEndian>().unwrap(); // Unknown

    let num_files = reader.read_u32::<BigEndian>().unwrap();
    println!("Number of files: {}", num_files);

    reader.read_u32::<BigEndian>().unwrap(); // Unknown
    reader.read_u32::<BigEndian>().unwrap(); // Checksum
}
