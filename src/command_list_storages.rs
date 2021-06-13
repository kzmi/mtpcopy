use crate::wpd::device::Device;
use crate::wpd::manager::Manager;

use crate::finders::*;

pub fn command_list_storages() -> Result<(), Box<dyn std::error::Error>> {
    log::trace!("COMMAND list-storages");

    let manager = Manager::get_portable_device_manager()?;
    let device_info_vec = device_find_devices(&manager, None)?;

    let mut count = 0;
    for device_info in device_info_vec {
        match Device::open(&device_info) {
            Err(err) => {
                log::debug!("{}", err);
                log::warn!("failed to open \"{}\" (skipped)", device_info.name);
            }
            Ok(device) => match device_find_storage_objects(&device, None) {
                Err(err) => {
                    log::debug!("{}", err);
                    log::warn!("failed to get storages from \"{}\" (skipped)", device_info.name);
                }
                Ok(storage_object_vec) => {
                    for storage_object_info in storage_object_vec {
                        count += 1;
                        println!("{}:{}:", &device_info.name, &storage_object_info.name);
                    }
                }
            },
        }
    }
    if count == 0 {
        println!("no storages were found.")
    }
    Ok(())
}
