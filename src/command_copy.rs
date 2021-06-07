use std::path::{Path, PathBuf};

use crate::copy::copy_processor::CopyProcessor;
use crate::copy::destination_folder::DestinationFolder;
use crate::copy::device_copy_processor::DeviceCopyProcessor;
use crate::copy::device_destination_folder::DeviceDestinationFolder;
use crate::copy::file_info::FileInfo;
use crate::copy::local_copy_processor::LocalCopyProcessor;
use crate::copy::local_destination_folder::LocalDestinationFolder;
use crate::finders::*;
use crate::path::get_path_type;
use crate::path::DeviceStoragePath;
use crate::path::PathType;
use crate::path::SEPARATORS;
use crate::path::WILDCARD_CHARACTERS;
use crate::wpd::device::ContentObjectInfo;
use crate::wpd::device::Device;
use crate::wpd::manager::DeviceInfo;
use crate::wpd::manager::Manager;
use crate::wpd::utils::init_com;
use crate::Paths;

pub fn command_copy(paths: &Paths, recursive: bool) -> Result<(), Box<dyn std::error::Error>> {
    log::trace!("command_copy paths={:?}", paths);
    init_com()?;
    let manager = Manager::get_portable_device_manager()?;

    let src_path = paths.src.as_str();
    let dest_path = paths.dest.as_str();

    let src_path_type = get_path_type(src_path);
    let dest_path_type = get_path_type(dest_path);

    if has_wildcard(src_path, src_path_type)? {
        return Err("wildcard characters in the source path are not allowed.".into());
    }
    if has_wildcard(dest_path, dest_path_type)? {
        return Err("wildcard characters in the destination path are not allowed.".into());
    }

    let dest_inspection = inspect_path(&manager, dest_path, dest_path_type)?;
    log::trace!("dest_inspection = {:?}", &dest_inspection);

    let dest_base_path: &str;
    let dest_name: Option<&str>;
    match dest_inspection.target_status {
        TargetStatus::NotExist => match dest_inspection.parent_status {
            TargetStatus::Folder => {
                dest_base_path = dest_inspection.parent_path.as_ref().unwrap();
                dest_name = dest_inspection
                    .target_name
                    .as_ref()
                    .and_then(|v| Some(v as &str));
            }
            _ => {
                return Err("cannot create the destination path.".into());
            }
        },
        TargetStatus::Hidden => {
            return Err("destination path is a hidden file or folder.".into());
        }
        TargetStatus::File => {
            dest_base_path = src_path;
            dest_name = None;
        }
        TargetStatus::Folder => {
            dest_base_path = src_path;
            dest_name = None;
        }
    }

    match dest_path_type {
        PathType::DeviceStorage => {
            let storage_path = DeviceStoragePath::from(dest_base_path)?;

            if let Some((_device_info, device, object_info)) =
                find_device_file_or_folder(&manager, &storage_path)?
            {
                let mut destination_folder = DeviceDestinationFolder::new(&device, object_info)?;
                do_copy(
                    &manager,
                    src_path,
                    src_path_type,
                    &mut destination_folder,
                    dest_name,
                    recursive,
                )
            } else {
                return Err(format!("filed to open folder: {}", dest_base_path).into());
            }
        }
        PathType::Local => {
            let mut destination_folder = LocalDestinationFolder::new(PathBuf::from(dest_base_path));
            do_copy(
                &manager,
                src_path,
                src_path_type,
                &mut destination_folder,
                dest_name,
                recursive,
            )
        }
        PathType::Invalid => Err("invalid destination path.".into()),
    }
}

fn has_wildcard(path: &str, path_type: PathType) -> Result<bool, Box<dyn std::error::Error>> {
    let storage_path: DeviceStoragePath;
    let path_to_check: &str;
    match path_type {
        PathType::DeviceStorage => {
            storage_path = DeviceStoragePath::from(path)?;
            // device name and storage name can contain wildcard characters
            path_to_check = &storage_path.path;
        }
        PathType::Local => {
            path_to_check = path;
        }
        _ => {
            return Ok(false);
        }
    }
    for p in path_to_check.split(SEPARATORS) {
        if p.contains(WILDCARD_CHARACTERS) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn do_copy<D>(
    manager: &Manager,
    src_path: &str,
    src_path_type: PathType,
    destination_folder: &mut D,
    dest_name: Option<&str>,
    recursive: bool,
) -> Result<(), Box<dyn std::error::Error>>
where
    D: DestinationFolder,
{
    match src_path_type {
        PathType::DeviceStorage => {
            let storage_path = DeviceStoragePath::from(src_path)?;

            if let Some((_device_info, device, content_object)) =
                find_device_file_or_folder(manager, &storage_path)?
            {
                let processor = DeviceCopyProcessor::new(&device, content_object.clone());
                let real_dest_name = dest_name.unwrap_or(&content_object.name);
                processor.copy_as(real_dest_name, destination_folder, recursive)
            } else {
                Err("failed to open source path.".into())
            }
        }
        PathType::Local => {
            let src_path_buf;
            let real_dest_name;
            match dest_name {
                Some(name) => {
                    real_dest_name = name;
                }
                None => {
                    src_path_buf = PathBuf::from(src_path);
                    match src_path_buf.file_name() {
                        Some(p) => {
                            real_dest_name = p.to_str().unwrap();
                        }
                        None => {
                            return Err("cannot copy the root directory.".into());
                        }
                    }
                }
            }
            let processor = LocalCopyProcessor::new(src_path);
            processor.copy_as(real_dest_name, destination_folder, recursive)
        }
        PathType::Invalid => {
            return Err("invalid source path.".into());
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum TargetStatus {
    NotExist,
    Hidden,
    File,
    Folder,
}

#[derive(Debug)]
struct TargetInspectionResult {
    target_name: Option<String>,
    target_status: TargetStatus,
    parent_status: TargetStatus,
    parent_path: Option<String>,
}

fn inspect_path(
    manager: &Manager,
    path: &str,
    path_type: PathType,
) -> Result<TargetInspectionResult, Box<dyn std::error::Error>> {
    match path_type {
        PathType::DeviceStorage => inspect_device_path(manager, path),
        PathType::Local => inspect_local_path(path),
        PathType::Invalid => Err(format!("invalid path: {}", path).into()),
    }
}

fn inspect_local_path(path: &str) -> Result<TargetInspectionResult, Box<dyn std::error::Error>> {
    let path_obj = Path::new(path);
    let target_status = inspect_local_path_status(path_obj)?;
    let target_name: Option<String> = path_obj
        .file_name()
        .and_then(|s| Some(String::from(s.to_str().unwrap())));
    if target_status != TargetStatus::NotExist && target_name.is_none() {
        return Err("failed to get the file name of the destination path.".into());
    }

    let parent_status: TargetStatus;
    let parent_path: Option<String>;
    match path_obj.parent() {
        Some(p) => {
            parent_path = Some(String::from(p.to_str().unwrap()));
            parent_status = inspect_local_path_status(p)?;
        }
        None => {
            parent_status = TargetStatus::NotExist;
            parent_path = None;
        }
    }

    Ok(TargetInspectionResult {
        target_name,
        target_status,
        parent_status,
        parent_path,
    })
}

fn inspect_local_path_status(path_obj: &Path) -> Result<TargetStatus, Box<dyn std::error::Error>> {
    if !path_obj.exists() {
        Ok(TargetStatus::NotExist)
    } else {
        let file_info = FileInfo::from_metadata(&path_obj.metadata()?, "")?;
        if file_info.is_hidden || file_info.is_system {
            Ok(TargetStatus::Hidden)
        } else if file_info.is_folder {
            Ok(TargetStatus::Folder)
        } else {
            Ok(TargetStatus::File)
        }
    }
}

fn inspect_device_path(
    manager: &Manager,
    path: &str,
) -> Result<TargetInspectionResult, Box<dyn std::error::Error>> {
    let storage_path = DeviceStoragePath::from(path)?;
    let target_name: Option<String> = storage_path.file_name().and_then(|v| Some(String::from(v)));
    let target_status = inspect_device_path_status(manager, &storage_path)?;

    let parent_status: TargetStatus;
    let parent_path: Option<String>;
    match storage_path.parent() {
        Some(p) => {
            parent_status = inspect_device_path_status(manager, &p)?;
            parent_path = Some(p.full_path());
        }
        None => {
            parent_status = TargetStatus::NotExist;
            parent_path = None;
        }
    }

    Ok(TargetInspectionResult {
        target_name,
        target_status,
        parent_status,
        parent_path,
    })
}

fn inspect_device_path_status(
    manager: &Manager,
    storage_path: &DeviceStoragePath,
) -> Result<TargetStatus, Box<dyn std::error::Error>> {
    match find_device_file_or_folder(manager, storage_path)? {
        Some((_, _, content_object_info)) => {
            let file_info = FileInfo::from_content_object_info(&content_object_info)?;
            if file_info.is_hidden || file_info.is_system {
                Ok(TargetStatus::Hidden)
            } else if file_info.is_folder {
                Ok(TargetStatus::Folder)
            } else {
                Ok(TargetStatus::File)
            }
        }
        None => Ok(TargetStatus::NotExist),
    }
}

fn find_device_file_or_folder(
    manager: &Manager,
    storage_path: &DeviceStoragePath,
) -> Result<Option<(DeviceInfo, Device, ContentObjectInfo)>, Box<dyn std::error::Error>> {
    log::trace!("find_device_file_or_folder");
    if let Some((device_info, device, storage_object)) = find_device_storage(manager, storage_path)?
    {
        log::trace!("find_device_file_or_folder: storage found");
        match device_find_file_or_folder(
            &device,
            &device_info,
            &storage_object,
            &storage_path.path,
        )? {
            Some((content_object_info, _)) => {
                log::trace!("find_device_file_or_folder: file/folder object found");
                Ok(Some((device_info, device, content_object_info)))
            },
            None => {
                log::trace!("find_device_file_or_folder: no object found");
                Ok(None)
            },
        }
    } else {
        log::trace!("find_device_file_or_folder: storage was not found");
        Ok(None)
    }
}

fn find_device_storage(
    manager: &Manager,
    storage_path: &DeviceStoragePath,
) -> Result<Option<(DeviceInfo, Device, ContentObjectInfo)>, Box<dyn std::error::Error>> {
    log::trace!("find_device_storage: storage_path = {:?}", storage_path);
    let mut device_vec = device_find_devices(manager, Some(&storage_path.device_name))?;
    if device_vec.len() == 0 {
        return Err(format!("device was not found: {}", &storage_path.device_name).into());
    }
    if device_vec.len() > 1 {
        return Err(format!(
            "multiple devices were matched: {}",
            &storage_path.device_name
        )
        .into());
    }

    let device_info = device_vec.pop().unwrap();

    let device = Device::open(&device_info)?;

    let mut storage_object_vec =
        device_find_storage_objects(&device, Some(&storage_path.storage_name))?;
    if storage_object_vec.len() == 0 {
        return Err(format!(
            "storage was not found: {}:{}",
            &storage_path.device_name, &storage_path.storage_name
        )
        .into());
    }
    if storage_object_vec.len() > 1 {
        return Err(format!(
            "multiple storages were matched: {}:{}",
            &storage_path.device_name, &storage_path.storage_name
        )
        .into());
    }

    let storage_object = storage_object_vec.pop().unwrap();

    log::trace!("find_device_storage: found {:?} {:?}", &device_info, &storage_object);
    Ok(Some((device_info, device, storage_object)))
}
