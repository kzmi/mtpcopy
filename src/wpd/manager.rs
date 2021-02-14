use bindings::windows::win32::windows_portable_devices::{
    IPortableDeviceManager, PortableDeviceManager,
};
use bindings::windows::Error;

use super::utils::*;

pub struct Manager {
    manager: IPortableDeviceManager,
}

pub struct DeviceInfo {
    pub id: IDStr,
    pub name: String,
}

impl Manager {
    pub fn get_portable_device_manager() -> Result<Manager, Error> {
        let manager: IPortableDeviceManager = co_create_instance(&PortableDeviceManager)?;
        Ok(Manager { manager })
    }

    pub fn get_devices<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(&DeviceInfo) -> Result<(), Error>,
    {
        let device_ids = self.get_device_ids()?;

        for mut device_id in device_ids {
            // get name length
            let mut name_buf_len = 0u32;
            self.manager
                .GetDeviceFriendlyName(
                    device_id.as_mut_ptr(),
                    std::ptr::null_mut(),
                    &mut name_buf_len as *mut u32,
                )
                .ok()?;

            // get name
            let mut name_buf = WStrBuf::create(name_buf_len);
            self.manager
                .GetDeviceFriendlyName(
                    device_id.as_mut_ptr(),
                    name_buf.as_mut_ptr(),
                    &mut name_buf_len as *mut u32,
                )
                .ok()?;

            let name = name_buf.to_string(name_buf_len - 1); // exclude null terminator

            let device_info = DeviceInfo {
                id: device_id,
                name,
            };
            callback(&device_info)?
        }
        Ok(())
    }

    fn get_device_ids(&self) -> Result<Vec<IDStr>, Error> {
        // get number of devices
        let mut device_id_count = 0u32;
        self.manager
            .GetDevices(std::ptr::null_mut(), &mut device_id_count)
            .ok()?;

        // get device ids
        let mut device_ids = WStrPtrArray::create(device_id_count);
        self.manager
            .GetDevices(device_ids.as_mut_ptr(), &mut device_id_count)
            .ok()?;

        Ok(device_ids.to_vec_all())
    }
}
