use std::fs::File;
use std::io::Read;

use super::file_reader::FileReader;

pub struct LocalFileReader {
    file: File,
    buf: Vec<u8>,
}

impl LocalFileReader {
    pub fn new(file: File) -> LocalFileReader {
        let mut buf = Vec::<u8>::new();
        buf.resize(32768, 0);
        LocalFileReader { file, buf }
    }
}

impl FileReader for LocalFileReader {
    fn get_optimized_buffer_size(&self) -> u32 {
        self.buf.len() as u32
    }

    fn next(&mut self, max_size: u32) -> Result<Option<&[u8]>, Box<dyn std::error::Error>> {
        self.buf.resize(max_size as usize, 0);
        let len = self.file.read(self.buf.as_mut_slice())?;
        if len > 0 {
            Ok(Some(&self.buf.as_slice()[..len]))
        } else {
            Ok(None)
        }
    }
}
