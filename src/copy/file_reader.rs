pub trait FileReader {
    fn get_optimized_buffer_size(&self) -> u32;
    fn next(&mut self, max_size: u32) -> Result<Option<&[u8]>, Box<dyn std::error::Error>>;
}
