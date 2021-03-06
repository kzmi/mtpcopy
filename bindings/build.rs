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
            IPortableDeviceDataStream,
            IPortableDeviceKeyCollection,
            IPortableDeviceManager,
            IPortableDeviceProperties,
            IPortableDevicePropVariantCollection,
            IPortableDeviceResources,
            IPortableDeviceValues,
            PortableDevice,
            PortableDeviceKeyCollection,
            PortableDeviceManager,
            PortableDevicePropVariantCollection,
            PortableDeviceValues,
        },
        windows::win32::structured_storage::{IStream, STGC},
    );
}
