#![allow(dead_code)]
#![allow(unused_imports)]

use crate::traits::{ReadoutError, ShellFormat};

use crate::extra;
use std::io::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs};
use std::{ffi::CStr, path::PathBuf};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use sysctl::SysctlError;

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl From<SysctlError> for ReadoutError {
    fn from(e: SysctlError) -> Self {
        ReadoutError::Other(format!("Could not access system control: {:?}", e))
    }
}

impl From<std::io::Error> for ReadoutError {
    fn from(e: Error) -> Self {
        ReadoutError::Other(e.to_string())
    }
}

#[cfg(any(target_os = "linux", target_os = "netbsd"))]
pub(crate) fn uptime() -> Result<usize, ReadoutError> {
    let uptime_file_text = fs::read_to_string("/proc/uptime")?;
    let uptime_text = uptime_file_text.split_whitespace().next().unwrap();
    let parsed_uptime = uptime_text.parse::<f64>();

    match parsed_uptime {
        Ok(s) => Ok(s as usize),
        Err(e) => Err(ReadoutError::Other(format!(
            "Could not convert '{}' to a digit: {:?}",
            uptime_text, e
        ))),
    }
}

/// Read desktop environment name from `DESKTOP_SESSION` environment variable
/// or from the fallback environment variable `XDG_CURRENT_DESKTOP`
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
pub(crate) fn desktop_environment() -> Result<String, ReadoutError> {
    let desktop_env = env::var("DESKTOP_SESSION").or_else(|_| env::var("XDG_CURRENT_DESKTOP"));
    match desktop_env {
        Ok(de) => {
            if de.contains('/') {
                return Ok(extra::ucfirst(basename(de)));
            }
            return Ok(extra::ucfirst(de));
        }
        Err(_) => Err(ReadoutError::Other(format!(
            "You're running solely a Window Manager."
        ))),
    }
}

/// This function should return the basename of a path
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
fn basename(mut path: String) -> String {
    let last_occurence_index = path.rfind('/').unwrap() + 1;
    path.replace_range(0..last_occurence_index, "");
    path
}

/// Read window manager using `wmctrl -m | grep Name:`
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
pub(crate) fn window_manager() -> Result<String, ReadoutError> {
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
            .args(&["-n", "1"])
            .stdin(Stdio::from(wmctrl_out))
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"head\" process");

        let output = head
            .wait_with_output()
            .expect("ERROR: failed to wait for \"head\" process to exit");

        let window_manager = String::from_utf8(output.stdout)
            .expect("ERROR: \"wmctrl -m | head -n1\" process stdout was not valid UTF-8");

        let window_man_name =
            extra::pop_newline(String::from(window_manager.replace("Name:", "").trim()));
        if window_man_name == "N/A" || window_man_name.is_empty() {
            return Err(ReadoutError::MetricNotAvailable);
        }
        return Ok(window_man_name);
    }

    Err(ReadoutError::Other(format!(
        "\"wmctrl\" must be installed to view your Window Manager."
    )))
}

/// Read current terminal name using `ps`
#[cfg(target_family = "unix")]
pub(crate) fn terminal() -> Result<String, ReadoutError> {
    //  ps -p $(ps -p $$ -o ppid=) o comm=
    //  $$ doesn't work natively in rust but its value can be
    //  accessed through libc::getppid()
    //  libc::getppid(): is always successful.
    let ppid = Command::new("ps")
        .arg("-p")
        .arg(unsafe { libc::getppid() }.to_string())
        .arg("-o")
        .arg("ppid=")
        .output()
        .expect("ERROR: failed to start \"ps\" process");

    let terminal_ppid = String::from_utf8(ppid.stdout)
        .expect("ERROR: \"ps\" process stdout was not valid UTF-8")
        .trim()
        .to_string();

    let name = Command::new("ps")
        .arg("-p")
        .arg(terminal_ppid)
        .arg("-o")
        .arg("comm=")
        .output()
        .expect("ERROR: failed to start \"ps\" output");

    let terminal_name = extra::ucfirst(
        String::from_utf8(name.stdout)
            .expect("ERROR: \"ps\" process stdout was not valid UTF-8")
            .trim(),
    );

    if terminal_name.is_empty() {
        return Err(ReadoutError::Other(String::from(
            "Terminal name was empty.",
        )));
    }

    Ok(terminal_name)
}

#[cfg(target_family = "unix")]
fn get_passwd_struct() -> Result<*mut libc::passwd, ReadoutError> {
    let uid: libc::uid_t = unsafe { libc::geteuid() };

    //do not call free on passwd pointer according to man page.
    let passwd = unsafe { libc::getpwuid(uid) };

    if passwd != std::ptr::null_mut() {
        return Ok(passwd);
    }

    Err(ReadoutError::Other(String::from(
        "Reading the account information failed.",
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
pub(crate) fn shell(shorthand: ShellFormat) -> Result<String, ReadoutError> {
    let passwd = get_passwd_struct()?;

    let shell_name = unsafe { CStr::from_ptr((*passwd).pw_shell) };
    if let Ok(str) = shell_name.to_str() {
        let path = String::from(str);

        match shorthand {
            ShellFormat::Relative => {
                let path = Path::new(&path);
                return Ok(path.file_stem().unwrap().to_str().unwrap().into());
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

/// Read processor information from `/proc/cpuinfo`
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
pub(crate) fn cpu_model_name() -> String {
    use std::io::{BufRead, BufReader};
    let file = fs::File::open("/proc/cpuinfo");
    match file {
        Ok(content) => {
            let reader = BufReader::new(content);
            for line in reader.lines() {
                if let Ok(l) = line {
                    if l.starts_with("model name") {
                        return l
                            .replace("model name", "")
                            .replace(":", "")
                            .trim()
                            .to_string();
                    }
                }
            }
            return String::new();
        }
        Err(_e) => String::new(),
    }
}

/// Obtain the value of a specified field from `/proc/meminfo` needed to calculate memory usage
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
pub(crate) fn get_meminfo_value(value: &str) -> u64 {
    use std::io::{BufRead, BufReader};
    let file = fs::File::open("/proc/meminfo");
    match file {
        Ok(content) => {
            let reader = BufReader::new(content);
            for line in reader.lines() {
                if let Ok(l) = line {
                    if l.starts_with(value) {
                        let s_mem_kb: String = l.chars().filter(|c| c.is_digit(10)).collect();
                        return s_mem_kb.parse::<u64>().unwrap_or(0);
                    }
                }
            }
            0
        }
        Err(_e) => 0,
    }
}

pub(crate) fn local_ip() -> Result<String, ReadoutError> {
    if let Some(s) = local_ipaddress::get() {
        return Ok(s);
    } else {
        Err(ReadoutError::Other(String::from(
            "Unable to get local IP address.",
        )))
    }
}

#[cfg(any(target_family = "unix", target_os = "windows"))]
pub(crate) fn count_cargo() -> Option<usize> {
    use std::fs::read_dir;
    if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        let cargo_bin = PathBuf::from(cargo_home).join("bin");
        if cargo_bin.exists() {
            match read_dir(cargo_bin) {
                Ok(read_dir) => return Some(read_dir.count()),
                _ => (),
            };
        }
        return None;
    }
    None
}
