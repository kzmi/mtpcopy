use bindings::windows::Error;

use super::wpd::utils::init_com;
use super::wpd::device::Device;
use super::wpd::device::ContentObject;
use super::wpd::manager::Manager;

use std::io::Write;

pub async fn command_dir() -> Result<(), Error> {
    init_com();
    let manager = Manager::get_portable_device_manager()?;
    let mut iter = manager.get_device_iterator()?;
    while let Some(device_info) = iter.next()? {
        println!("{}", device_info.name);
        let device = Device::open(&device_info)?;
        walk(&device, &device.get_root_object(), &"".to_string())?;
    }
    Ok(())
}

fn walk(device: &Device, parent: &ContentObject, indent: &String) -> Result<(), Error> {
    let new_indent = indent.clone() + "  ";
    let mut iter = device.get_object_iterator(parent)?;
    while let Some(obj) = iter.next()? {
        let info = device.get_object_info(obj)?;
        println!("{}>{}<", indent, info.name);

        if info.is_file() && info.name == "12 Reach For The Sky.m4a" {
            let mut resource_reader = device.get_resoure(&info.content_object)?;
            let mut file = std::fs::File::create("C:\\Users\\kzmi\\Desktop\\rust\\foo.m4a").unwrap();
            while let Some(chunk) = resource_reader.next()? {
                let _ = file.write(chunk);
            }
            return Ok(());
        }

        walk(device, &info.content_object, &new_indent)?;
    }
    Ok(())
}
