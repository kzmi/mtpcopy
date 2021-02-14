use bindings::windows::Error;

use super::wpd::utils::init_com;
use super::wpd::device::Device;
use super::wpd::device::ContentObject;
use super::wpd::manager::Manager;

pub async fn command_dir() -> Result<(), Error> {
    init_com();
    let manager = Manager::get_portable_device_manager()?;
    let devices = manager.get_devices()?;

    // TODO: select device
    let dev = &devices[0];
    println!("{}", dev.name);
    let device = Device::open(dev)?;

    walk(&device, &device.get_root_object(), &String::from(""))
}

fn walk(device: &Device, parent: &ContentObject, indent: &String) -> Result<(), Error> {
    let objects = device.get_objects(parent)?;
    let new_indent = indent.clone() + "  ";
    for obj in &objects {
        let name = device.get_object_name(obj)?;
        println!("{}{}", indent, name);
        walk(device, obj, &new_indent)?;
    }
    Ok(())
}
