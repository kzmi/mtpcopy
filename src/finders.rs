use crate::wpd::device::Device;
use crate::wpd::device::ContentObjectInfo;
use crate::wpd::manager::DeviceInfo;
use crate::wpd::manager::Manager;

use crate::glob::filename::FileNamePattern;
use crate::glob::path::create_path_pattern_matcher;
use crate::glob::path::PathMatcher;
use crate::glob::path::PathMatchingState;

pub fn find_devices(
    manager: &Manager,
    pattern: Option<&str>,
) -> Result<Vec<DeviceInfo>, Box<dyn std::error::Error>> {
    let mut devices = Vec::<DeviceInfo>::new();

    let has_name_pattern = pattern.is_some();
    let name_pattern = FileNamePattern::new(pattern.unwrap_or(""));

    let mut iter = manager.get_device_iterator()?;
    while let Some(device_info) = iter.next()? {
        if !has_name_pattern || name_pattern.matches(device_info.name.as_str()) {
            devices.push(device_info);
        }
    }
    return Ok(devices);
}

pub fn find_device_by_number(
    manager: &Manager,
    number: u32,
) -> Result<Vec<DeviceInfo>, Box<dyn std::error::Error>> {
    let mut devices = Vec::<DeviceInfo>::new();

    let mut count = 0u32;
    let mut iter = manager.get_device_iterator()?;
    while let Some(device_info) = iter.next()? {
        count += 1;
        if count == number {
            devices.push(device_info);
            break;
        }
    }
    return Ok(devices);
}

pub fn find_storage_objects(
    device: &Device,
    pattern: Option<&str>,
) -> Result<Vec<ContentObjectInfo>, Box<dyn std::error::Error>> {
    let mut objects = Vec::<ContentObjectInfo>::new();

    let device_obj_info = match find_device_object(device)? {
        Some(info) => info,
        None => return Ok(objects),
    };

    let has_name_pattern = pattern.is_some();
    let name_pattern = FileNamePattern::new(pattern.unwrap_or(""));

    let mut iter = device.get_object_iterator(&device_obj_info.content_object)?;
    while let Some(obj) = iter.next()? {
        let info = device.get_object_info(obj)?;
        if info.is_storage() && (!has_name_pattern || name_pattern.matches(info.name.as_str())) {
            objects.push(info);
        }
    }
    Ok(objects)
}

fn find_device_object(
    device: &Device,
) -> Result<Option<ContentObjectInfo>, Box<dyn std::error::Error>> {
    let root = device.get_root_object();
    let mut iter = device.get_object_iterator(&root)?;
    while let Some(obj) = iter.next()? {
        let info = device.get_object_info(obj)?;
        if info.is_device() {
            return Ok(Some(info));
        }
    }
    Ok(None)
}

pub fn find_file_or_folder(
    device: &Device,
    storage_object: &ContentObjectInfo,
    path: &str,
) -> Result<Option<ContentObjectInfo>, Box<dyn std::error::Error>> {
    let root_path_matcher = create_path_pattern_matcher(path)?;
    let (state, next_matcher) = root_path_matcher.matches_root();
    match state {
        PathMatchingState::Rejected => return Ok(None),
        PathMatchingState::Completed => return Ok(Some(storage_object.clone())),
        PathMatchingState::Accepted => (),
    }
    find_file_or_folder_from(device, storage_object, &next_matcher.unwrap())
}

fn find_file_or_folder_from(
    device: &Device,
    base: &ContentObjectInfo,
    path_matcher: &PathMatcher,
) -> Result<Option<ContentObjectInfo>, Box<dyn std::error::Error>> {
    let mut next_levels = Vec::<(ContentObjectInfo, &PathMatcher)>::new();

    let mut iter = device.get_object_iterator(&base.content_object)?;
    while let Some(obj) = iter.next()? {
        let info = device.get_object_info(obj)?;
        if info.is_functional_object() {
            continue;
        }
        let (state, next_matcher) = path_matcher.matches(info.name.as_str(), info.is_folder());
        match state {
            PathMatchingState::Rejected => continue,
            PathMatchingState::Completed => return Ok(Some(info)),
            PathMatchingState::Accepted => next_levels.push((info, next_matcher.unwrap())),
        }
    }

    for (info, next_matcher) in next_levels.iter() {
        let result = find_file_or_folder_from(device, info, next_matcher)?;
        if result.is_some() {
            return Ok(result);
        }
    }

    Ok(None)
}
