use bindings::windows::Error;

use super::wpd::manager::Manager;
use super::wpd::utils::init_com;

pub async fn command_list() -> Result<(), Error> {
    init_com();
    let manager = Manager::get_portable_device_manager()?;
    let mut count = 0;
    manager.get_devices(|device_info| {
        count += 1;
        println!("{}: {}", count, device_info.name);
        Ok(())
    })?;
    if count == 0 {
        println!("no devices found.")
    }
    Ok(())
}
