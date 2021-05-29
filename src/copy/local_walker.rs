use std::path::PathBuf;
use std::{fs::File, os::windows::prelude::MetadataExt};

use chrono::{DateTime, Local};

use super::destination_folder::DestinationFolder;
use super::file_info::FileInfo;
use super::local_file_reader::LocalFileReader;

use super::walker::{Walker, can_skip_copying, report_copying_end, report_copying_start};

pub struct LocalWalker {
    path: PathBuf,
}

impl LocalWalker {
    pub fn new(path: &str) -> LocalWalker {
        LocalWalker {
            path: PathBuf::from(path),
        }
    }
}

impl Walker for LocalWalker {
    fn copy(&self, dest: &mut impl DestinationFolder) -> Result<(), Box<dyn std::error::Error>> {
        local_walker_do_copy(&self.path, dest)
    }
}

fn local_walker_do_copy(
    path: &PathBuf,
    dest: &mut impl DestinationFolder,
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = path.metadata()?;
    let file_attr = metadata.file_attributes();
    let is_hidden = (file_attr & 2/* FILE_ATTRIBUTE_HIDDEN */) != 0;
    let is_system = (file_attr & 4/* FILE_ATTRIBUTE_SYSTEM */) != 0;

    if is_hidden || is_system {
        return Ok(());
    }

    let file_name = String::from(path.file_name().unwrap().to_str().unwrap());

    if metadata.is_file() {
        let src_file_info = FileInfo::from_metadata(&metadata, &file_name)?;
        let dest_file_info = dest.get_file_info(&file_name)?;

        if let Some(dest_file_info_ref) = dest_file_info.as_ref() {
            if can_skip_copying(&src_file_info, dest_file_info_ref) {
                return Ok(());
            }
        }

        if dest_file_info.is_some() {
            dest.delete_file_or_folder(&file_name)?;
        }

        let file = File::open(path)?;
        let mut reader = LocalFileReader::new(file);
        let created_date_time = DateTime::<Local>::from(metadata.created()?);
        let modified_date_time = DateTime::<Local>::from(metadata.modified()?);
        report_copying_start(&src_file_info);
        dest.create_file(
            &file_name,
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

    let mut new_dest = dest.open_or_create_folder(&file_name)?;

    for result in std::fs::read_dir(path)? {
        let entry = result?;
        let new_path = entry.path();
        local_walker_do_copy(&new_path, new_dest.as_mut())?;
    }
    Ok(())
}
