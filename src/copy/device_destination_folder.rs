use chrono::NaiveDateTime;
use std::collections::HashMap;

use crate::wpd::device::{ContentObjectInfo, Device};

use super::file_info::FileInfo;
use super::file_reader::FileReader;

use super::destination_folder::DestinationFolder;

pub struct DeviceDestinationFolder<'d> {
    device: &'d Device,
    folder_object_info: ContentObjectInfo,
    entry_map: HashMap<String, ContentObjectInfo>,
}

impl<'d> DeviceDestinationFolder<'d> {
    pub fn new(
        device: &'d Device,
        folder_object_info: ContentObjectInfo,
    ) -> Result<DeviceDestinationFolder<'d>, Box<dyn std::error::Error>> {
        let mut iter = device.get_object_iterator(&folder_object_info.content_object)?;
        let mut entry_map = HashMap::<String, ContentObjectInfo>::new();
        while let Some(object) = iter.next()? {
            let object_info = device.get_object_info(object)?;
            entry_map.insert(object_info.name.clone(), object_info);
        }

        Ok(DeviceDestinationFolder::<'d> {
            device,
            folder_object_info,
            entry_map,
        })
    }
}

impl<'d> DestinationFolder for DeviceDestinationFolder<'d> {
    fn get_file_info(
        &mut self,
        name: &String,
    ) -> Result<Option<FileInfo>, Box<dyn std::error::Error>> {
        match self.entry_map.get(name) {
            None => Ok(None),
            Some(object_info) => Ok(Some(FileInfo::from_content_object_info(object_info)?)),
        }
    }

    fn create_file(
        &mut self,
        name: &String,
        reader: &mut impl FileReader,
        size: u64,
        created: &Option<NaiveDateTime>,
        modified: &Option<NaiveDateTime>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut resource_writer = self.device.create_file(
            &self.folder_object_info.content_object,
            name,
            size,
            created,
            modified,
        )?;

        while let Some(bytes) = reader.next(resource_writer.get_buffer_size())? {
            resource_writer.write(bytes)?;
        }
        let content_object = resource_writer.commit()?;

        let object_info = self.device.get_object_info(content_object)?;
        self.entry_map.insert(object_info.name.clone(), object_info);

        Ok(())
    }

    fn open_or_create_folder(
        &mut self,
        name: &String,
    ) -> Result<Box<Self>, Box<dyn std::error::Error>> {
        match self.entry_map.get(name) {
            None => {
                // create
                let content_object = self
                    .device
                    .create_folder(&self.folder_object_info.content_object, name)?;
                let object_info = self.device.get_object_info(content_object)?;
                self.entry_map
                    .insert(object_info.name.clone(), object_info.clone());
                Ok(Box::new(DeviceDestinationFolder::new(
                    self.device,
                    object_info,
                )?))
            }
            Some(object_info_ref) => {
                // open
                Ok(Box::new(DeviceDestinationFolder::new(
                    self.device,
                    object_info_ref.clone(),
                )?))
            }
        }
    }

    fn delete_file_or_folder(&mut self, name: &String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(object_info) = self.entry_map.get(name) {
            self.device.delete(&object_info.content_object)?;
            self.entry_map.remove(name);
        }
        Ok(())
    }
}
