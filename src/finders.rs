use crate::wpd::device::ContentObjectInfo;
use crate::wpd::device::{ContentObjectIterator, Device};
use crate::wpd::manager::DeviceInfo;
use crate::wpd::manager::Manager;

use crate::glob::filename::FileNamePattern;
use crate::glob::path::create_path_pattern_matcher;
use crate::glob::path::PathMatcher;
use crate::glob::path::PathMatchingState;

/// Returns WPD devices whose name is matching the specified pattern, or
/// returns all devices if the pattern was None.
pub fn device_find_devices(
    manager: &Manager,
    pattern: Option<&str>,
) -> Result<Vec<DeviceInfo>, Box<dyn std::error::Error>> {
    log::trace!("device_find_devices pattern={:?}", &pattern);

    let mut devices = Vec::<DeviceInfo>::new();

    let name_pattern = FileNamePattern::new(pattern.unwrap_or("*"));

    let mut iter = manager.get_device_iterator()?;
    while let Some(device_info) = iter.next()? {
        log::trace!("  detected \"{:?}\"", &device_info);
        if name_pattern.matches(&device_info.name) {
            log::trace!("   --> matched");
            devices.push(device_info);
        }
    }
    return Ok(devices);
}

/// Returns storage objects whose name is matching the specified pattern, or
/// returns all storage objects if the pattern was None.
pub fn device_find_storage_objects(
    device: &Device,
    pattern: Option<&str>,
) -> Result<Vec<ContentObjectInfo>, Box<dyn std::error::Error>> {
    log::trace!("device_find_storage_objects pattern={:?}", &pattern);

    let mut objects = Vec::<ContentObjectInfo>::new();

    let device_obj_info = match device_find_device_object(device)? {
        Some(info) => info,
        None => return Ok(objects),
    };

    let name_pattern = FileNamePattern::new(pattern.unwrap_or("*"));

    match device.get_object_iterator(&device_obj_info.content_object) {
        Err(err) => {
            log::debug!("{}", err);
            log::warn!("failed to open device: {}", &device_obj_info.name);
        }
        Ok(mut iter) => {
            while let Some(obj) = iter.next()? {
                log::trace!("  detected device object entry {:?}", &obj);
                let info = device.get_object_info(obj)?;
                log::trace!("   details {:?}", &info);
                if info.is_storage() && name_pattern.matches(&info.name) {
                    log::trace!("   --> storage object found");
                    objects.push(info);
                }
            }
        }
    }
    Ok(objects)
}

fn device_find_device_object(
    device: &Device,
) -> Result<Option<ContentObjectInfo>, Box<dyn std::error::Error>> {
    let root = device.get_root_object();
    match device.get_object_iterator(&root) {
        Err(err) => {
            log::debug!("{}", err);
            log::warn!("failed to get the device object: {}", &device.name);
        }
        Ok(mut iter) => {
            while let Some(obj) = iter.next()? {
                log::trace!("  detected device root entry {:?}", &obj);
                let info = device.get_object_info(obj)?;
                if info.is_device() {
                    log::trace!("   --> device object found");
                    return Ok(Some(info));
                }
            }
        }
    }
    Ok(None)
}

/// Returns the first matched object which is matching the specified path.
/// Path can be the glob pattern.
pub fn device_find_file_or_folder(
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
    device_find_file_or_folder_from(device, storage_object, &next_matcher.unwrap())
}

fn device_find_file_or_folder_from(
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
        let (state, next_matcher) = path_matcher.matches(&info.name, info.is_folder());
        match state {
            PathMatchingState::Rejected => continue,
            PathMatchingState::Completed => return Ok(Some(info)),
            PathMatchingState::Accepted => next_levels.push((info, next_matcher.unwrap())),
        }
    }

    for (info, next_matcher) in next_levels.iter() {
        let result = device_find_file_or_folder_from(device, info, next_matcher)?;
        if result.is_some() {
            return Ok(result);
        }
    }

    Ok(None)
}

pub fn device_iterate_file_or_folder<F>(
    device: &Device,
    device_info: &DeviceInfo,
    storage_object: &ContentObjectInfo,
    path: &str,
    recursive: bool,
    mut callback: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&ContentObjectInfo, &str) -> Result<bool, Box<dyn std::error::Error>>,
{
    log::trace!("device_iterate_file_or_folder path={}", path);
    let root_path_matcher = create_path_pattern_matcher(path)?;
    let storage_path = format!("{}:{}:", &device_info.name, &storage_object.name);

    let (state, next_matcher) = root_path_matcher.matches_root();
    log::trace!("  matches_root state {:?}", &state);
    match state {
        PathMatchingState::Rejected => Ok(()),
        PathMatchingState::Completed => {
            let path = join_path(&storage_path, "");
            log::trace!("  call callback path={:?}", &path);
            let continued = callback(storage_object, &path)?;
            if continued && recursive {
                log::trace!("  go recursively");
                match device.get_object_iterator(&storage_object.content_object) {
                    Err(err) => {
                        log::debug!("{}", err);
                        log::warn!("failed to open: {}", &storage_path);
                    }
                    Ok(iter) => {
                        let _ = iterate_file_or_folder_recursively(
                            device,
                            iter,
                            storage_path,
                            &mut callback,
                        )?;
                    }
                }
            }
            Ok(())
        }
        PathMatchingState::Accepted => {
            match device.get_object_iterator(&storage_object.content_object) {
                Err(err) => {
                    log::debug!("{}", err);
                    log::warn!("failed to open: {}", &storage_path);
                }
                Ok(iter) => {
                    let _ = iterate_file_or_folder(
                        device,
                        iter,
                        next_matcher.unwrap(),
                        storage_path,
                        recursive,
                        &mut callback,
                    )?;
                }
            }
            Ok(())
        }
    }
}

fn iterate_file_or_folder<F>(
    device: &Device,
    mut content_object_iterator: ContentObjectIterator,
    path_matcher: &PathMatcher,
    base_path: String,
    recursive: bool,
    callback: &mut F,
) -> Result<bool, Box<dyn std::error::Error>>
where
    F: FnMut(&ContentObjectInfo, &str) -> Result<bool, Box<dyn std::error::Error>>,
{
    log::trace!("iterate_file_or_folder start base_path={}", &base_path);
    let mut continued = true;
    while let Some(content_object) = content_object_iterator.next()? {
        log::trace!("  detected {:?}", &content_object);
        let content_object_info = device.get_object_info(content_object)?;
        log::trace!("  dftails {:?}", &content_object_info);
        if !content_object_info.is_file() && !content_object_info.is_folder() {
            log::trace!("  --> skip");
            continue;
        }

        let (state, next_matcher) =
            path_matcher.matches(&content_object_info.name, content_object_info.is_folder());
        log::trace!("  matching state {:?}", &state);

        match state {
            PathMatchingState::Rejected => (),
            PathMatchingState::Completed => {
                let next_base_path = join_path(&base_path, &content_object_info.name);
                log::trace!("  call callback path={:?}", &next_base_path);
                continued = callback(&content_object_info, &next_base_path)?;
                if !continued {
                    break;
                }
                if recursive {
                    log::trace!("  go recursively");
                    match device.get_object_iterator(&content_object_info.content_object) {
                        Err(err) => {
                            log::debug!("{}", err);
                            log::warn!("failed to open: {}", &next_base_path);
                        }
                        Ok(iter) => {
                            continued = iterate_file_or_folder_recursively(
                                device,
                                iter,
                                next_base_path,
                                callback,
                            )?;
                            if !continued {
                                break;
                            }
                        }
                    }
                }
            }
            PathMatchingState::Accepted => {
                let next_base_path = join_path(&base_path, &content_object_info.name);
                match device.get_object_iterator(&content_object_info.content_object) {
                    Err(err) => {
                        log::debug!("{}", err);
                        log::warn!("failed to open: {}", &next_base_path);
                    }
                    Ok(iter) => {
                        let next_content_object_iterator = iter;
                        continued = iterate_file_or_folder(
                            device,
                            next_content_object_iterator,
                            next_matcher.unwrap(),
                            next_base_path,
                            recursive,
                            callback,
                        )?;
                        if !continued {
                            break;
                        }
                    }
                }
            }
        }
    }
    log::trace!(
        "iterate_file_or_folder {} base_path={}",
        if continued { "end" } else { "stop" },
        &base_path
    );
    Ok(continued)
}

fn iterate_file_or_folder_recursively<F>(
    device: &Device,
    mut content_object_iterator: ContentObjectIterator,
    base_path: String,
    callback: &mut F,
) -> Result<bool, Box<dyn std::error::Error>>
where
    F: FnMut(&ContentObjectInfo, &str) -> Result<bool, Box<dyn std::error::Error>>,
{
    log::trace!(
        "iterate_file_or_folder_recursively start base_path={}",
        &base_path
    );
    let mut continued = true;
    while let Some(content_object) = content_object_iterator.next()? {
        let content_object_info = device.get_object_info(content_object)?;
        if !content_object_info.is_file() && !content_object_info.is_folder() {
            continue;
        }

        let path = join_path(&base_path, &content_object_info.name);
        continued = callback(&content_object_info, &path)?;
        if !continued {
            break;
        }
        if content_object_info.is_folder() {
            match device.get_object_iterator(&content_object_info.content_object) {
                Err(err) => {
                    log::debug!("{}", err);
                    log::warn!("failed to open: {}", &path);
                }
                Ok(iter) => {
                    continued = iterate_file_or_folder_recursively(device, iter, path, callback)?;
                    if !continued {
                        break;
                    }
                }
            }
        }
    }
    log::trace!(
        "iterate_file_or_folder_recursively {} base_path={}",
        if continued { "end" } else { "stop" },
        &base_path
    );
    Ok(continued)
}

fn join_path(base_path: &str, sub_path: &str) -> String {
    let mut s = String::from(base_path);
    s.push('\\');
    s.push_str(sub_path);
    s
}
