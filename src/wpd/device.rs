use bindings::windows::win32::windows_portable_devices::{
    IEnumPortableDeviceObjectIDs, IPortableDevice, IPortableDeviceContent,
    IPortableDeviceKeyCollection, IPortableDeviceProperties, IPortableDeviceResources,
    IPortableDeviceValues, PortableDevice, PortableDeviceKeyCollection, PortableDeviceValues,
};
use bindings::windows::Error;
use bindings::windows::ErrorCode;
use bindings::windows::Guid;

use super::guids::*;
use super::manager::DeviceInfo;
use super::property_keys::*;
use super::utils::*;

pub struct Device {
    device: IPortableDevice,
    content: IPortableDeviceContent,
    properties: IPortableDeviceProperties,
    resources: IPortableDeviceResources,
}

pub struct ContentObject {
    id: IDStr,
}

impl Device {
    pub fn open(info: &DeviceInfo) -> Result<Device, Error> {
        let device: IPortableDevice = co_create_instance(&PortableDevice)?;
        let values: IPortableDeviceValues = co_create_instance(&PortableDeviceValues)?;
        device.Open(info.id.as_ptr(), Some(values)).ok()?;

        let mut content_receptor: Option<IPortableDeviceContent> = None;
        device.Content(&mut content_receptor).ok()?;
        let content = content_receptor.unwrap();

        let mut properties_receptor: Option<IPortableDeviceProperties> = None;
        content.Properties(&mut properties_receptor).ok()?;
        let properties = properties_receptor.unwrap();

        let mut resources_receptor: Option<IPortableDeviceResources> = None;
        content.Transfer(&mut resources_receptor).ok()?;
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

    pub fn get_objects<F>(
        &self,
        parent: &ContentObject,
        mut callback: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&ContentObject) -> Result<(), Error>,
    {
        let mut enum_object_ids_receptor: Option<IEnumPortableDeviceObjectIDs> = None;
        self.content
            .EnumObjects(0, parent.id.as_ptr(), None, &mut enum_object_ids_receptor)
            .ok()?;
        let enum_object_ids = enum_object_ids_receptor.unwrap();

        const ARRAY_SIZE: u32 = 32;
        loop {
            // note that IDStrArrayBuf cannot be reused across iterations
            // because the obtained strings must be freed with its destructor.
            let mut object_ids = WStrPtrArray::create(ARRAY_SIZE);
            let mut read = 0u32;
            let err = enum_object_ids.Next(object_ids.size(), object_ids.as_mut_ptr(), &mut read);
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

    pub fn get_object_name(&self, object: &ContentObject) -> Result<String, Error> {
        let key_collection: IPortableDeviceKeyCollection =
            co_create_instance(&PortableDeviceKeyCollection)?;
        key_collection.Add(&WPD_OBJECT_NAME).ok()?;
        key_collection.Add(&WPD_OBJECT_CONTENT_TYPE).ok()?;
        key_collection.Add(&WPD_FUNCTIONAL_OBJECT_CATEGORY).ok()?;

        let mut values_receptor: Option<IPortableDeviceValues> = None;
        self.properties
            .GetValues(
                object.id.as_ptr(),
                Some(key_collection),
                &mut values_receptor,
            )
            .ok()?;
        let values = values_receptor.unwrap();

        let mut object_name = WStrPtr::create();
        values
            .GetStringValue(&WPD_OBJECT_NAME, object_name.as_mut_ptr())
            .ok()?;

        let mut content_type_clsid = Guid::zeroed();
        values
            .GetGuidValue(&WPD_OBJECT_CONTENT_TYPE, &mut content_type_clsid)
            .ok()?;

        let mut category_clsid = Guid::zeroed();
        if content_type_clsid == WPD_CONTENT_TYPE_FUNCTIONAL_OBJECT {
            values
                .GetGuidValue(&WPD_FUNCTIONAL_OBJECT_CATEGORY, &mut category_clsid)
                .ok()?;
        }

        println!("   {:?}", content_type_clsid);
        println!("   {:?}", category_clsid);

        Ok(object_name.to_string())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.device.Close();
    }
}
