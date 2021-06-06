pub const SEPARATORS: &[char] = &['\\', '/'];
pub const WILDCARD_CHARACTERS: &[char] = &['*', '?'];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PathType {
    Invalid,
    DeviceStorage,
    Local,
}

pub fn get_path_type(path: &str) -> PathType {
    let colon_count = path.chars().filter(|ch| *ch == ':').count();
    if colon_count > 2 {
        PathType::Invalid
    } else if colon_count == 2 {
        PathType::DeviceStorage
    } else {
        PathType::Local
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DeviceStoragePath {
    pub device_name: String,
    pub storage_name: String,
    pub path: String,
}

impl DeviceStoragePath {
    pub fn from(path: &str) -> Result<DeviceStoragePath, Box<dyn std::error::Error>> {
        let mut path_sep: Vec<String> = path.split(':').map(|s| s.to_string()).collect();
        if path_sep.len() != 3 {
            return Err("invalid device storage path format.".into());
        }
        let mut path = path_sep.pop().unwrap();
        if path.len() == 0 {
            path = String::from("\\");
        }
        let storage_name = path_sep.pop().unwrap();
        let device_name = path_sep.pop().unwrap();
        Ok(DeviceStoragePath {
            device_name,
            storage_name,
            path,
        })
    }
}
