use std::fmt::Debug;

use bindings::Windows::Win32::Devices::PortableDevices::{
    IPortableDeviceManager, PortableDeviceManager,
};
use bindings::Windows::Win32::Foundation::PWSTR;
use windows::Error;

use super::utils::*;

pub struct Manager {
    manager: IPortableDeviceManager,
}

#[derive(Debug)]
pub struct DeviceInfo {
    pub id: IDStr,
    pub name: String,
}

impl Manager {
    pub fn get_portable_device_manager() -> Result<Manager, Error> {
        let manager: IPortableDeviceManager = windows::create_instance(&PortableDeviceManager)?;
        Ok(Manager { manager })
    }

    pub fn get_device_iterator<'a>(&'a self) -> Result<DeviceInfoIterator<'a>, Error> {
        // get number of devices
        let mut device_id_count = 0u32;
        unsafe {
            self.manager
                .GetDevices(std::ptr::null_mut(), &mut device_id_count)
                .ok()?;
        }

        // get device ids
        let mut device_ids = WStrPtrArray::create(device_id_count);
        unsafe {
            self.manager
                .GetDevices(device_ids.as_mut_ptr(), &mut device_id_count)
                .ok()?;
        }

        Ok(DeviceInfoIterator::new(
            &self.manager,
            device_ids.to_vec_all(),
        ))
    }
}

pub struct DeviceInfoIterator<'a> {
    manager: &'a IPortableDeviceManager,
    device_ids: Vec<IDStr>,
}

impl<'a> DeviceInfoIterator<'a> {
    fn new(
        manager: &'a IPortableDeviceManager,
        mut device_ids: Vec<IDStr>,
    ) -> DeviceInfoIterator<'a> {
        device_ids.reverse(); // for moving item out by pop()
        DeviceInfoIterator::<'a> {
            manager,
            device_ids,
        }
    }

    pub fn next(&mut self) -> Result<Option<DeviceInfo>, Error> {
        let mut device_id = match self.device_ids.pop() {
            Some(id) => id,
            None => return Ok(None),
        };

        // get name length
        let mut name_buf_len = 0u32;
        unsafe {
            self.manager
                .GetDeviceFriendlyName(
                    device_id.as_pwstr(),
                    PWSTR::NULL,
                    &mut name_buf_len as *mut u32,
                )
                .ok()?;
        }

        // get name
        let mut name_buf = WStrBuf::create(name_buf_len);
        unsafe {
            self.manager
                .GetDeviceFriendlyName(
                    device_id.as_pwstr(),
                    name_buf.as_pwstr(),
                    &mut name_buf_len as *mut u32,
                )
                .ok()?;
        }

        let name = name_buf.to_string(name_buf_len - 1); // exclude null terminator

        Ok(Some(DeviceInfo {
            id: device_id,
            name,
        }))
    }
}
