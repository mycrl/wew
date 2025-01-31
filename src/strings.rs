use std::{
    ffi::{c_char, CString, NulError},
    ptr::null,
};

pub struct PSTR(pub *const c_char);

unsafe impl Send for PSTR {}
unsafe impl Sync for PSTR {}

impl TryFrom<&str> for PSTR {
    type Error = NulError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(CString::new(value)?.into_raw()))
    }
}

impl Drop for PSTR {
    fn drop(&mut self) {
        if !self.0.is_null() {
            drop(unsafe { CString::from_raw(self.0 as *mut c_char) })
        }
    }
}

pub trait StringConvert {
    fn as_pstr(&self) -> PSTR;
}

impl StringConvert for String {
    fn as_pstr(&self) -> PSTR {
        PSTR::try_from(self.as_str()).unwrap()
    }
}

impl<'a> StringConvert for &'a str {
    fn as_pstr(&self) -> PSTR {
        PSTR::try_from(*self).unwrap()
    }
}

impl<'a> StringConvert for Option<&'a str> {
    fn as_pstr(&self) -> PSTR {
        self.map(|it| PSTR::try_from(it).unwrap())
            .unwrap_or(PSTR(null()))
    }
}

pub mod ffi {
    use std::{
        ffi::{c_char, CStr, CString},
        ptr::null,
    };

    pub fn into(value: &str) -> *const c_char {
        CString::new(value).unwrap().into_raw()
    }

    pub fn into_opt(value: Option<&str>) -> *const c_char {
        value
            .map(|it| CString::new(it).unwrap().into_raw() as _)
            .unwrap_or_else(|| null())
    }

    pub fn from(value: *const c_char) -> Option<String> {
        if !value.is_null() {
            unsafe { CStr::from_ptr(value) }
                .to_str()
                .map(|s| s.to_string())
                .ok()
        } else {
            None
        }
    }

    pub fn free(value: *const c_char) {
        if !value.is_null() {
            drop(unsafe { CString::from_raw(value as _) })
        }
    }
}
