use crate::wpd::device::ContentObjectInfo;
use chrono::{DateTime, Local, NaiveDateTime};
use std::{
    fs::Metadata,
    os::windows::prelude::MetadataExt,
};

pub struct FileInfo {
    /// Name to display
    pub name: String,
    /// Size of the resource data (or None if not provided)
    pub data_size: Option<u64>,
    /// Whether this entry is a folder
    pub is_folder: bool,
    /// Hidden flag
    pub is_hidden: bool,
    /// System flag
    pub is_system: bool,
    /// Whether the object can be deleted
    pub can_delete: bool,
    /// Time created (or None if not provided)
    pub time_created: Option<NaiveDateTime>,
    /// Time modified (or None if not provided)
    pub time_modified: Option<NaiveDateTime>,
}

impl FileInfo {
    pub fn from_content_object_info(
        info: &ContentObjectInfo,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(FileInfo {
            name: info.name.clone(),
            data_size: info.data_size.clone(),
            is_folder: info.is_folder(),
            is_hidden: info.is_hidden,
            is_system: info.is_system,
            can_delete: info.can_delete,
            time_created: info.time_created.clone(),
            time_modified: info.time_modified.clone(),
        })
    }

    pub fn from_metadata(
        metadata: &Metadata,
        name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let created_date_time  = DateTime::<Local>::from(metadata.created()?);
        let modified_date_time = DateTime::<Local>::from(metadata.modified()?);
        let file_attr = metadata.file_attributes();
        let data_size = if metadata.is_dir() {
            None
        } else {
            Some(metadata.file_size())
        };
        Ok(FileInfo {
            name: name.to_string(),
            data_size,
            is_folder: metadata.is_dir(),
            is_hidden: (file_attr & 2/* FILE_ATTRIBUTE_HIDDEN */) != 0,
            is_system: (file_attr & 4/* FILE_ATTRIBUTE_SYSTEM */) != 0,
            can_delete: true,
            time_created: Some(created_date_time.naive_local()),
            time_modified: Some(modified_date_time.naive_local()),
        })
    }
}
