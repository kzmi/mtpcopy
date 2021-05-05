use bindings::Windows::Win32::{
    StructuredStorage::{IStream, STGC},
    WindowsPortableDevices::IPortableDeviceDataStream,
};
use windows::Interface;
use windows::Error;

use super::{device::ContentObject, utils::WStrPtr};

pub struct ResourceReader {
    stream: IStream,
    buffer: Vec<u8>,
}

impl ResourceReader {
    pub fn new(stream: IStream, buff_size: u32) -> ResourceReader {
        let mut buffer = Vec::<u8>::with_capacity(buff_size as usize);
        buffer.resize(buff_size as usize, 0);
        ResourceReader { stream, buffer }
    }

    pub fn next(&mut self) -> Result<Option<&[u8]>, Error> {
        let mut size = 0u32;
        unsafe {
            self.stream
                .Read(
                    self.buffer.as_mut_ptr() as *mut std::ffi::c_void,
                    self.buffer.len() as u32,
                    &mut size,
                )
                .ok()?;
        }
        if size == 0 {
            Ok(None)
        } else {
            Ok(Some(&self.buffer[..size as usize]))
        }
    }
}

pub struct ResourceWriter {
    pub buff_size: usize,
    stream: IStream,
    committed: bool,
}

impl ResourceWriter {
    pub fn new(stream: IStream, buff_size: u32) -> ResourceWriter {
        ResourceWriter {
            buff_size: buff_size as usize,
            stream,
            committed: false,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        let data_len = data.len();
        let mut data_offset: usize = 0;
        while data_offset < data_len {
            let write_len = std::cmp::min(data_len - data_offset, self.buff_size);
            let mut size = 0u32;
            unsafe {
                self.stream
                    .Write(
                        data.as_ptr().offset(data_offset as isize) as *const std::ffi::c_void,
                        write_len as u32,
                        &mut size,
                    )
                    .ok()?;
            }
            data_offset += write_len;
        }
        Ok(())
    }

    pub fn commit(&mut self) -> Result<ContentObject, Error> {
        self.committed = true;
        unsafe {
            self.stream.Commit(STGC::STGC_DEFAULT.0 as u32).ok();
        }

        let data_stream: IPortableDeviceDataStream = self.stream.cast()?;

        let mut object_id = WStrPtr::create();
        unsafe {
            data_stream.GetObjectID(object_id.as_pwstr_mut_ptr()).ok()?;
        }
        Ok(ContentObject::new(object_id.to_idstr()))
    }
}
