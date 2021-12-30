use crate::extra;
use crate::extra::get_entries;
use crate::linux::sysinfo;
use crate::shared;
use crate::traits::GeneralReadout;
use crate::enums::{ReadoutError, ShellFormat, ShellKind};
use std::fs;
use std::fs::{read_dir, read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use sysctl::Ctl;
use sysctl::Sysctl;

pub struct LinuxGeneralReadout {
    hostname_ctl: Option<Ctl>,
    sysinfo: sysinfo,
}

impl GeneralReadout for LinuxGeneralReadout {
    fn new() -> Self {
        LinuxGeneralReadout {
            hostname_ctl: Ctl::new("kernel.hostname").ok(),
            sysinfo: sysinfo::new(),
        }
    }

    fn backlight(&self) -> Result<usize, ReadoutError> {
        if let Some(base) = get_entries(Path::new("/sys/class/backlight/")) {
            if let Some(backlight_path) = base.into_iter().next() {
                let max_brightness_path = backlight_path.join("max_brightness");
                let current_brightness_path = backlight_path.join("brightness");

                let max_brightness_value =
                    extra::pop_newline(fs::read_to_string(max_brightness_path)?)
                        .parse::<usize>()
                        .ok();

                let current_brightness_value =
                    extra::pop_newline(fs::read_to_string(current_brightness_path)?)
                        .parse::<usize>()
                        .ok();

                match (current_brightness_value, max_brightness_value) {
                    (Some(c), Some(m)) => {
                        let brightness = c as f64 / m as f64 * 100f64;
                        return Ok(brightness.round() as usize);
                    }
                    _ => {
                        return Err(ReadoutError::Other(String::from(
                            "Error occurred while calculating backlight (brightness) value.",
                        )));
                    }
                }
            }
        }

        Err(ReadoutError::Other(String::from(
            "Could not obtain backlight information.",
        )))
    }

    fn resolution(&self) -> Result<String, ReadoutError> {
        let drm = Path::new("/sys/class/drm");

        if let Some(entries) = get_entries(drm) {
            let mut resolutions: Vec<String> = Vec::new();
            entries.into_iter().for_each(|entry| {
                // Append "modes" to /sys/class/drm/<card>/
                let modes = entry.join("modes");
                if let Ok(file) = File::open(modes) {
                    // Push the resolution to the resolutions vector.
                    if let Some(Ok(res)) = BufReader::new(file).lines().next() {
                        resolutions.push(res);
                    }
                }
            });

            return Ok(resolutions.join(", "));
        }

        Err(ReadoutError::Other(
            "Could not obtain screen resolution from /sys/class/drm".to_string(),
        ))
    }

    fn username(&self) -> Result<String, ReadoutError> {
        shared::username()
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        Ok(self
            .hostname_ctl
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .value_string()?)
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        use os_release::OsRelease;
        let content = OsRelease::new()?;

        if !content.version.is_empty() {
            return Ok(format!("{} {}", content.name, content.version));
        } else if !content.version_id.is_empty() {
            return Ok(format!("{} {}", content.name, content.version_id));
        }

        Ok(content.name)
    }

    fn shell(&self, format: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError> {
        shared::shell(format, kind)
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };

        if ret != -1 {
            return Ok(info.uptime as usize);
        }

        Err(ReadoutError::Other(
            "Something went wrong during the initialization of the sysinfo struct.".to_string(),
        ))
    }

    fn operating_system(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn disk_space(&self) -> Result<(u128, u128), ReadoutError> {
        shared::disk_space(String::from("/"))
    }
}
