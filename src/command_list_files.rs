use chrono::NaiveDateTime;

use crate::wpd::device::ContentObjectInfo;
use crate::wpd::device::Device;
use crate::wpd::manager::Manager;
use crate::wpd::utils::init_com;

use crate::finders::*;
use crate::path::DeviceStoragePath;

pub fn command_list_files(
    path: String,
    recursive: bool,
    verbose: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    log::trace!("COMMAND list-files");

    let storage_path = DeviceStoragePath::from(&path)?;

    init_com()?;
    let manager = Manager::get_portable_device_manager()?;
    let mut device_info_vec = device_find_devices(&manager, Some(&storage_path.device_name))?;
    if device_info_vec.len() == 0 {
        return Err("No device matched.".into());
    }
    if device_info_vec.len() > 1 {
        return Err(format!(
            "Multiple devices are matched. : {}",
            &storage_path.device_name
        )
        .into());
    }

    let device_info = device_info_vec.pop().unwrap();
    let device = Device::open(&device_info)?;

    let mut storage_object_vec =
        device_find_storage_objects(&device, Some(&storage_path.storage_name))?;
    if storage_object_vec.len() == 0 {
        return Err(format!("No storage matched on '{}'", &device_info.name).into());
    }
    if storage_object_vec.len() > 1 {
        return Err(format!(
            "Multiple storages are matched on '{}' : {}",
            &device_info.name, &storage_path.storage_name
        )
        .into());
    }

    let storage_object = storage_object_vec.pop().unwrap();

    let base_object_opt =
        device_find_file_or_folder(&device, &storage_object, &storage_path.path)?;
    match base_object_opt {
        None => Err(format!(
            "No file or folder matched.: device: '{}', storage: '{}', path: '{}'",
            &device_info.name, &storage_object.name, &storage_path.path
        )
        .into()),

        Some(base_object) => {
            if base_object.is_storage() || base_object.is_folder() {
                list_folder(
                    &device,
                    recursive,
                    verbose,
                    &base_object,
                    1,
                    &"".to_string(),
                )
            } else if base_object.is_file() {
                show_file_or_folder(
                    &device,
                    recursive,
                    verbose,
                    &base_object,
                    1,
                    &"".to_string(),
                )
            } else {
                Ok(())
            }
        }
    }
}

fn show_file_or_folder(
    device: &Device,
    recursive: bool,
    verbose: u32,
    object_info: &ContentObjectInfo,
    level: u32,
    indent: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}{}", indent, &object_info.name);
    if verbose > 0 {
        println!(
            "{}  Created : {}",
            indent,
            format_datetime_opt(&object_info.time_created)
        );
        println!(
            "{}  Modified: {}",
            indent,
            format_datetime_opt(&object_info.time_modified)
        );
        println!("");
    }

    if recursive && object_info.is_folder() {
        let next_level = level + 1;
        let next_indent = indent.clone() + "  ";
        list_folder(
            device,
            recursive,
            verbose,
            object_info,
            next_level,
            &next_indent,
        )?;
    }
    Ok(())
}

fn list_folder(
    device: &Device,
    recursive: bool,
    verbose: u32,
    folder_object_info: &ContentObjectInfo,
    level: u32,
    indent: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut iter = device.get_object_iterator(&folder_object_info.content_object)?;
    while let Some(obj) = iter.next()? {
        let info = device.get_object_info(obj)?;
        if !info.is_functional_object() {
            show_file_or_folder(device, recursive, verbose, &info, level, &indent)?;
        }
    }
    Ok(())
}

fn format_datetime_opt(opt: &Option<NaiveDateTime>) -> String {
    match &opt {
        None => "(not provided)".to_string(),
        Some(dt) => dt.to_string(),
    }
}
