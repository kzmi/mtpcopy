fn main() {
    windows::build!(
        Windows::Win32::Com::{
                CoInitialize, CoCreateInstance, CoTaskMemFree, CLSCTX,
        },
        Windows::Win32::FileSystem::{
            CreateFileW,
            FILE_ACCESS_FLAGS,
            FILE_CREATION_DISPOSITION,
            FILE_FLAGS_AND_ATTRIBUTES,
            FILE_SHARE_MODE,
        },
        Windows::Win32::StructuredStorage::{IStream, STGC},
        Windows::Win32::SystemServices::{BOOL, HANDLE, S_OK, PWSTR},
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
        },
        Windows::Win32::WindowsProgramming::{
            CloseHandle,
            FILETIME,
            SetFileTime,
            SYSTEMTIME,
            SystemTimeToFileTime,
        },
        Windows::Win32::WindowsPropertiesSystem::PROPERTYKEY,
    );
}
