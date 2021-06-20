use chrono::NaiveDateTime;

use super::{destination_folder::DestinationFolder, file_info::FileInfo};

pub trait CopyProcessor {
    fn copy_as(
        &self,
        name: &str,
        dest: &mut impl DestinationFolder,
        recursive: bool,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub fn can_skip_copying(src_file_info: &FileInfo, dest_file_info: &FileInfo) -> bool {
    if src_file_info.data_size == dest_file_info.data_size {
        if let Some(src_time) = get_file_time(src_file_info) {
            if let Some(dest_time) = get_file_time(dest_file_info) {
                // use .timestamp() value to ignore the subsecond
                let src_ts = src_time.timestamp();
                let dest_ts = dest_time.timestamp();
                log::debug!(
                    "src_time = {:?} ({:?})  dest_time = {:?} ({:?})",
                    &src_time,
                    &src_ts,
                    &dest_time,
                    &dest_ts
                );
                return src_ts <= dest_ts;
            }
        }
    }
    false
}

fn get_file_time(file_info: &FileInfo) -> Option<NaiveDateTime> {
    if let Some(time_created) = file_info.time_created {
        if let Some(time_modified) = file_info.time_modified {
            Some(std::cmp::max(time_created, time_modified))
        } else {
            Some(time_created)
        }
    } else {
        if let Some(time_modified) = file_info.time_modified {
            Some(time_modified)
        } else {
            None
        }
    }
}

pub fn report_copying_start(src_file_info: &FileInfo) {
    print!("copying {} ...", src_file_info.name);
}

pub fn report_copying_end() {
    println!("");
}
