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
    let device_info_vec = device_find_devices(&manager, Some(&storage_path.device_name))?;

    if device_info_vec.len() == 0 {
        return Err("No device matched.".into());
    }

    for device_info in device_info_vec {
        let device = Device::open(&device_info)?;
        let storage_object_vec =
            device_find_storage_objects(&device, Some(&storage_path.storage_name))?;

        let callback = if verbose > 0 {
            show_file_or_folder_with_details
        } else {
            show_file_or_folder_path_only
        };

        for storage_object_info in storage_object_vec {
            device_iterate_file_or_folder(
                &device,
                &device_info,
                &storage_object_info,
                &storage_path.path,
                recursive,
                callback,
            )?;
        }
    }
    Ok(())
}

fn show_file_or_folder_path_only(_info: &ContentObjectInfo, path: &str) {
    println!("{}", path);
}

fn show_file_or_folder_with_details(info: &ContentObjectInfo, path: &str) {
    println!(
        "[{:<4}] {:<19} {:<19} {}{} {}",
        if info.is_file() {
            "FILE"
        } else if info.is_folder() {
            "DIR"
        } else {
            ""
        },
        format_datetime_opt(&info.time_created),
        format_datetime_opt(&info.time_modified),
        if info.is_system { "S" } else { "-" },
        if info.is_hidden { "H" } else { "-" },
        path
    );
}

fn format_datetime_opt(opt: &Option<NaiveDateTime>) -> String {
    match &opt {
        None => "(not provided)".to_string(),
        Some(dt) => dt.to_string(),
    }
}
