use bindings::windows::Error;

use super::wpd::utils::init_com;
use super::wpd::device::Device;
use super::wpd::device::ContentObject;
use super::wpd::manager::Manager;

pub async fn command_dir() -> Result<(), Error> {
    init_com();
    let manager = Manager::get_portable_device_manager()?;
    manager.get_devices(|device_info| {
        println!("{}", device_info.name);
        let device = Device::open(device_info)?;
        walk(&device, &device.get_root_object(), &String::from(""))
    })?;
    Ok(())
}

fn walk(device: &Device, parent: &ContentObject, indent: &String) -> Result<(), Error> {
    let new_indent = indent.clone() + "  ";
    device.get_objects(parent, |obj| {
        let name = device.get_object_name(obj)?;
        println!("{}>{}<", indent, name);
        walk(device, obj, &new_indent)?;
        Ok(())
    })?;
    Ok(())
}
