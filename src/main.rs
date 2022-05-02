mod reader;

use byteorder::{BigEndian, ReadBytesExt};
use indicatif::ProgressBar;
use reader::PartReader;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::process;

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

fn hash(data: &str) -> u32 {
    let data = data.to_uppercase();
    let mut hash: u32 = 2166136261;

    for &byte in data.as_bytes() {
        hash = hash.overflowing_mul(1677619).0;
        hash ^= byte as u32;
    }

    hash
}

fn extend_path(path: &str, filename: &str) -> String {
    if path == "" {
        filename[1..].into()
    } else {
        format!("{}/{}", path, &filename[1..])
    }
}

fn read_directory(
    reader: &mut PartReader,
    path: &str,
    offset: u32,
    nodes: &HashMap<u32, FileNode>,
    progress: &ProgressBar,
) -> std::io::Result<()> {
    progress.inc(1);

    reader.seek(SeekFrom::Start(offset as u64))?;

    let mut filenames = Vec::new();
    loop {
        let mut next_name = String::new();
        while let Ok(byte) = reader.read_u8() {
            if byte == 0 {
                break;
            }
            next_name.push(byte as char);
        }
        if next_name == "" {
            break;
        } else {
            filenames.push(next_name);
        }
    }

    for filename in filenames {
        let path = extend_path(path, &filename);
        if let Some(node) = nodes.get(&hash(&path)) {
            if filename.starts_with('D') {
                read_directory(reader, &path, node.offset, nodes, progress)?;
            } else {
                progress.inc(1);

                let out_path = Path::new("out").join(path);
                std::fs::create_dir_all(out_path.parent().unwrap())?;
                let mut file = File::create(out_path)?;

                reader.seek(SeekFrom::Start(node.offset as u64))?;
                std::io::copy(&mut reader.take(node.size as u64), &mut file)?;
            }
        }
    }

    Ok(())
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

    reader.read_u32::<BigEndian>().unwrap(); // Unknown
    reader.read_u32::<BigEndian>().unwrap(); // Unknown

    let num_files = reader.read_u32::<BigEndian>().unwrap();

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
        file_nodes.insert(file_hash, FileNode { offset, size });
    }

    println!("Extracting {} files", num_files);
    let progress = ProgressBar::new(num_files as u64);
    if let Err(e) = read_directory(&mut reader, "", base_offset, &file_nodes, &progress) {
        eprintln!("Unexpected extraction error: {}", e);
        process::exit(1);
    }
}
