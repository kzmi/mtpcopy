use crate::wpd::resource_stream::ResourceReader;

use super::file_reader::FileReader;

pub struct DeviceFileReader {
    reader: ResourceReader,
}

impl DeviceFileReader {
    pub fn new(reader: ResourceReader) -> DeviceFileReader {
        DeviceFileReader { reader }
    }
}

impl FileReader for DeviceFileReader {
    fn get_optimized_buffer_size(&self) -> u32 {
        self.reader.get_optimized_buffer_size()
    }

    fn next(&mut self, max_size: u32) -> Result<Option<&[u8]>, Box<dyn std::error::Error>> {
        Ok(self.reader.next(max_size)?)
    }
}
