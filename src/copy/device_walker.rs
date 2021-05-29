use crate::wpd::device::{ContentObjectInfo, Device};

use super::destination_folder::DestinationFolder;
use super::device_file_reader::DeviceFileReader;
use super::file_info::FileInfo;

use super::walker::{Walker, can_skip_copying, report_copying_end, report_copying_start};

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
        report_copying_start(&src_file_info);
        dest.create_file(
            &target_object_info.name,
            &mut dev_reader,
            src_file_info.data_size,
            &target_object_info.time_created,
            &target_object_info.time_modified,
        )?;
        report_copying_end();
        return Ok(());
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
