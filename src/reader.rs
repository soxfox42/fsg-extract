use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read, Seek};

/// FSG IMG files may be split into multiple parts; this struct allows any number of parts to be read as one stream.
pub struct FsgReader {
    parts: VecDeque<File>,
    current_len: u64,
}

impl FsgReader {
    pub fn new(first_file: File) -> Self {
        let current_len = first_file.metadata().unwrap().len();
        Self {
            parts: vec![first_file].into(),
            current_len,
        }
    }

    pub fn add(&mut self, file: File) {
        self.parts.push_back(file)
    }

    fn next_file(&mut self) {
        self.parts.pop_front();
        self.current_len = self.parts.front().unwrap().metadata().unwrap().len();
    }
}

impl Read for FsgReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        while let Some(file) = self.parts.front_mut() {
            match file.read(buf) {
                Ok(0) => self.next_file(),
                r => return r,
            }
        }

        Ok(0)
    }
}

impl Seek for FsgReader {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        if let io::SeekFrom::End(_) = pos {
            while self.parts.len() > 1 {
                self.next_file();
            }

            return self.parts.front_mut().unwrap().seek(pos);
        }

        loop {
            let new_pos = self.parts.front_mut().unwrap().seek(pos)?;
            if new_pos >= self.current_len {
                self.next_file();
            } else {
                return Ok(new_pos);
            }
        }
    }
}
