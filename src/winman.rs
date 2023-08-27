//! This module provides a set of functions that detect the name of the window manager the host is
//! running.

use crate::extra;
use crate::traits::ReadoutError;

use std::process::{Command, Stdio};

#[cfg(target_os = "linux")]
use wayland_sys::{client::*, ffi_dispatch};

#[cfg(target_os = "linux")]
use nix::sys::socket::{sockopt, GetSockOpt};

#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;

#[cfg(target_os = "linux")]
pub fn detect_wayland_window_manager() -> Result<String, ReadoutError> {
    if !is_lib_available() {
        return Err(ReadoutError::MetricNotAvailable);
    }

    let display_ptr = unsafe {
        ffi_dispatch!(
            wayland_client_handle(),
            wl_display_connect,
            ::std::ptr::null()
        )
    };

    if display_ptr.is_null() {
        return Err(ReadoutError::MetricNotAvailable);
    }

    let display_fd =
        unsafe { ffi_dispatch!(wayland_client_handle(), wl_display_get_fd, display_ptr) }
            .as_raw_fd();

    let pid = sockopt::PeerCredentials
        .get(display_fd)
        .map_err(|_| ReadoutError::MetricNotAvailable)?
        .pid();

    Ok(std::fs::read_to_string(format!("/proc/{}/comm", pid))?)
}

pub fn detect_xorg_window_manager() -> Result<String, ReadoutError> {
    if extra::which("xprop") {
        let xprop_id = Command::new("xprop")
            .args(["-root", "-notype", "_NET_SUPPORTING_WM_CHECK"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"xprop\" process");

        let xprop_id_output = xprop_id
            .wait_with_output()
            .expect("ERROR: failed to wait for \"xprop\" process to exit");

        let window_manager_id_info = String::from_utf8(xprop_id_output.stdout)
            .expect("ERROR: \"xprop -root -notype _NET_SUPPORTING_WM_CHECK\" process stdout was not valid UTF-8");

        let window_manager_id = window_manager_id_info.split(' ').last().unwrap_or_default();

        let xprop_property = Command::new("xprop")
            .args([
                "-id",
                window_manager_id,
                "-notype",
                "-len",
                "25",
                "-f",
                "_NET_WM_NAME",
                "8t",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"xprop\" process");

        let xprop_property_output = xprop_property
            .wait_with_output()
            .expect("ERROR: failed to wait for \"xprop\" process to exit");

        let window_manager_name_info = String::from_utf8(xprop_property_output.stdout)
            .unwrap_or_else(|_| {
                panic!(
                    "ERROR: \"xprop -id {window_manager_id} -notype -len 25
                                       -f _NET_WM_NAME 8t\" process stdout was not valid UTF-8"
                )
            });

        if let Some(line) = window_manager_name_info
            .lines()
            .find(|line| line.starts_with("_NET_WM_NAME"))
        {
            return Ok(line
                .split_once('=')
                .unwrap_or_default()
                .1
                .trim()
                .replace(['\"', '\''], ""));
        };
    }

    if extra::which("wmctrl") {
        let wmctrl = Command::new("wmctrl")
            .arg("-m")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"wmctrl\" process");

        let wmctrl_out = wmctrl
            .stdout
            .expect("ERROR: failed to open \"wmctrl\" stdout");

        let head = Command::new("head")
            .args(["-n", "1"])
            .stdin(Stdio::from(wmctrl_out))
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"head\" process");

        let output = head
            .wait_with_output()
            .expect("ERROR: failed to wait for \"head\" process to exit");

        let window_manager = String::from_utf8(output.stdout)
            .expect("ERROR: \"wmctrl -m | head -n1\" process stdout was not valid UTF-8");

        let winman_name =
            extra::pop_newline(String::from(window_manager.replace("Name:", "").trim()));

        if winman_name == "N/A" || winman_name.is_empty() {
            return Err(ReadoutError::Other(
                "Window manager not available — perhaps it's not EWMH-compliant.".to_string(),
            ));
        }

        return Ok(winman_name);
    }

    Err(ReadoutError::Other(
        "\"wmctrl\" must be installed to display your window manager.".to_string(),
    ))
}
