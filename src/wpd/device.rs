use bindings::windows::win32::windows_portable_devices::{
    IEnumPortableDeviceObjectIDs, IPortableDevice, IPortableDeviceContent,
    IPortableDeviceKeyCollection, IPortableDeviceProperties, IPortableDeviceResources,
    IPortableDeviceValues, PortableDevice, PortableDeviceKeyCollection, PortableDeviceValues,
};
use bindings::windows::Error;
use bindings::windows::ErrorCode;
use bindings::windows::Guid;
use bindings::windows::BOOL;
use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};

use super::guids::*;
use super::manager::DeviceInfo;
use super::property_keys::*;
use super::utils::*;

pub struct ContentObject {
    id: IDStr,
}

pub struct ObjectInfo {
    /// Name to display
    pub name: String,
    /// Content type GUID
    content_type: Guid,
    /// Category GUID of the functional object.
    /// Zeroes if the object was not a functional object.
    functional_object_category: Guid,
    /// Size of the resource data (or None if not provided)
    pub data_size: Option<u64>,
    /// Hidden flag
    pub is_hidden: bool,
    /// System flag
    pub is_system: bool,
    /// Whether the object can be deleted
    pub can_delete: bool,
    /// Time modified (or None if not provided)
    pub time_modified: Option<NaiveDateTime>,
}

impl ObjectInfo {
    pub fn is_functional_object(&self) -> bool {
        self.content_type == WPD_CONTENT_TYPE_FUNCTIONAL_OBJECT
    }

    pub fn is_device(&self) -> bool {
        self.functional_object_category == WPD_FUNCTIONAL_CATEGORY_DEVICE
    }

    pub fn is_storage(&self) -> bool {
        self.functional_object_category == WPD_FUNCTIONAL_CATEGORY_STORAGE
    }

    pub fn is_folder(&self) -> bool {
        self.content_type == WPD_CONTENT_TYPE_FOLDER
    }
}

pub struct Device {
    device: IPortableDevice,
    content: IPortableDeviceContent,
    properties: IPortableDeviceProperties,
    resources: IPortableDeviceResources,
}

impl Device {
    pub fn open(info: &DeviceInfo) -> Result<Device, Error> {
        let device: IPortableDevice = co_create_instance(&PortableDevice)?;
        let values: IPortableDeviceValues = co_create_instance(&PortableDeviceValues)?;
        unsafe {
            device.Open(info.id.as_ptr(), Some(values)).ok()?;
        }

        let mut content_receptor: Option<IPortableDeviceContent> = None;
        unsafe {
            device.Content(&mut content_receptor).ok()?;
        }
        let content = content_receptor.unwrap();

        let mut properties_receptor: Option<IPortableDeviceProperties> = None;
        unsafe {
            content.Properties(&mut properties_receptor).ok()?;
        }
        let properties = properties_receptor.unwrap();

        let mut resources_receptor: Option<IPortableDeviceResources> = None;
        unsafe {
            content.Transfer(&mut resources_receptor).ok()?;
        }
        let resources = resources_receptor.unwrap();

        Ok(Device {
            device,
            content,
            properties,
            resources,
        })
    }

    pub fn get_root_object(&self) -> ContentObject {
        ContentObject { id: vec![0u16] } // empty string
    }

    pub fn get_objects<F>(&self, parent: &ContentObject, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(&ContentObject) -> Result<(), Error>,
    {
        let mut enum_object_ids_receptor: Option<IEnumPortableDeviceObjectIDs> = None;
        unsafe {
            self.content
                .EnumObjects(0, parent.id.as_ptr(), None, &mut enum_object_ids_receptor)
                .ok()?;
        }
        let enum_object_ids = enum_object_ids_receptor.unwrap();

        const ARRAY_SIZE: u32 = 32;
        loop {
            // note that IDStrArrayBuf cannot be reused across iterations
            // because the obtained strings must be freed with its destructor.
            let mut object_ids = WStrPtrArray::create(ARRAY_SIZE);
            let mut read = 0u32;
            let err;
            unsafe {
                err = enum_object_ids.Next(object_ids.size(), object_ids.as_mut_ptr(), &mut read);
            }
            err.ok()?;

            for id in object_ids.to_vec(read) {
                callback(&ContentObject { id })?;
            }

            if err != ErrorCode::S_OK {
                break;
            }
        }
        Ok(())
    }

    pub fn get_object_info(&self, object: &ContentObject) -> Result<ObjectInfo, Error> {
        let key_collection: IPortableDeviceKeyCollection =
            co_create_instance(&PortableDeviceKeyCollection)?;
        unsafe {
            key_collection.Add(&WPD_OBJECT_NAME).ok()?;
            key_collection.Add(&WPD_OBJECT_ORIGINAL_FILE_NAME).ok()?;
            key_collection.Add(&WPD_OBJECT_SIZE).ok()?;
            key_collection.Add(&WPD_OBJECT_CONTENT_TYPE).ok()?;
            key_collection.Add(&WPD_FUNCTIONAL_OBJECT_CATEGORY).ok()?;
            key_collection.Add(&WPD_OBJECT_ISHIDDEN).ok()?;
            key_collection.Add(&WPD_OBJECT_ISSYSTEM).ok()?;
            key_collection.Add(&WPD_OBJECT_CAN_DELETE).ok()?;
            key_collection.Add(&WPD_OBJECT_DATE_MODIFIED).ok()?;
        }

        let mut values_receptor: Option<IPortableDeviceValues> = None;
        unsafe {
            self.properties
                .GetValues(
                    object.id.as_ptr(),
                    Some(key_collection),
                    &mut values_receptor,
                )
                .ok()?;
        }
        let values = values_receptor.unwrap();

        let mut object_name_ptr = WStrPtr::create();
        unsafe {
            values
                .GetStringValue(&WPD_OBJECT_NAME, object_name_ptr.as_mut_ptr())
                .ok()?;
        }
        let object_name = object_name_ptr.to_string();

        let mut content_type = Guid::zeroed();
        unsafe {
            values
                .GetGuidValue(&WPD_OBJECT_CONTENT_TYPE, &mut content_type)
                .ok()?;
        }

        let mut object_orig_name: Option<String> = None;
        let mut data_size: Option<u64> = None;
        let mut functional_object_category = Guid::zeroed();
        let mut is_hidden = false;
        let mut is_system = false;
        let mut can_delete = true;
        let mut time_modified: Option<NaiveDateTime> = None;

        if content_type == WPD_CONTENT_TYPE_FUNCTIONAL_OBJECT {
            unsafe {
                values
                    .GetGuidValue(
                        &WPD_FUNCTIONAL_OBJECT_CATEGORY,
                        &mut functional_object_category,
                    )
                    .ok()?;
            }
        } else {
            // get the original file name if it was provided
            let mut object_orig_name_ptr = WStrPtr::create();
            unsafe {
                let _ = values
                    .GetStringValue(
                        &WPD_OBJECT_ORIGINAL_FILE_NAME,
                        object_orig_name_ptr.as_mut_ptr(),
                    )
                    .and_then(|| object_orig_name = Some(object_orig_name_ptr.to_string()));
            }

            // get the hidden flag if it was provided
            let mut is_hidden_bool = BOOL::from(false);
            unsafe {
                let _ = values
                    .GetBoolValue(&WPD_OBJECT_ISHIDDEN, &mut is_hidden_bool)
                    .and_then(|| is_hidden = is_hidden_bool.as_bool());
            }

            // get the system flag if it was provided
            let mut is_system_bool = BOOL::from(false);
            unsafe {
                let _ = values
                    .GetBoolValue(&WPD_OBJECT_ISSYSTEM, &mut is_system_bool)
                    .and_then(|| is_system = is_system_bool.as_bool());
            }

            // get the can-delete flag if it was provided
            let mut can_delete_bool = BOOL::from(true);
            unsafe {
                let _ = values
                    .GetBoolValue(&WPD_OBJECT_CAN_DELETE, &mut can_delete_bool)
                    .and_then(|| can_delete = can_delete_bool.as_bool());
            }

            // get the time modified if it was provided
            let mut time_modified_ptr = WStrPtr::create();
            unsafe {
                let _ = values
                    .GetStringValue(&WPD_OBJECT_DATE_MODIFIED, time_modified_ptr.as_mut_ptr())
                    .and_then(|| {
                        let time_modified_s = &time_modified_ptr.to_string();
                        time_modified = parse_datetime(&time_modified_s);
                        println!("   {:?}", &time_modified_s);
                        println!("   {:?}", &time_modified);
                    });
            }

            if content_type != WPD_CONTENT_TYPE_FOLDER {
                // get the resource size
                let mut data_size_tmp = 0u64;
                unsafe {
                    let _ = values
                        .GetUnsignedLargeIntegerValue(&WPD_OBJECT_SIZE, &mut data_size_tmp)
                        .and_then(|| data_size = Some(data_size_tmp));
                }
            }
        }

        let name = object_orig_name.unwrap_or(object_name);

        Ok(ObjectInfo {
            name,
            content_type,
            functional_object_category,
            data_size,
            is_hidden,
            is_system,
            can_delete,
            time_modified,
        })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.Close();
        }
    }
}

fn parse_datetime(s: &String) -> Option<NaiveDateTime> {
    // YYYY/MM/DD:HH:MM:SS.SSS
    let date_part: String = s.chars().take(10).collect();
    let date = NaiveDate::parse_from_str(date_part.as_str(), "%Y/%m/%d").ok()?;

    let time_part: String = s.chars().skip(11).collect();
    let time = NaiveTime::parse_from_str(time_part.as_str(), "%H:%M:%S%.f").ok()?;

    Some(NaiveDateTime::new(date, time))
}
