use bindings::Windows::Win32::{
    StructuredStorage::{IStream, STGC},
    WindowsPortableDevices::IPortableDeviceDataStream,
};
use windows::Error;
use windows::Interface;

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

    pub fn next(&mut self, max_size: u32) -> Result<Option<&[u8]>, Error> {
        let available_buffer_size = std::cmp::min(self.buffer.len() as u32, max_size);
        let mut read: u32 = 0;
        unsafe {
            self.stream
                .Read(
                    self.buffer.as_mut_ptr().cast(),
                    available_buffer_size,
                    &mut read,
                )
                .ok()?;
        }
        if read == 0 {
            Ok(None)
        } else {
            Ok(Some(&self.buffer[..read as usize]))
        }
    }

    pub fn get_optimized_buffer_size(&self) -> u32 {
        self.buffer.len() as u32
    }
}

pub struct ResourceWriter {
    buff_size: u32,
    stream: IStream,
    committed: bool,
}

impl ResourceWriter {
    pub fn new(stream: IStream, buff_size: u32) -> ResourceWriter {
        ResourceWriter {
            buff_size,
            stream,
            committed: false,
        }
    }

    pub fn get_buffer_size(&self) -> u32 {
        self.buff_size
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        let data_len = data.len() as u32;
        let mut data_offset: u32 = 0;
        while data_offset < data_len {
            let write_len = std::cmp::min(data_len - data_offset, self.buff_size as u32);
            let mut written: u32 = 0;
            unsafe {
                self.stream
                    .Write(
                        data.as_ptr().offset(data_offset as isize) as *const std::ffi::c_void,
                        write_len,
                        &mut written,
                    )
                    .ok()?;
            }
            data_offset += written;
        }
        Ok(())
    }

    pub fn commit(&mut self) -> Result<ContentObject, Error> {
        self.committed = true;
        unsafe {
            self.stream.Commit(STGC::STGC_DEFAULT.0 as u32).ok()?;
        }

        let data_stream: IPortableDeviceDataStream = self.stream.cast()?;

        let mut object_id = WStrPtr::create();
        unsafe {
            data_stream.GetObjectID(object_id.as_pwstr_mut_ptr()).ok()?;
        }
        Ok(ContentObject::new(object_id.to_idstr()))
    }
}
