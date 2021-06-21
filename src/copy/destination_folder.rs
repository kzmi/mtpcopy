use chrono::NaiveDateTime;

use super::file_info::FileInfo;
use super::file_reader::FileReader;

pub trait DestinationFolder {
    fn get_file_info(&mut self, name: &str)
        -> Result<Option<FileInfo>, Box<dyn std::error::Error>>;

    fn create_file(
        &mut self,
        name: &str,
        reader: &mut impl FileReader,
        size: u64,
        created: &Option<NaiveDateTime>,
        modified: &Option<NaiveDateTime>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn open_or_create_folder<FBeforeOpen, FBeforeCreate>(
        &mut self,
        name: &str,
        before_open: FBeforeOpen,
        before_create: FBeforeCreate,
    ) -> Result<Box<Self>, Box<dyn std::error::Error>>
    where
        FBeforeOpen: FnOnce(&str),
        FBeforeCreate: FnOnce(&str);

    fn delete_file_or_folder(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>>;
}
