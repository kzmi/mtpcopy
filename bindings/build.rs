fn main() {
    windows::build!(
        Windows::Win32::Com::{
            CoInitialize, CoCreateInstance, CoTaskMemFree,
        },
        Windows::Win32::FileSystem::{
            CreateFileW,
        },
        Windows::Win32::StructuredStorage::{IStream, STGC, PROPVARIANT},
        Windows::Win32::SystemServices::{S_OK},
        Windows::Win32::WindowsPortableDevices::{
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
            DELETE_OBJECT_OPTIONS,
        },
        Windows::Win32::WindowsProgramming::{
            CloseHandle,
            SetFileTime,
            SystemTimeToFileTime,
        },
        Windows::Win32::WindowsPropertiesSystem::{PROPERTYKEY},
    );
}
