#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use crate::traits::{ReadoutError, ShellFormat, ShellKind};

use std::fs::read_dir;
use std::fs::read_to_string;
use std::io::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs};
use std::{ffi::CStr, path::PathBuf};

use std::ffi::CString;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "android"))]
use sysctl::SysctlError;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "android"))]
impl From<SysctlError> for ReadoutError {
    fn from(e: SysctlError) -> Self {
        ReadoutError::Other(format!("Could not access sysctl: {e:?}"))
    }
}

impl From<std::io::Error> for ReadoutError {
    fn from(e: Error) -> Self {
        ReadoutError::Other(e.to_string())
    }
}

#[cfg(not(any(target_os = "freebsd", target_os = "macos", target_os = "windows")))]
pub(crate) fn uptime() -> Result<usize, ReadoutError> {
    let uptime_buf = fs::read_to_string("/proc/uptime")?;
    let uptime_str = uptime_buf.split_whitespace().next().unwrap();
    let uptime_val = uptime_str.parse::<f64>();

    match uptime_val {
        Ok(s) => Ok(s as usize),
        Err(e) => Err(ReadoutError::Other(format!(
            "Could not convert '{uptime_str}' to a digit: {e:?}",
        ))),
    }
}

#[cfg(not(any(
    feature = "openwrt",
    target_os = "android",
    target_os = "macos",
    target_os = "windows"
)))]
pub(crate) fn desktop_environment() -> Result<String, ReadoutError> {
    let desktop_env = env::var("XDG_CURRENT_DESKTOP").or_else(|_| env::var("DESKTOP_SESSION"));
    match desktop_env {
        Ok(de) => {
            if de.to_lowercase() == "xinitrc" {
                return Err(ReadoutError::Other(String::from(
                    "You appear to be only running a window manager.",
                )));
            }

            Ok(crate::extra::ucfirst(de))
        }
        Err(_) => Err(ReadoutError::Other(String::from(
            "You appear to be only running a window manager.",
        ))),
    }
}

#[cfg(not(any(
    feature = "openwrt",
    target_os = "android",
    target_os = "macos",
    target_os = "windows"
)))]
pub(crate) fn session() -> Result<String, ReadoutError> {
    match env::var("XDG_SESSION_TYPE") {
        Ok(s) => Ok(crate::extra::ucfirst(s)),
        Err(_) => Err(ReadoutError::Other(String::from(
            "No graphical session detected.",
        ))),
    }
}

#[cfg(all(target_os = "linux", not(feature = "openwrt")))]
pub(crate) fn window_manager() -> Result<String, ReadoutError> {
    use crate::winman::*;

    match session()?.as_str() {
        "Wayland" => detect_wayland_window_manager(),
        "X11" => detect_xorg_window_manager(),
        _ => Err(ReadoutError::MetricNotAvailable),
    }
}

#[cfg(any(target_os = "netbsd", target_os = "freebsd"))]
pub(crate) fn resolution() -> Result<String, ReadoutError> {
    use x11rb::connection::Connection;

    let mut resolution: Vec<String> = vec![];
    if let Ok(conn) = x11rb::connect(None) {
        let screens = &conn.0.setup().roots;
        for s in screens {
            let width = s.width_in_pixels;
            let height = s.height_in_pixels;
            resolution.push(width.to_string() + "x" + &height.to_string())
        }

        return Ok(resolution.join(", "));
    }

    Err(ReadoutError::Warning(String::from(
        "Could not open a connection to the X11 server.",
    )))
}

#[cfg(target_family = "unix")]
fn get_passwd_struct() -> Result<*mut libc::passwd, ReadoutError> {
    let uid: libc::uid_t = unsafe { libc::geteuid() };

    // Do not call free on passwd pointer according to man page.
    let passwd = unsafe { libc::getpwuid(uid) };

    if !passwd.is_null() {
        return Ok(passwd);
    }

    Err(ReadoutError::Other(String::from(
        "Unable to read account information.",
    )))
}

#[cfg(target_family = "unix")]
pub(crate) fn username() -> Result<String, ReadoutError> {
    let passwd = get_passwd_struct()?;

    let name = unsafe { CStr::from_ptr((*passwd).pw_name) };
    if let Ok(str) = name.to_str() {
        return Ok(String::from(str));
    }

    Err(ReadoutError::Other(String::from(
        "Unable to read username for the current UID.",
    )))
}

#[cfg(target_family = "unix")]
pub(crate) fn shell(shorthand: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError> {
    match kind {
        ShellKind::Default => {
            let passwd = get_passwd_struct()?;
            let shell_name = unsafe { CStr::from_ptr((*passwd).pw_shell) };

            if let Ok(str) = shell_name.to_str() {
                let path = String::from(str);

                match shorthand {
                    ShellFormat::Relative => {
                        let path = Path::new(&path);
                        if let Some(relative) = path.file_name() {
                            if let Some(shell) = relative.to_str() {
                                return Ok(shell.to_owned());
                            }
                        }
                    }
                    _ => {
                        return Ok(path);
                    }
                }
            }

            Err(ReadoutError::Other(String::from(
                "Unable to read default shell for the current UID.",
            )))
        }
        ShellKind::Current => {
            if cfg!(target_os = "macos") {
                Err(ReadoutError::Other(String::from(
                    "Retrieving the current shell is not supported on macOS.",
                )))
            } else {
                let path = PathBuf::from("/proc")
                    .join(unsafe { libc::getppid() }.to_string())
                    .join("comm");

                if let Ok(shell) = read_to_string(path) {
                    return Ok(shell);
                }

                Err(ReadoutError::Other(String::from(
                    "Unable to read current shell.",
                )))
            }
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn cpu_model_name() -> String {
    use std::io::{BufRead, BufReader};
    let file = fs::File::open("/proc/cpuinfo");
    match file {
        Ok(content) => {
            let reader = BufReader::new(content);
            for line in reader.lines().flatten() {
                if line.starts_with("model name") {
                    return line
                        .replace("model name", "")
                        .replace(':', "")
                        .trim()
                        .to_string();
                }
            }
            String::new()
        }
        Err(_e) => String::new(),
    }
}

#[cfg(any(target_os = "freebsd", target_os = "macos", target_os = "netbsd"))]
pub(crate) fn cpu_usage() -> Result<usize, ReadoutError> {
    let nelem: i32 = 1;
    let mut value: f64 = 0.0;
    let value_ptr: *mut f64 = &mut value;
    let cpu_load = unsafe { libc::getloadavg(value_ptr, nelem) };
    if cpu_load != -1 {
        if let Ok(logical_cores) = cpu_cores() {
            return Ok((value as f64 / logical_cores as f64 * 100.0).round() as usize);
        }
    }
    Err(ReadoutError::Other(format!(
        "getloadavg failed with return code: {cpu_load}"
    )))
}

#[cfg(target_family = "unix")]
pub(crate) fn cpu_cores() -> Result<usize, ReadoutError> {
    Ok(num_cpus::get())
}

#[cfg(target_family = "unix")]
pub(crate) fn cpu_physical_cores() -> Result<usize, ReadoutError> {
    Ok(num_cpus::get_physical())
}

#[cfg(not(any(target_os = "netbsd", target_os = "windows")))]
pub(crate) fn disk_space(path: &Path) -> Result<(u64, u64), ReadoutError> {
    use std::os::unix::ffi::OsStrExt;

    if !path.is_dir() || !path.is_absolute() {
        return Err(ReadoutError::Other(format!(
            "The provided path is not valid: {:?}",
            path
        )));
    }

    let mut s: std::mem::MaybeUninit<libc::statfs> = std::mem::MaybeUninit::uninit();
    let path = CString::new(path.as_os_str().as_bytes())
        .expect("Could not create C string for disk usage path.");

    if unsafe { libc::statfs(path.as_ptr(), s.as_mut_ptr()) } == 0 {
        #[cfg(target_pointer_width = "32")]
        type UInt = u32;
        #[cfg(target_pointer_width = "64")]
        type UInt = u64;

        let stats: libc::statfs = unsafe { s.assume_init() };

        let disk_size = stats.f_blocks * stats.f_bsize as UInt;
        let free = stats.f_bavail as UInt * stats.f_bsize as UInt;

        let used_byte = disk_size - free;
        let disk_size_byte = disk_size;

        #[cfg(target_pointer_width = "32")]
        return Ok((used_byte.into(), disk_size_byte.into()));
        #[cfg(target_pointer_width = "64")]
        return Ok((used_byte, disk_size_byte));
    }

    Err(ReadoutError::Other(String::from(
        "Error while trying to get statfs structure.",
    )))
}

/// Obtain the value of a specified field from `/proc/meminfo` needed to calculate memory usage
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn get_meminfo_value(value: &str) -> u64 {
    use std::io::{BufRead, BufReader};
    let file = fs::File::open("/proc/meminfo");
    match file {
        Ok(content) => {
            let reader = BufReader::new(content);
            for line in reader.lines().flatten() {
                if line.starts_with(value) {
                    let s_mem_kb: String = line.chars().filter(|c| c.is_ascii_digit()).collect();
                    return s_mem_kb.parse::<u64>().unwrap_or(0);
                }
            }
            0
        }
        Err(_e) => 0,
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn logical_address(interface: Option<&str>) -> Result<String, ReadoutError> {
    if let Some(ifname) = interface {
        if let Some(addr) = if_addrs::get_if_addrs()?.into_iter().find_map(|i| {
            if i.name.ne(ifname) {
                return None;
            }

            if i.addr.is_loopback() {
                return None;
            }

            if let if_addrs::IfAddr::V4(v4_addr) = i.addr {
                return Some(v4_addr);
            }

            None
        }) {
            return Ok(addr.ip.to_string());
        };
    }
    Err(ReadoutError::Other(String::from(
        "Unable to get local IPv4 address.",
    )))
}

pub(crate) fn count_cargo() -> Option<usize> {
    let bin = home::cargo_home().ok()?.join("bin");
    let read_dir = read_dir(bin).ok()?;

    match read_dir.count() {
        0 => None,
        pkgs => Some(pkgs),
    }
}
