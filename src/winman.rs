//! This module provides a set of functions that detect the name of the window manager the host is
//! running.

use crate::extra;
use crate::traits::ReadoutError;

use std::process::{Command, Stdio};

#[cfg(target_os = "linux")]
/// Detects if the host is using Sway.
pub fn is_running_sway() -> bool {
    if let Ok(socket) = std::env::var("SWAYSOCK") {
        if std::path::PathBuf::from(socket).exists() {
            return true;
        }
    }

    false
}

#[cfg(target_os = "linux")]
/// Detects if the host is using Wayfire.
pub fn is_running_wayfire() -> bool {
    if let Ok(config) = std::env::var("WAYFIRE_CONFIG_FILE") {
        if std::path::PathBuf::from(config).exists() {
            return true;
        }
    }

    false
}

#[cfg(target_os = "linux")]
/// Detects if the host is using Qtile.
pub fn is_running_qtile() -> bool {
    if let Some(cache) = dirs::cache_dir() {
        if let Ok(display) = std::env::var("WAYLAND_DISPLAY") {
            let socket = cache
                .join("qtile")
                .join("qtilesocket.".to_owned() + &display);

            if socket.exists() {
                return true;
            }
        }
    }

    false
}

#[cfg(target_os = "linux")]
pub fn detect_wayland_window_manager() -> Result<String, ReadoutError> {
    if is_running_sway() {
        Ok(String::from("Sway"))
    } else if is_running_qtile() {
        Ok(String::from("Qtile"))
    } else if is_running_wayfire() {
        Ok(String::from("Wayfire"))
    } else {
        Err(ReadoutError::Other(String::from("Unknown window manager.")))
    }
}

pub fn detect_xorg_window_manager() -> Result<String, ReadoutError> {
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
