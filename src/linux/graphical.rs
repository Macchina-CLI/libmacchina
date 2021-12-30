use std::fs::{File, read_dir, read_to_string};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use crate::traits::GraphicalReadout;
use crate::enums::ReadoutError;
use crate::extra::get_entries;
use crate::extra::path_extension;
use crate::shared;
use crate::extra;

pub struct LinuxGraphicalReadout;

#[cfg(feature = "graphical")]
impl GraphicalReadout for LinuxGraphicalReadout {
    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        shared::desktop_environment()
    }

    fn session(&self) -> Result<String, ReadoutError> {
        shared::session()
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        shared::window_manager()
    }

    fn terminal(&self) -> Result<String, ReadoutError> {
        // This function returns the PPID of a given PID:
        //  - The file used to extract this data: /proc/<pid>/status
        //  - This function parses and returns the value of the ppid line.
        fn get_parent(pid: i32) -> i32 {
            let process_path = PathBuf::from("/proc").join(pid.to_string()).join("status");
            let file = File::open(process_path);
            match file {
                Ok(content) => {
                    let reader = BufReader::new(content);
                    for line in reader.lines().flatten() {
                        if line.to_uppercase().starts_with("PPID") {
                            let s_mem_kb: String =
                                line.chars().filter(|c| c.is_digit(10)).collect();
                            return s_mem_kb.parse::<i32>().unwrap_or(-1);
                        }
                    }

                    -1
                }

                Err(_) => -1,
            }
        }

        // This function returns the name associated with a given PPID
        fn terminal_name() -> String {
            let mut terminal_pid = get_parent(unsafe { libc::getppid() });

            let path = PathBuf::from("/proc")
                .join(terminal_pid.to_string())
                .join("comm");

            // The below loop will traverse /proc to find the
            // terminal inside of which the user is operating
            if let Ok(mut terminal_name) = read_to_string(path) {
                // Any command_name we find that matches
                // one of the elements within this table
                // is effectively ignored
                while extra::common_shells().contains(&terminal_name.replace('\n', "").as_str()) {
                    let ppid = get_parent(terminal_pid);
                    terminal_pid = ppid;

                    let path = PathBuf::from("/proc").join(ppid.to_string()).join("comm");

                    if let Ok(comm) = read_to_string(path) {
                        terminal_name = comm;
                    }
                }

                return terminal_name;
            }

            String::new()
        }

        let terminal = terminal_name();

        if terminal.is_empty() {
            return Err(ReadoutError::Other(
                "Querying terminal information failed".to_string(),
            ));
        }

        Ok(terminal)
    }
}

