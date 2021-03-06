use bindings::windows::win32::com::{CoCreateInstance, CoInitialize, CoTaskMemFree, CLSCTX};
use bindings::windows::Abi;
use bindings::windows::Error;
use bindings::windows::Guid;
use bindings::windows::Interface;
use std::sync::Once;

pub type WChar = u16;
pub type IDStr = Vec<WChar>;

static INIT: Once = Once::new();

pub fn init_com() {
    INIT.call_once(|| unsafe {
        let _ = CoInitialize(std::ptr::null_mut());
    });
}

pub fn co_create_instance<T>(clsid: &Guid) -> Result<T, Error>
where
    T: Interface,
{
    let mut receptor: Option<T> = None;
    unsafe {
        CoCreateInstance(
            clsid,
            None,
            CLSCTX::CLSCTX_INPROC_SERVER.0 as u32,
            &T::IID,
            receptor.set_abi(),
        )
        .and_then(|| receptor.unwrap())
    }
}

/// Manages LPWSTR
pub struct WStrPtr {
    ptr: *mut WChar,
}

impl WStrPtr {
    pub fn create() -> WStrPtr {
        WStrPtr {
            ptr: std::ptr::null_mut(),
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut *mut WChar {
        &mut self.ptr
    }

    pub fn to_idstr(&self) -> IDStr {
        let len = get_wstr_length(self.ptr.cast());
        let idstr: IDStr;
        unsafe {
            idstr = std::slice::from_raw_parts(self.ptr.cast(), len + 1).to_vec(); // includes null terminator
        }
        idstr
    }

    pub fn to_string(&self) -> String {
        let len = get_wstr_length(self.ptr);
        unsafe {
            String::from_utf16_lossy(std::slice::from_raw_parts(self.ptr, len))
        }
    }
}

impl Drop for WStrPtr {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(self.ptr.cast());
        }
    }
}

/// Manages LPWSTR array
pub struct WStrPtrArray {
    ptr_vec: Vec<*mut WChar>,
}

impl WStrPtrArray {
    pub fn create(size: u32) -> WStrPtrArray {
        let mut ptr_vec = Vec::<*mut WChar>::new();
        ptr_vec.resize(size as usize, std::ptr::null_mut());
        WStrPtrArray { ptr_vec }
    }

    pub fn size(&self) -> u32 {
        self.ptr_vec.len() as u32
    }

    pub fn as_mut_ptr(&mut self) -> *mut *mut WChar {
        self.ptr_vec.as_mut_ptr()
    }

    pub fn to_vec(&self, size: u32) -> Vec<IDStr> {
        let mut idstr_vec = Vec::<IDStr>::with_capacity(size as usize);
        for p in &self.ptr_vec[..size as usize] {
            let len = get_wstr_length(p.cast());
            let idstr: IDStr;
            unsafe {
                idstr = std::slice::from_raw_parts(p.cast(), len + 1).to_vec(); // includes null terminator
            }
            idstr_vec.push(idstr);
        }
        idstr_vec
    }

    pub fn to_vec_all(&self) -> Vec<IDStr> {
        self.to_vec(self.size())
    }
}

impl Drop for WStrPtrArray {
    fn drop(&mut self) {
        unsafe {
            for p in &self.ptr_vec {
                CoTaskMemFree(p.cast());
            }
        }
    }
}

/// Manages WSTR
pub struct WStrBuf {
    buf: Vec<WChar>,
}

impl WStrBuf {
    pub fn create(size: u32) -> WStrBuf {
        let mut buf = Vec::<WChar>::new();
        buf.resize(size as usize, 0);
        WStrBuf { buf }
    }

    pub fn from(s: &str, include_null: bool) -> WStrBuf {
        let mut buf: Vec<WChar> = s.encode_utf16().collect();
        if include_null {
            buf.push(0);
        }
        WStrBuf { buf }
    }

    pub fn as_mut_ptr(&mut self) -> *mut WChar {
        self.buf.as_mut_ptr()
    }

    pub fn as_ptr(&self) -> *const WChar {
        self.buf.as_ptr()
    }

    pub fn to_string(&self, size: u32) -> String {
        String::from_utf16_lossy(&self.buf[..size as usize])
    }
}

fn get_wstr_length(p: *const WChar) -> usize {
    let mut len: usize = 0;
    let mut ptr: *const WChar = p;
    unsafe {
        while *ptr != 0 {
            ptr = ptr.offset(1);
            len += 1;
        }
    }
    len
}
