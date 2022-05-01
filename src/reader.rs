use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/// FSG IMG files may be split into multiple parts; this struct allows any number of parts to be read as one stream.
pub struct PartReader {
    parts: VecDeque<File>,
    lengths: Vec<u64>,
    part_index: usize,
    current_pos: usize,
}

impl PartReader {
    pub fn new(parts: Vec<impl AsRef<Path>>) -> Self {
        let parts = parts
            .iter()
            .map(|p| File::open(p).unwrap())
            .collect::<VecDeque<_>>();
        let lengths = parts.iter().map(|f| f.metadata().unwrap().len()).collect();
        Self {
            parts,
            lengths,
            part_index: 0,
            current_pos: 0,
        }
    }
}

impl Read for PartReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut filled = 0;

        loop {
            if self.part_index >= self.parts.len() {
                self.current_pos += filled;
                return Ok(filled);
            }

            filled += self.parts[self.part_index].read(&mut buf[filled..])?;
            if filled == buf.len() {
                self.current_pos += filled;
                return Ok(filled);
            } else {
                self.part_index += 1;
                if self.part_index < self.parts.len() {
                    self.parts[self.part_index].seek(SeekFrom::Start(0))?;
                }
            }
        }
    }
}

impl Seek for PartReader {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(mut n) => {
                self.current_pos = n as usize;

                self.part_index = 0;
                for &len in &self.lengths {
                    if n < len {
                        break;
                    } else {
                        n -= len;
                        self.part_index += 1;
                    }
                }
                self.parts[self.part_index].seek(SeekFrom::Start(n))
            }
            SeekFrom::End(n) => {
                let pos = self.lengths.iter().sum::<u64>() as i64 + n;
                self.seek(SeekFrom::Start(pos as u64))
            }
            SeekFrom::Current(n) => {
                let pos = self.current_pos as i64 + n;
                self.seek(SeekFrom::Start(pos as u64))
            }
        }
    }
}
