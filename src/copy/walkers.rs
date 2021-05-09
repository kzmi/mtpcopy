use std::path::PathBuf;
use std::{fs::File, os::windows::prelude::MetadataExt};

use chrono::{DateTime, Local, NaiveDateTime};

use crate::wpd::device::{ContentObjectInfo, Device};

use super::{destination::DestinationFolder, file_info::FileInfo};
use super::file_reader::{DeviceFileReader, LocalFileReader};

pub trait Walker {
    fn copy(&self, dest: &mut impl DestinationFolder) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct DeviceWalker<'d> {
    device: &'d Device,
    source_root_object_info: ContentObjectInfo,
}

impl<'d> DeviceWalker<'d> {
    pub fn new(device: &'d Device, source_root_object_info: ContentObjectInfo) -> DeviceWalker<'d> {
        DeviceWalker {
            device,
            source_root_object_info,
        }
    }
}

impl<'d> Walker for DeviceWalker<'d> {
    fn copy(&self, dest: &mut impl DestinationFolder) -> Result<(), Box<dyn std::error::Error>> {
        device_walker_do_copy(self.device, dest, &self.source_root_object_info)
    }
}

fn device_walker_do_copy(
    device: &Device,
    dest: &mut impl DestinationFolder,
    target_object_info: &ContentObjectInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    if target_object_info.is_system || target_object_info.is_hidden {
        return Ok(());
    }

    if target_object_info.is_file() {
        let src_file_info = FileInfo::from_content_object_info(&target_object_info)?;
        let dest_file_info = dest.get_file_info(&target_object_info.name)?;

        if let Some(dest_file_info_ref) = dest_file_info.as_ref() {
            if can_skip_copying(&src_file_info, dest_file_info_ref) {
                return Ok(());
            }
        }

        if dest_file_info.is_some() {
            dest.delete_file_or_folder(&target_object_info.name)?;
        }

        let res_reader = device.get_resoure(&target_object_info.content_object)?;
        let mut dev_reader = DeviceFileReader::new(res_reader);
        return dest.create_file(
            &target_object_info.name,
            &mut dev_reader,
            src_file_info.data_size,
            &target_object_info.time_created,
            &target_object_info.time_modified,
        );
    }

    if !target_object_info.is_folder() {
        return Ok(());
    }

    let mut new_dest = dest.open_or_create_folder(&target_object_info.name)?;

    let mut iter = device.get_object_iterator(&target_object_info.content_object)?;
    while let Some(content_object) = iter.next()? {
        let content_object_info = device.get_object_info(content_object)?;
        device_walker_do_copy(device, new_dest.as_mut(), &content_object_info)?;
    }
    Ok(())
}

pub struct LocalWalker {
    path: PathBuf,
}

impl LocalWalker {
    pub fn new(path: &str) -> LocalWalker {
        LocalWalker { path: PathBuf::from(path) }
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
        return dest.create_file(
            &file_name,
            &mut reader,
            src_file_info.data_size,
            &Some(created_date_time.naive_local()),
            &Some(modified_date_time.naive_local()),
        );
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

fn can_skip_copying(src_file_info: &FileInfo, dest_file_info: &FileInfo) -> bool {
    if let Some(src_time) = get_file_time(src_file_info) {
        if let Some(dest_time) = get_file_time(dest_file_info) {
            return src_time <= dest_time;
        }
    }
    false
}

fn get_file_time(file_info: &FileInfo) -> Option<NaiveDateTime> {
    if let Some(time_created) = file_info.time_created {
        if let Some(time_modified) = file_info.time_modified {
            Some(std::cmp::max(time_created, time_modified))
        } else {
            Some(time_created)
        }
    } else {
        if let Some(time_modified) = file_info.time_modified {
            Some(time_modified)
        } else {
            None
        }
    }
}
