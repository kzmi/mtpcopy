use crate::wpd::resource_stream::ResourceReader;
use std::{fs::File, io::Read};

pub trait FileReader {
    fn next(&mut self) -> Result<Option<&[u8]>, Box<dyn std::error::Error>>;
}

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
    fn next(&mut self) -> Result<Option<&[u8]>, Box<dyn std::error::Error>> {
        let len = self.file.read(self.buf.as_mut_slice())?;
        if len > 0 {
            Ok(Some(&self.buf.as_slice()[..len]))
        } else {
            Ok(None)
        }
    }
}

pub struct DeviceFileReader {
    reader: ResourceReader,
}

impl DeviceFileReader {
    pub fn new(reader: ResourceReader) -> DeviceFileReader {
        DeviceFileReader { reader }
    }
}

impl FileReader for DeviceFileReader {
    fn next(&mut self) -> Result<Option<&[u8]>, Box<dyn std::error::Error>> {
        Ok(self.reader.next()?)
    }
}
