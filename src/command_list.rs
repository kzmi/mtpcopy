use bindings::windows::Error;

use super::wpd::utils::init_com;
use super::wpd::manager::Manager;

pub async fn command_list() -> Result<(), Error> {
    init_com();
    let manager = Manager::get_portable_device_manager()?;
    let devices = manager.get_devices()?;

    for dev in devices {
        println!(">{}<", dev.name);
    }

    Ok(())
}
