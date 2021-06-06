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
        let storage_name = path_sep.pop().unwrap();
        let device_name = path_sep.pop().unwrap();

        path = path
            .split(SEPARATORS)
            .filter(|s| s.len() > 0)
            .fold(String::new(), |mut s, p| {
                s.push('\\');
                s.push_str(p);
                s
            });
        if path.len() == 0 {
            path.push('\\');
        }

        Ok(DeviceStoragePath {
            device_name,
            storage_name,
            path,
        })
    }

    pub fn full_path(&self) -> String {
        format!(
            "{}:{}:{}",
            &self.device_name, &self.storage_name, &self.path
        )
    }

    pub fn file_name<'s>(&'s self) -> Option<&'s str> {
        if self.path.ends_with('\\') {
            None
        }
        else if let Some(index) = self.path.rfind('\\') {
            Some(&self.path[index+1..])
        } else {
            None
        }
    }

    pub fn parent(&self) -> Option<DeviceStoragePath> {
        if self.path == "\\" {
            return None;
        }
        let parent_path: String;
        if let Some(index) = self.path.rfind('\\') {
            if index == 0 {
                parent_path = String::from("\\");
            } else {
                parent_path = String::from(&self.path[..index]);
            }
        } else {
            parent_path = String::from("\\");
        }
        Some(DeviceStoragePath {
            device_name: self.device_name.clone(),
            storage_name: self.storage_name.clone(),
            path: parent_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceStoragePath;

    #[test]
    fn test_invalid_format() {
        assert!(DeviceStoragePath::from("").is_err());
        assert!(DeviceStoragePath::from("a").is_err());
        assert!(DeviceStoragePath::from("a:").is_err());
        assert!(DeviceStoragePath::from("a:b").is_err());
        assert!(DeviceStoragePath::from("a:b:c:").is_err());
    }

    fn check_valid_format(
        input: &str,
        expected_device_name: &str,
        expected_storage_name: &str,
        expected_path: &str,
    ) {
        assert_eq!(
            DeviceStoragePath::from(input).unwrap(),
            DeviceStoragePath {
                device_name: String::from(expected_device_name),
                storage_name: String::from(expected_storage_name),
                path: String::from(expected_path),
            }
        );
    }

    #[test]
    fn test_valid_format() {
        check_valid_format("a:b:", "a", "b", "\\");
        check_valid_format("a:b:\\", "a", "b", "\\");
        check_valid_format("a:b:/", "a", "b", "\\");
        check_valid_format("a:b:c", "a", "b", "\\c");
        check_valid_format("a:b:\\c", "a", "b", "\\c");
        check_valid_format("a:b:/c", "a", "b", "\\c");
        check_valid_format("a:b:\\\\c///d\\e/", "a", "b", "\\c\\d\\e");
        check_valid_format(
            "\u{3a3a}\u{3a3a}:\u{3a3a}\u{3a3a}\u{3a3a}:\\\u{5c5c}\\\u{5c5c}",
            "\u{3a3a}\u{3a3a}",
            "\u{3a3a}\u{3a3a}\u{3a3a}",
            "\\\u{5c5c}\\\u{5c5c}",
        );
    }

    #[test]
    fn test_full_path() {
        assert_eq!(
            DeviceStoragePath::from("a:b:c").unwrap().full_path(),
            String::from("a:b:\\c")
        );
    }

    #[test]
    fn test_file_name() {
        assert_eq!(DeviceStoragePath::from("a:b:").unwrap().file_name(), None);
        assert_eq!(DeviceStoragePath::from("a:b:/").unwrap().file_name(), None);
        assert_eq!(DeviceStoragePath::from("a:b:/c").unwrap().file_name(), Some("c"));
        assert_eq!(DeviceStoragePath::from("a:b:/c/d").unwrap().file_name(), Some("d"));
    }

    fn check_valid_parent(
        input: &str,
        expected_device_name: &str,
        expected_storage_name: &str,
        expected_path: &str,
    ) {
        assert_eq!(
            DeviceStoragePath::from(input).unwrap().parent().unwrap(),
            DeviceStoragePath {
                device_name: String::from(expected_device_name),
                storage_name: String::from(expected_storage_name),
                path: String::from(expected_path),
            }
        );
    }

    #[test]
    fn test_valid_parent() {
        check_valid_parent("a:b:/c/d/e", "a", "b", "\\c\\d");
        check_valid_parent("a:b:\\c\\d\\e", "a", "b", "\\c\\d");
        check_valid_parent("a:b:/c/d/e/", "a", "b", "\\c\\d");
        check_valid_parent("a:b:\\c\\d\\e\\", "a", "b", "\\c\\d");
        check_valid_parent("a:b:/c", "a", "b", "\\");
        check_valid_parent("a:b:\\c", "a", "b", "\\");
        check_valid_parent("a:b:\\\u{5c5c}\\\u{5c5c}", "a", "b", "\\\u{5c5c}");
    }

    #[test]
    fn test_invalid_parent() {
        assert!(DeviceStoragePath::from("a:b:").unwrap().parent().is_none());
        assert!(DeviceStoragePath::from("a:b:\\")
            .unwrap()
            .parent()
            .is_none());
        assert!(DeviceStoragePath::from("a:b:/").unwrap().parent().is_none());
    }
}
