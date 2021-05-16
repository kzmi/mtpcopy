use std::os::windows::prelude::MetadataExt;
use std::path::Path;

use crate::copy::walkers::{DeviceWalker, LocalWalker, Walker};
use crate::finders::*;
use crate::path::get_path_type;
use crate::path::PathType;
use crate::wpd::device::Device;
use crate::wpd::device::{ContentObjectInfo};
use crate::wpd::manager::Manager;
use crate::wpd::utils::init_com;
use crate::Paths;
use crate::{
    copy::destination_folder::{DeviceDestinationFolder, LocalDestinationFolder},
    path::DeviceStoragePath,
};

pub fn command_copy(paths: &Paths) -> Result<(), Box<dyn std::error::Error>> {
    init_com()?;
    let manager = Manager::get_portable_device_manager()?;

    let src_path = paths.src.as_str();
    let dest_path = paths.dest.as_str();
    let src_path_type = get_path_type(src_path);
    let dest_path_type = get_path_type(dest_path);

    match src_path_type {
        PathType::DeviceStorage => {
            let (device, object_info) =
                find_device_file_or_folder(&manager, src_path, "source", true)?;
            let walker = DeviceWalker::new(&device, object_info);
            do_walk(&manager, &walker, dest_path, dest_path_type)
        }
        PathType::Local => {
            check_local_path(src_path, "source", true)?;
            let walker = LocalWalker::new(src_path);
            do_walk(&manager, &walker, dest_path, dest_path_type)
        }
        PathType::Invalid => {
            return Err("invalid source path.".into());
        }
    }
}

fn do_walk(
    manager: &Manager,
    walker: &impl Walker,
    dest_path: &str,
    dest_path_type: PathType,
) -> Result<(), Box<dyn std::error::Error>> {
    match dest_path_type {
        PathType::DeviceStorage => {
            let (device, object_info) =
                find_device_file_or_folder(manager, dest_path, "destination", false)?;
            let mut dest = DeviceDestinationFolder::new(&device, object_info)?;
            walker.copy(&mut dest)
        }
        PathType::Local => {
            check_local_path(dest_path, "destination", false)?;
            let mut dest = LocalDestinationFolder::new(dest_path.into());
            walker.copy(&mut dest)
        }
        PathType::Invalid => {
            return Err("invalid destination path.".into());
        }
    }
}

fn find_device_file_or_folder<'d>(
    manager: &Manager,
    path: &str,
    subject_type: &str,
    allow_file: bool,
) -> Result<(Device, ContentObjectInfo), Box<dyn std::error::Error>> {
    let storage_path = DeviceStoragePath::from(path)?;

    let device_vec = device_find_devices(manager, Some(&storage_path.device_name))?;
    if device_vec.len() == 0 {
        return Err(format!("the {} device not found.", subject_type).into());
    }
    if device_vec.len() > 1 {
        return Err(format!("cannot determine the {} device.", subject_type).into());
    }

    let device = Device::open(&device_vec[0])?;

    let storage_object_vec =
        device_find_storage_objects(&device, Some(&storage_path.storage_name))?;
    if storage_object_vec.len() == 0 {
        return Err(format!("the {} storage not found.", subject_type).into());
    }
    if storage_object_vec.len() > 1 {
        return Err(format!("cannot determine the {} storage.", subject_type).into());
    }

    let object_opt =
        device_find_file_or_folder(&device, &storage_object_vec[0], &storage_path.path)?;
    if object_opt.is_none() {
        let message = if allow_file {
            format!("the {} file or folder not found.", subject_type)
        } else {
            format!("the {} folder not found.", subject_type)
        };
        return Err(message.into());
    }

    let object_info = object_opt.unwrap();
    if object_info.is_system {
        return Err(format!("the {} path is a system file.", subject_type).into());
    }
    if object_info.is_hidden {
        return Err(format!("the {} path is a hidden file.", subject_type).into());
    }

    if object_info.is_file() {
        if !allow_file {
            return Err(format!("the {} path is a file.", subject_type).into());
        }
    } else if !object_info.is_folder() && !object_info.is_storage() {
        return Err(format!("the {} path is not a folder.", subject_type).into());
    }

    Ok((device, object_info))
}

fn check_local_path(
    path: &str,
    subject_type: &str,
    allow_file: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        let message = if allow_file {
            format!("the {} file or directory not found.", subject_type)
        } else {
            format!("the {} directory not found.", subject_type)
        };
        return Err(message.into());
    }

    let metadata = path_obj.metadata()?;
    let file_attr = metadata.file_attributes();
    let is_hidden = (file_attr & 2/* FILE_ATTRIBUTE_HIDDEN */) != 0;
    let is_system = (file_attr & 4/* FILE_ATTRIBUTE_SYSTEM */) != 0;

    if is_system {
        return Err(format!("the {} path is a system file.", subject_type).into());
    }
    if is_hidden {
        return Err(format!("the {} path is a hidden file.", subject_type).into());
    }

    if path_obj.is_file() {
        if !allow_file {
            return Err(format!("the {} path is a file.", subject_type).into());
        }
    } else if !path_obj.is_dir() {
        return Err(format!("the {} path is not a directory.", subject_type).into());
    }

    Ok(())
}
