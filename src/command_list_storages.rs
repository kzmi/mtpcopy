use crate::wpd::device::Device;
use crate::wpd::manager::Manager;
use crate::wpd::utils::init_com;

use crate::finders::*;

pub fn command_list_storages() -> Result<(), Box<dyn std::error::Error>> {
    init_com();
    let manager = Manager::get_portable_device_manager()?;
    let device_info_vec = find_devices(&manager, None)?;

    let mut count = 0;
    for device_info in device_info_vec {
        let device = Device::open(&device_info)?;
        let storage_object_vec = find_storage_objects(&device, None)?;
        for storage_object_info in storage_object_vec {
            count += 1;
            println!("{}:{}:", &device_info.name, &storage_object_info.name);
        }
    }
    if count == 0 {
        println!("no devices found.")
    }
    Ok(())
}
