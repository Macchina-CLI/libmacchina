use crate::enums::BatteryState;
use crate::enums::ReadoutError;
use crate::extra;
use crate::extra::get_entries;
use crate::linux::ffi::sysinfo;
use crate::traits::BatteryReadout;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub struct LinuxBatteryReadout;

impl BatteryReadout for LinuxBatteryReadout {
    fn new() -> Self {
        LinuxBatteryReadout
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        if let Some(entries) = get_entries(Path::new("/sys/class/power_supply")) {
            let dirs: Vec<PathBuf> = entries
                .into_iter()
                .filter(|x| {
                    !x.components()
                        .last()
                        .unwrap()
                        .as_os_str()
                        .to_string_lossy()
                        .starts_with("ADP")
                })
                .collect();

            if let Some(battery) = dirs.first() {
                let path_to_capacity = battery.join("capacity");
                let percentage_text = extra::pop_newline(fs::read_to_string(path_to_capacity)?);
                let percentage_parsed = percentage_text.parse::<u8>();

                match percentage_parsed {
                    Ok(p) => return Ok(p),
                    Err(e) => {
                        return Err(ReadoutError::Other(format!(
                            "Could not parse the value '{}' into a digit: {:?}",
                            percentage_text, e
                        )))
                    }
                };
            }
        };

        Err(ReadoutError::Other("No batteries detected.".to_string()))
    }

    fn status(&self) -> Result<BatteryState, ReadoutError> {
        if let Some(entries) = get_entries(Path::new("/sys/class/power_supply")) {
            let dirs: Vec<PathBuf> = entries
                .into_iter()
                .filter(|x| {
                    !x.components()
                        .last()
                        .unwrap()
                        .as_os_str()
                        .to_string_lossy()
                        .starts_with("ADP")
                })
                .collect();

            if let Some(battery) = dirs.first() {
                let path_to_status = battery.join("status");
                let status_text =
                    extra::pop_newline(fs::read_to_string(path_to_status)?).to_lowercase();

                match &status_text[..] {
                    "charging" => return Ok(BatteryState::Charging),
                    "discharging" | "full" => return Ok(BatteryState::Discharging),
                    s => {
                        return Err(ReadoutError::Other(format!(
                            "Got an unexpected value \"{}\" reading battery status",
                            s,
                        )))
                    }
                }
            }
        }

        Err(ReadoutError::Other("No batteries detected.".to_string()))
    }

    fn health(&self) -> Result<u64, ReadoutError> {
        if let Some(entries) = get_entries(Path::new("/sys/class/power_supply")) {
            let dirs: Vec<PathBuf> = entries
                .into_iter()
                .filter(|x| {
                    !x.components()
                        .last()
                        .unwrap()
                        .as_os_str()
                        .to_string_lossy()
                        .starts_with("ADP")
                })
                .collect();

            if let Some(battery) = dirs.first() {
                let energy_full =
                    extra::pop_newline(fs::read_to_string(battery.join("energy_full"))?)
                        .parse::<u64>();

                let energy_full_design =
                    extra::pop_newline(fs::read_to_string(battery.join("energy_full_design"))?)
                        .parse::<u64>();

                match (energy_full, energy_full_design) {
                    (Ok(mut ef), Ok(efd)) => {
                        if ef > efd {
                            ef = efd;
                            return Ok(((ef as f64 / efd as f64) * 100_f64) as u64);
                        }

                        return Ok(((ef as f64 / efd as f64) * 100_f64) as u64);
                    }
                    _ => {
                        return Err(ReadoutError::Other(
                            "Error calculating battery health.".to_string(),
                        ))
                    }
                }
            }
        }

        Err(ReadoutError::Other("No batteries detected.".to_string()))
    }
}
