use bindings::Windows::Win32::Foundation::PWSTR;
use bindings::Windows::Win32::System::Com::CoTaskMemFree;
use std::fmt::{Debug, Write};

pub type WChar = u16;

/// Manages object ID (WCHAR string which ends with a NULL terminator)
pub struct IDStr {
    vec: Vec<WChar>,
}

impl IDStr {
    pub fn create_empty() -> IDStr {
        let mut vec = Vec::<WChar>::with_capacity(1);
        vec.extend([0].iter());
        IDStr { vec }
    }

    pub fn from(p: PWSTR) -> IDStr {
        let len = get_wstr_length(p);
        let vec: Vec<WChar>;
        unsafe {
            vec = std::slice::from_raw_parts(p.0, len + 1).to_vec(); // includes null terminator
        }
        IDStr { vec }
    }

    pub fn as_pwstr(&mut self) -> PWSTR {
        PWSTR(self.vec.as_mut_ptr())
    }

    // TODO: need as_pcwstr(&self) -> PCWSTR
}

impl Clone for IDStr {
    fn clone(&self) -> Self {
        IDStr {
            vec: self.vec.clone(),
        }
    }
}

impl Debug for IDStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        for wc in self.vec.iter() {
            if *wc >= 0x20 && *wc <= 0x7e {
                f.write_char(*wc as u8 as char)?;
            } else if *wc == 0 {
                f.write_str("{NUL}")?;
            } else {
                f.write_fmt(format_args!("{{{:04x}}}", wc))?;
            }
        }
        f.write_char('"')
    }
}

/// Manages LPWSTR
pub struct WStrPtr {
    ptr: PWSTR,
}

impl WStrPtr {
    pub fn create() -> WStrPtr {
        WStrPtr { ptr: PWSTR::NULL }
    }

    pub fn as_pwstr_mut_ptr(&mut self) -> *mut PWSTR {
        &mut self.ptr
    }

    pub fn to_idstr(&self) -> IDStr {
        IDStr::from(self.ptr)
    }

    pub fn to_string(&self) -> String {
        let len = get_wstr_length(self.ptr);
        unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(self.ptr.0, len)) }
    }
}

impl Drop for WStrPtr {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(self.ptr.0.cast());
        }
    }
}

/// Manages LPWSTR array
pub struct WStrPtrArray {
    ptr_vec: Vec<PWSTR>,
}

impl WStrPtrArray {
    pub fn create(size: u32) -> WStrPtrArray {
        let mut ptr_vec = Vec::<PWSTR>::new();
        ptr_vec.resize(size as usize, PWSTR::NULL);
        WStrPtrArray { ptr_vec }
    }

    pub fn size(&self) -> u32 {
        self.ptr_vec.len() as u32
    }

    pub fn as_mut_ptr(&mut self) -> *mut PWSTR {
        self.ptr_vec.as_mut_ptr()
    }

    pub fn to_vec(&self, size: u32) -> Vec<IDStr> {
        let mut idstr_vec = Vec::<IDStr>::with_capacity(size as usize);
        for p in &self.ptr_vec[..size as usize] {
            idstr_vec.push(IDStr::from(*p));
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
                CoTaskMemFree(p.0.cast());
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

    pub fn as_pwstr(&mut self) -> PWSTR {
        PWSTR(self.buf.as_mut_ptr())
    }

    pub fn to_string(&self, size: u32) -> String {
        String::from_utf16_lossy(&self.buf[..size as usize])
    }
}

fn get_wstr_length(pwstr: PWSTR) -> usize {
    let mut len: usize = 0;
    let mut ptr: *const WChar = pwstr.0;
    unsafe {
        while *ptr != 0 {
            ptr = ptr.offset(1);
            len += 1;
        }
    }
    len
}
