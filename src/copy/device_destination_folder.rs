use chrono::NaiveDateTime;
use std::collections::{HashMap, HashSet};

use crate::wpd::device::{ContentObjectInfo, Device};

use super::file_info::FileInfo;
use super::file_reader::FileReader;

use super::destination_folder::DestinationFolder;

pub struct DeviceDestinationFolder<'d> {
    device: &'d Device,
    folder_object_info: ContentObjectInfo,
    entry_map: HashMap<String, ContentObjectInfo>,
    retained: HashSet<String>,
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
        let retained = HashSet::<String>::new();

        Ok(DeviceDestinationFolder::<'d> {
            device,
            folder_object_info,
            entry_map,
            retained,
        })
    }
}

impl<'d> DestinationFolder for DeviceDestinationFolder<'d> {
    fn get_file_info(
        &mut self,
        name: &str,
    ) -> Result<Option<FileInfo>, Box<dyn std::error::Error>> {
        match self.entry_map.get(name) {
            None => Ok(None),
            Some(object_info) => Ok(Some(FileInfo::from_content_object_info(object_info)?)),
        }
    }

    fn create_file(
        &mut self,
        name: &str,
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

    fn open_or_create_folder<FBeforeOpen, FBeforeCreate>(
        &mut self,
        name: &str,
        before_open: FBeforeOpen,
        before_create: FBeforeCreate,
    ) -> Result<Box<Self>, Box<dyn std::error::Error>>
    where
        FBeforeOpen: FnOnce(&str),
        FBeforeCreate: FnOnce(&str),
    {
        match self.entry_map.get(name) {
            None => {
                // create
                before_create(name);
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
                before_open(name);
                Ok(Box::new(DeviceDestinationFolder::new(
                    self.device,
                    object_info_ref.clone(),
                )?))
            }
        }
    }

    fn delete_file_or_folder(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(object_info) = self.entry_map.get(name) {
            self.device.delete(&object_info.content_object)?;
            self.entry_map.remove(name);
        }
        Ok(())
    }

    fn retain(&mut self, name: &str) {
        self.retained.insert(String::from(name));
    }

    fn delete_unretained<FBeforeDeleteFile, FBeforeDeleteFolder>(
        &mut self,
        before_delete_file: FBeforeDeleteFile,
        before_delete_folder: FBeforeDeleteFolder,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        FBeforeDeleteFile: Fn(&str),
        FBeforeDeleteFolder: Fn(&str),
    {
        let mut delete_error: Option<windows::Error> = None;
        let mut names_to_delete = Vec::<String>::new();
        for (name, object_info) in self.entry_map.iter() {
            if object_info.is_file() || object_info.is_folder() {
                if !self.retained.contains(name) {
                    if object_info.is_file() {
                        before_delete_file(name);
                    } else if object_info.is_folder() {
                        before_delete_folder(name);
                    }

                    if let Err(err) = self.device.delete(&object_info.content_object) {
                        delete_error = Some(err);
                        break;
                    }
                    // entry_map.remove() is not allowed here because entry_map must be immutable
                    // as long as `name` and `object_info` refer internal data.
                    names_to_delete.push(String::from(name));
                }
            }
        }

        for name in names_to_delete.iter() {
            self.entry_map.remove(name);
        }

        if delete_error.is_some() {
            Err(delete_error.unwrap().into())
        } else {
            Ok(())
        }
    }
}
