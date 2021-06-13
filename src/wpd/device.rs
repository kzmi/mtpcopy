use bindings::Windows::Win32::Devices::PortableDevices::{
    IEnumPortableDeviceObjectIDs, IPortableDevice, IPortableDeviceContent,
    IPortableDeviceKeyCollection, IPortableDevicePropVariantCollection, IPortableDeviceProperties,
    IPortableDeviceResources, IPortableDeviceValues, PortableDevice, PortableDeviceKeyCollection,
    PortableDevicePropVariantCollection, PortableDeviceValues,
    PORTABLE_DEVICE_DELETE_WITH_RECURSION,
};
use bindings::Windows::Win32::Foundation::{BOOL, S_OK};
use bindings::Windows::Win32::Storage::StructuredStorage::{
    IStream, PROPVARIANT_0_0_0_abi, PROPVARIANT_0_0_abi, PROPVARIANT, PROPVARIANT_0,
};
use bindings::Windows::Win32::System::PropertiesSystem::PROPERTYKEY;
use chrono::format::strftime::StrftimeItems;
use chrono::format::Parsed;
use chrono::naive::NaiveDateTime;
use std::{fmt::Debug, sync::Once};
use windows::Error;
use windows::Guid;

use super::guids::*;
use super::manager::DeviceInfo;
use super::property_keys::*;
use super::resource_stream::{ResourceReader, ResourceWriter};
use super::utils::*;

pub struct ContentObject {
    pub id: IDStr,
}

impl ContentObject {
    pub fn new(id: IDStr) -> ContentObject {
        ContentObject { id }
    }
}

impl Clone for ContentObject {
    fn clone(&self) -> Self {
        ContentObject {
            id: self.id.clone(),
        }
    }
}

impl Debug for ContentObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContentObject")
            .field("id", &self.id)
            .finish()
    }
}

#[derive(Debug)]
pub struct ContentObjectInfo {
    pub content_object: ContentObject,
    /// Name to display
    pub name: String,
    /// Content type GUID
    content_type: Guid,
    /// Category GUID of the functional object.
    /// Zeroes if the object was not a functional object.
    functional_object_category: Guid,
    /// Size of the resource data
    pub data_size: u64,
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

impl Clone for ContentObjectInfo {
    fn clone(&self) -> Self {
        ContentObjectInfo {
            content_object: self.content_object.clone(),
            name: self.name.clone(),
            content_type: self.content_type.clone(),
            functional_object_category: self.functional_object_category.clone(),
            data_size: self.data_size,
            is_hidden: self.is_hidden,
            is_system: self.is_system,
            can_delete: self.can_delete,
            time_created: self.time_created.clone(),
            time_modified: self.time_modified.clone(),
        }
    }
}

impl ContentObjectInfo {
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

    pub fn is_file(&self) -> bool {
        !self.is_functional_object() && !self.is_folder()
    }
}

pub struct Device {
    device: IPortableDevice,
    content: IPortableDeviceContent,
    properties: IPortableDeviceProperties,
    resources: IPortableDeviceResources,
    pub name: String,
}

impl Device {
    pub fn open(info: &DeviceInfo) -> Result<Device, Error> {
        log::trace!("open Device ({})", &info.name);

        let device: IPortableDevice = windows::create_instance(&PortableDevice)?;
        let values: IPortableDeviceValues = windows::create_instance(&PortableDeviceValues)?;
        unsafe {
            device.Open(info.id.clone().as_pwstr(), values).ok()?;
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
            name: info.name.clone(),
        })
    }

    pub fn get_root_object(&self) -> ContentObject {
        ContentObject::new(IDStr::create_empty())
    }

    pub fn get_object_iterator(
        &self,
        parent: &ContentObject,
    ) -> Result<ContentObjectIterator, Error> {
        let mut enum_object_ids_receptor: Option<IEnumPortableDeviceObjectIDs> = None;
        unsafe {
            self.content
                .EnumObjects(
                    0,
                    parent.id.clone().as_pwstr(),
                    None,
                    &mut enum_object_ids_receptor,
                )
                .ok()?;
        }
        let enum_object_ids = enum_object_ids_receptor.unwrap();

        Ok(ContentObjectIterator::new(enum_object_ids))
    }

    pub fn get_object_info(&self, object: ContentObject) -> Result<ContentObjectInfo, Error> {
        let key_collection: IPortableDeviceKeyCollection =
            windows::create_instance(&PortableDeviceKeyCollection)?;
        unsafe {
            key_collection.Add(&WPD_OBJECT_NAME).ok()?;
            key_collection.Add(&WPD_OBJECT_ORIGINAL_FILE_NAME).ok()?;
            key_collection.Add(&WPD_OBJECT_SIZE).ok()?;
            key_collection.Add(&WPD_OBJECT_CONTENT_TYPE).ok()?;
            key_collection.Add(&WPD_FUNCTIONAL_OBJECT_CATEGORY).ok()?;
            key_collection.Add(&WPD_OBJECT_ISHIDDEN).ok()?;
            key_collection.Add(&WPD_OBJECT_ISSYSTEM).ok()?;
            key_collection.Add(&WPD_OBJECT_CAN_DELETE).ok()?;
            key_collection.Add(&WPD_OBJECT_DATE_CREATED).ok()?;
            key_collection.Add(&WPD_OBJECT_DATE_MODIFIED).ok()?;
        }

        let mut values_receptor: Option<IPortableDeviceValues> = None;
        unsafe {
            self.properties
                .GetValues(
                    object.id.clone().as_pwstr(),
                    Some(key_collection),
                    &mut values_receptor,
                )
                .ok()?;
        }
        let values = values_receptor.unwrap();

        let mut object_name_ptr = WStrPtr::create();
        unsafe {
            values
                .GetStringValue(&WPD_OBJECT_NAME, object_name_ptr.as_pwstr_mut_ptr())
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
        let mut data_size: u64 = 0;
        let mut functional_object_category = Guid::zeroed();
        let mut is_hidden = false;
        let mut is_system = false;
        let mut can_delete = true;
        let mut time_created: Option<NaiveDateTime> = None;
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
                        object_orig_name_ptr.as_pwstr_mut_ptr(),
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

            // get the time created if it was provided
            let mut time_created_ptr = WStrPtr::create();
            unsafe {
                let _ = values
                    .GetStringValue(
                        &WPD_OBJECT_DATE_CREATED,
                        time_created_ptr.as_pwstr_mut_ptr(),
                    )
                    .and_then(|| {
                        let time_created_s = &time_created_ptr.to_string();
                        time_created = parse_datetime(&time_created_s);
                    });
            }

            // get the time modified if it was provided
            let mut time_modified_ptr = WStrPtr::create();
            unsafe {
                let _ = values
                    .GetStringValue(
                        &WPD_OBJECT_DATE_MODIFIED,
                        time_modified_ptr.as_pwstr_mut_ptr(),
                    )
                    .and_then(|| {
                        let time_modified_s = &time_modified_ptr.to_string();
                        time_modified = parse_datetime(&time_modified_s);
                    });
            }

            if content_type != WPD_CONTENT_TYPE_FOLDER {
                // get the resource size
                let mut data_size_tmp = 0u64;
                unsafe {
                    let _ = values
                        .GetUnsignedLargeIntegerValue(&WPD_OBJECT_SIZE, &mut data_size_tmp)
                        .and_then(|| data_size = data_size_tmp);
                }
            }
        }

        let name = object_orig_name.unwrap_or(object_name);

        Ok(ContentObjectInfo {
            content_object: object,
            name,
            content_type,
            functional_object_category,
            data_size,
            is_hidden,
            is_system,
            can_delete,
            time_created,
            time_modified,
        })
    }

    #[allow(dead_code)]
    pub fn get_resource_keys(&self, object: &ContentObject) -> Result<Vec<PROPERTYKEY>, Error> {
        let mut key_collection_receptor: Option<IPortableDeviceKeyCollection> = None;
        unsafe {
            self.resources
                .GetSupportedResources(object.id.clone().as_pwstr(), &mut key_collection_receptor)
                .ok()?;
        }
        let key_collection = key_collection_receptor.unwrap();

        let mut count = 0u32;
        unsafe {
            key_collection.GetCount(&mut count).ok()?;
        }

        let mut property_keys = Vec::<PROPERTYKEY>::new();
        for i in 0..count as u32 {
            let mut propkey = make_empty_propertykey();
            unsafe {
                key_collection.GetAt(i, &mut propkey).ok()?;
            }
            property_keys.push(propkey);
        }

        Ok(property_keys)
    }

    pub fn get_resoure(&self, object: &ContentObject) -> Result<ResourceReader, Error> {
        const STGM_READ: u32 = 0;
        let mut buff_size: u32 = 0;
        let mut stream_receptor: Option<IStream> = None;
        unsafe {
            self.resources
                .GetStream(
                    object.id.clone().as_pwstr(),
                    &WPD_RESOURCE_DEFAULT,
                    STGM_READ,
                    &mut buff_size,
                    &mut stream_receptor,
                )
                .ok()?;
        }
        let stream = stream_receptor.unwrap();
        Ok(ResourceReader::new(stream, buff_size))
    }

    pub fn create_file(
        &self,
        parent: &ContentObject,
        name: &str,
        size: u64,
        created: &Option<NaiveDateTime>,
        modified: &Option<NaiveDateTime>,
    ) -> Result<ResourceWriter, Error> {
        let values: IPortableDeviceValues = windows::create_instance(&PortableDeviceValues)?;
        let mut name_buf = WStrBuf::from(name, true);
        unsafe {
            values
                .SetStringValue(&WPD_OBJECT_PARENT_ID, parent.id.clone().as_pwstr())
                .ok()?;
            values
                .SetStringValue(&WPD_OBJECT_NAME, name_buf.as_pwstr())
                .ok()?;
            values
                .SetStringValue(&WPD_OBJECT_ORIGINAL_FILE_NAME, name_buf.as_pwstr())
                .ok()?;
            values
                .SetGuidValue(&WPD_OBJECT_FORMAT, &WPD_OBJECT_FORMAT_ALL)
                .ok()?;
            values
                .SetGuidValue(&WPD_OBJECT_CONTENT_TYPE, &WPD_CONTENT_TYPE_GENERIC_FILE)
                .ok()?;
            values
                .SetUnsignedLargeIntegerValue(&WPD_OBJECT_SIZE, size)
                .ok()?;
        }
        if let Some(&created_dt) = created.as_ref() {
            let dt = format_datetime(&created_dt);
            let mut dt_buf = WStrBuf::from(&dt, true);
            unsafe {
                values
                    .SetStringValue(&WPD_OBJECT_DATE_CREATED, dt_buf.as_pwstr())
                    .ok()?;
            }
        }
        if let Some(&modified_dt) = modified.as_ref() {
            let dt = format_datetime(&modified_dt);
            let mut dt_buf = WStrBuf::from(&dt, true);
            unsafe {
                values
                    .SetStringValue(&WPD_OBJECT_DATE_MODIFIED, dt_buf.as_pwstr())
                    .ok()?;
            }
        }

        let mut stream_receptor: Option<IStream> = None;
        let mut buffer_size: u32 = 0;

        unsafe {
            self.content
                .CreateObjectWithPropertiesAndData(
                    Some(values),
                    &mut stream_receptor,
                    &mut buffer_size,
                    std::ptr::null_mut(),
                )
                .ok()?;
        }

        let stream = stream_receptor.unwrap();

        Ok(ResourceWriter::new(stream, buffer_size))
    }

    pub fn create_folder(
        &self,
        parent: &ContentObject,
        name: &str,
    ) -> Result<ContentObject, Error> {
        let values: IPortableDeviceValues = windows::create_instance(&PortableDeviceValues)?;
        let mut name_buf = WStrBuf::from(name, true);
        unsafe {
            values
                .SetStringValue(&WPD_OBJECT_PARENT_ID, parent.id.clone().as_pwstr())
                .ok()?;
            values
                .SetStringValue(&WPD_OBJECT_NAME, name_buf.as_pwstr())
                .ok()?;
            values
                .SetGuidValue(&WPD_OBJECT_FORMAT, &WPD_OBJECT_FORMAT_ALL)
                .ok()?;
            values
                .SetGuidValue(&WPD_OBJECT_CONTENT_TYPE, &WPD_CONTENT_TYPE_FOLDER)
                .ok()?;
        }

        let mut object_id = WStrPtr::create();
        unsafe {
            self.content
                .CreateObjectWithPropertiesOnly(Some(values), object_id.as_pwstr_mut_ptr())
                .ok()?;
        }
        let content_object = ContentObject::new(object_id.to_idstr());

        Ok(content_object)
    }

    pub fn delete(&self, object: &ContentObject) -> Result<(), Error> {
        let collection: IPortableDevicePropVariantCollection =
            windows::create_instance(&PortableDevicePropVariantCollection)?;
        let propvar = PROPVARIANT {
            Anonymous: PROPVARIANT_0 {
                Anonymous: PROPVARIANT_0_0_abi {
                    vt: 31, // VT_LPWSTR
                    wReserved1: 0,
                    wReserved2: 0,
                    wReserved3: 0,
                    Anonymous: PROPVARIANT_0_0_0_abi {
                        pwszVal: object.id.clone().as_pwstr(),
                    },
                },
            },
        };
        unsafe {
            collection.Add(&propvar).ok()?;
            self.content
                .Delete(
                    PORTABLE_DEVICE_DELETE_WITH_RECURSION.0 as u32,
                    &collection,
                    std::ptr::null_mut(),
                )
                .ok()?;
        }
        Ok(())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        log::trace!("drop Device ({})", &self.name);
        unsafe {
            let _ = self.device.Close();
        }
    }
}

pub struct ContentObjectIterator {
    enum_object_ids: IEnumPortableDeviceObjectIDs,
    object_ids: Option<Vec<IDStr>>,
    completed: bool,
}

impl ContentObjectIterator {
    fn new(enum_object_ids: IEnumPortableDeviceObjectIDs) -> ContentObjectIterator {
        ContentObjectIterator {
            enum_object_ids,
            object_ids: None,
            completed: false,
        }
    }

    pub fn next(&mut self) -> Result<Option<ContentObject>, Error> {
        if let Some(object_ids_ref) = self.object_ids.as_mut() {
            if let Some(id) = object_ids_ref.pop() {
                return Ok(Some(ContentObject::new(id)));
            }
        }

        if self.completed {
            return Ok(None);
        }

        const ARRAY_SIZE: u32 = 32;
        let mut object_ids = WStrPtrArray::create(ARRAY_SIZE);
        let mut read = 0u32;
        let err;
        unsafe {
            err = self
                .enum_object_ids
                .Next(object_ids.size(), object_ids.as_mut_ptr(), &mut read);
        }
        err.ok()?;

        if read == 0 {
            self.object_ids = None;
            self.completed = true;
            return Ok(None);
        }

        let mut object_ids_vec = object_ids.to_vec(read);
        object_ids_vec.reverse(); // for moving item out by pop()
        self.object_ids = Some(object_ids_vec);

        if err != S_OK {
            self.completed = true;
        }

        self.next()
    }
}

static INIT_PARSING: Once = Once::new();
static mut DATE_FORMAT_ITEMS: Vec<chrono::format::Item> = Vec::<chrono::format::Item>::new();
static mut TIME_FORMAT_ITEMS: Vec<chrono::format::Item> = Vec::<chrono::format::Item>::new();

fn parse_datetime(s: &String) -> Option<NaiveDateTime> {
    INIT_PARSING.call_once(|| unsafe {
        DATE_FORMAT_ITEMS.clear();
        DATE_FORMAT_ITEMS.extend(StrftimeItems::new("%Y/%m/%d"));
        TIME_FORMAT_ITEMS.clear();
        TIME_FORMAT_ITEMS.extend(StrftimeItems::new("%H:%M:%S%.f"));
    });
    // YYYY/MM/DD:HH:MM:SS.SSS
    let date;
    {
        let date_part: String = s.chars().take(10).collect();
        let mut parsed_date = Parsed::new();
        chrono::format::parse(&mut parsed_date, &date_part, unsafe {
            DATE_FORMAT_ITEMS.iter()
        })
        .ok()?;
        date = parsed_date.to_naive_date().ok()?;
    }

    let time;
    {
        let time_part: String = s.chars().skip(11).collect();
        let mut parsed_time = Parsed::new();
        chrono::format::parse(&mut parsed_time, &time_part, unsafe {
            TIME_FORMAT_ITEMS.iter()
        })
        .ok()?;
        time = parsed_time.to_naive_time().ok()?;
    }

    Some(NaiveDateTime::new(date, time))
}

static INIT_FORMATTING: Once = Once::new();
static mut DATETIME_FORMAT: Vec<chrono::format::Item> = Vec::<chrono::format::Item>::new();

fn format_datetime(dt: &NaiveDateTime) -> String {
    INIT_FORMATTING.call_once(|| unsafe {
        DATETIME_FORMAT.clear();
        DATETIME_FORMAT.extend(StrftimeItems::new("%Y/%m/%d:%H:%M:%S%.3f"));
    });
    // YYYY/MM/DD:HH:MM:SS.SSS
    dt.format_with_items(unsafe { DATETIME_FORMAT.iter() })
        .to_string()
}
