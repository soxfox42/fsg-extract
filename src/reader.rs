use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read};

/// FSG IMG files may be split into multiple parts; this struct allows any number of parts to be read as one stream.
pub struct FsgReader {
    parts: VecDeque<File>,
}

impl FsgReader {
    pub fn new(first_file: File) -> Self {
        Self {
            parts: vec![first_file].into(),
        }
    }

    pub fn add(&mut self, file: File) {
        self.parts.push_back(file)
    }
}

impl Read for FsgReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        while let Some(file) = self.parts.front_mut() {
            match file.read(buf) {
                Ok(0) => {
                    self.parts.pop_front();
                }
                r => return r,
            }
        }

        Ok(0)
    }
}
