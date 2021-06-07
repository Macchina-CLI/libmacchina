// X11 (Xlib) Bindings
use std::os::raw::{c_char, c_int};
pub enum _XDisplay {}
type Display = _XDisplay;

extern "C" {
    pub fn XOpenDisplay(_1: *const c_char) -> *mut Display;
    pub fn XCloseDisplay(_1: *mut Display) -> c_int;
    pub fn XDefaultScreen(_1: *mut Display) -> c_int;
    pub fn XDisplayHeight(_2: *mut Display, _1: c_int) -> c_int;
    pub fn XDisplayWidth(_2: *mut Display, _1: c_int) -> c_int;
}
