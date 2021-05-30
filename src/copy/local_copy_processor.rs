use std::path::PathBuf;
use std::{fs::File, os::windows::prelude::MetadataExt};

use chrono::{DateTime, Local};

use super::destination_folder::DestinationFolder;
use super::file_info::FileInfo;
use super::local_file_reader::LocalFileReader;

use super::copy_processor::{
    can_skip_copying, report_copying_end, report_copying_start, CopyProcessor,
};

pub struct LocalCopyProcessor {
    path: PathBuf,
}

impl LocalCopyProcessor {
    pub fn new(path: &str) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }
}

impl CopyProcessor for LocalCopyProcessor {
    fn copy(&self, dest: &mut impl DestinationFolder) -> Result<(), Box<dyn std::error::Error>> {
        let dest_file_name = self.path.file_name().unwrap().to_str().unwrap();
        copy_hierarchy(&self.path, dest, dest_file_name)
    }

    fn copy_as(
        &self,
        name: &str,
        dest: &mut impl DestinationFolder,
    ) -> Result<(), Box<dyn std::error::Error>> {
        copy_hierarchy(&self.path, dest, name)
    }
}

fn copy_hierarchy(
    path: &PathBuf,
    dest: &mut impl DestinationFolder,
    dest_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = path.metadata()?;
    let file_attr = metadata.file_attributes();
    let is_hidden = (file_attr & 2/* FILE_ATTRIBUTE_HIDDEN */) != 0;
    let is_system = (file_attr & 4/* FILE_ATTRIBUTE_SYSTEM */) != 0;

    if is_hidden || is_system {
        return Ok(());
    }

    let src_file_name = path.file_name().unwrap().to_str().unwrap();

    if metadata.is_file() {
        let src_file_info = FileInfo::from_metadata(&metadata, src_file_name)?;
        let dest_file_info = dest.get_file_info(dest_name)?;

        if let Some(dest_file_info_ref) = dest_file_info.as_ref() {
            if can_skip_copying(&src_file_info, dest_file_info_ref) {
                return Ok(());
            }
        }

        if dest_file_info.is_some() {
            dest.delete_file_or_folder(dest_name)?;
        }

        let file = File::open(path)?;
        let mut reader = LocalFileReader::new(file);
        let created_date_time = DateTime::<Local>::from(metadata.created()?);
        let modified_date_time = DateTime::<Local>::from(metadata.modified()?);
        report_copying_start(&src_file_info);
        dest.create_file(
            dest_name,
            &mut reader,
            src_file_info.data_size,
            &Some(created_date_time.naive_local()),
            &Some(modified_date_time.naive_local()),
        )?;
        report_copying_end();
        return Ok(());
    }

    if !metadata.is_dir() {
        return Ok(());
    }

    let mut new_dest = dest.open_or_create_folder(dest_name)?;

    for result in std::fs::read_dir(path)? {
        let entry = result?;
        let new_path = entry.path();
        let dest_file_name = new_path.file_name().unwrap().to_str().unwrap();
        copy_hierarchy(&new_path, new_dest.as_mut(), dest_file_name)?;
    }
    Ok(())
}
