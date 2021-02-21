fn main() {
    windows::build!(
        windows::win32::com::{
            CoInitialize, CoCreateInstance, CoTaskMemFree, CLSCTX,
        },
        windows::win32::windows_properties_system::PROPERTYKEY,
        windows::win32::windows_portable_devices::{
            IEnumPortableDeviceObjectIDs,
            IPortableDevice,
            IPortableDeviceContent,
            IPortableDeviceKeyCollection,
            IPortableDeviceManager,
            IPortableDeviceProperties,
            IPortableDeviceResources,
            IPortableDeviceValues,
            PortableDevice,
            PortableDeviceKeyCollection,
            PortableDeviceManager,
            PortableDeviceValues,
        },
        windows::win32::structured_storage::{IStream, STGC},
    );
}
