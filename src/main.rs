mod reader;

use byteorder::{BigEndian, ReadBytesExt};
use reader::PartReader;
use std::collections::HashMap;
use std::env;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::process;

#[derive(Debug)]
struct FileNode {
    offset: u32,
    size: u32,
}

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

    let mut table_offsets = HashMap::new();
    for _ in 0..num_files {
        let file_hash = reader.read_u32::<BigEndian>().unwrap();
        reader.read_u8().unwrap();
        let table_offset = reader.read_u24::<BigEndian>().unwrap();
        table_offsets.insert(file_hash, table_offset);
    }

    let mut file_nodes = HashMap::new();
    for (file_hash, table_offset) in table_offsets {
        reader.seek(SeekFrom::Start(table_offset as u64)).unwrap();
        let offset = reader.read_u32::<BigEndian>().unwrap();
        let offset = base_offset + (offset << 10);
        let size = reader.read_u32::<BigEndian>().unwrap();
        file_nodes.insert(file_hash, FileNode {offset, size});
    }

    let (hash, node) = file_nodes.iter().next().unwrap();
    println!("0x{:08x}: {:?}", hash, node);
}
