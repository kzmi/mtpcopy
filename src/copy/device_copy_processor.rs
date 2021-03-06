use crate::wpd::device::{ContentObjectInfo, Device};

use super::destination_folder::DestinationFolder;
use super::device_file_reader::DeviceFileReader;
use super::file_info::FileInfo;

use super::copy_processor::{
    can_skip_copying, report_copying_end, report_copying_start, report_creating_new_folder,
    report_delete_file, report_delete_folder, CopyProcessor,
};

pub struct DeviceCopyProcessor<'d> {
    device: &'d Device,
    source_root_object_info: ContentObjectInfo,
}

impl<'d> DeviceCopyProcessor<'d> {
    pub fn new(device: &'d Device, source_root_object_info: ContentObjectInfo) -> Self {
        Self {
            device,
            source_root_object_info,
        }
    }
}

impl<'d> CopyProcessor for DeviceCopyProcessor<'d> {
    fn copy_as(
        &self,
        name: &str,
        dest: &mut impl DestinationFolder,
        dest_is_parent_folder: bool,
        recursive: bool,
        mirror: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        copy_hierarchy(
            self.device,
            dest,
            dest_is_parent_folder,
            &self.source_root_object_info,
            name,
            recursive,
            mirror,
        )
    }
}

fn copy_hierarchy(
    device: &Device,
    dest: &mut impl DestinationFolder,
    dest_is_parent_folder: bool,
    target_object_info: &ContentObjectInfo,
    dest_name: &str,
    recursive: bool,
    mirror: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if target_object_info.is_system || target_object_info.is_hidden {
        return Ok(());
    }

    if target_object_info.is_file() {
        let src_file_info = FileInfo::from_content_object_info(&target_object_info)?;
        let dest_file_info = dest.get_file_info(dest_name)?;

        if let Some(dest_file_info_ref) = dest_file_info.as_ref() {
            if can_skip_copying(&src_file_info, dest_file_info_ref) {
                dest.retain(dest_name);
                return Ok(());
            }
        }

        if dest_file_info.is_some() {
            dest.delete_file_or_folder(dest_name)?;
        }

        let res_reader = device.get_resoure(&target_object_info.content_object)?;
        let mut dev_reader = DeviceFileReader::new(res_reader);
        report_copying_start(&src_file_info);
        dest.create_file(
            dest_name,
            &mut dev_reader,
            src_file_info.data_size,
            &target_object_info.time_created,
            &target_object_info.time_modified,
        )?;
        dest.retain(dest_name);
        report_copying_end();

        return Ok(());
    }

    if target_object_info.is_folder() {
        let mut new_dest;
        let new_dest_ref;

        if dest_is_parent_folder {
            new_dest = dest.open_or_create_folder(dest_name, |_| {}, report_creating_new_folder)?;
            dest.retain(dest_name);
            new_dest_ref = new_dest.as_mut();
        } else {
            // if the source object was a folder, and the specified destination
            // was an existing folder, use the destination folder as it is.
            new_dest_ref = dest;
        }

        if recursive {
            let mut iter = device.get_object_iterator(&target_object_info.content_object)?;
            while let Some(content_object) = iter.next()? {
                let content_object_info = device.get_object_info(content_object)?;
                copy_hierarchy(
                    device,
                    new_dest_ref,
                    true, // dest_is_parent_folder
                    &content_object_info,
                    &content_object_info.name,
                    recursive,
                    mirror,
                )?;
            }

            if mirror {
                new_dest_ref.delete_unretained(report_delete_file, report_delete_folder)?;
            }
        }
    }
    Ok(())
}
