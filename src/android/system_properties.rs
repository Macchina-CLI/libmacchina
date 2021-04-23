#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
use std::os::raw::c_char;
use std::ffi::{CStr, CString};

include!(concat!(env!("OUT_DIR"), "/system_properties.rs"));

fn to_string_safe(param: *mut c_char) -> String {
    unsafe { CStr::from_ptr(param).to_string_lossy().into_owned() }
}

// Takes a property name, and returns its value.
pub fn getprop<T>(name: T) -> Option<String>
where
    T: ToString,
{
    let name = name.to_string();
    if !name.is_ascii() {
        return None;
    }
    // Property name
    let __name: *const c_char = CString::new(name).unwrap().into_raw();
    // Property value
    let mut __value: *mut c_char = CString::new("").unwrap().into_raw();
    // making them mut / const doesn't matter in rust.
    // I'm keeping them like that since it is idiomatic.

    let ret = unsafe { __system_property_get(__name, __value) };

    if ret == -1 {
        None
    } else {
        Some(to_string_safe(__value))
    }
}
