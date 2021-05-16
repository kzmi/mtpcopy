use chrono::NaiveDateTime;

use super::file_info::FileInfo;
use super::file_reader::FileReader;

pub trait DestinationFolder {
    fn get_file_info(
        &mut self,
        name: &String,
    ) -> Result<Option<FileInfo>, Box<dyn std::error::Error>>;

    fn create_file(
        &mut self,
        name: &String,
        reader: &mut impl FileReader,
        size: u64,
        created: &Option<NaiveDateTime>,
        modified: &Option<NaiveDateTime>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn open_or_create_folder(
        &mut self,
        name: &String,
    ) -> Result<Box<Self>, Box<dyn std::error::Error>>;

    fn delete_file_or_folder(&mut self, name: &String) -> Result<(), Box<dyn std::error::Error>>;
}
