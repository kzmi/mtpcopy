use bindings::windows::Error;

use super::wpd::manager::Manager;
use super::wpd::utils::init_com;

pub async fn command_list() -> Result<(), Error> {
    init_com();
    let manager = Manager::get_portable_device_manager()?;
    let mut count = 0;
    let mut iter = manager.get_device_iterator()?;
    while let Some(device_info) = iter.next()? {
        count += 1;
        println!("{}: >{}<", count, device_info.name);
    }
    if count == 0 {
        println!("no devices found.")
    }
    Ok(())
}
