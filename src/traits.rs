//! This module contains all the traits and types for creating a cross-platform API to query
//! different readouts from various operating systems. For each operating system, there must be an implementation of these traits.
#![allow(unused_variables)]

/// This enum contains possible error types when doing sensor & variable readouts.
#[derive(Debug, Clone)]
pub enum ReadoutError {
    /// A specific metric might not be available on all systems (e. g. battery percentage on a
    /// desktop). \
    /// If you encounter this error, it means that the requested value is not available.
    MetricNotAvailable,

    /// The default error for any readout that is not implemented by a particular platform.
    NotImplemented,

    /// A readout for a metric might be available, but fails due to missing dependencies or other
    /// unsatisfied requirements.
    Other(String),

    /// Getting a readout on a specific operating system might not make sense or causes some other
    /// kind of warning. This is not necessarily an error.
    Warning(String),
}

impl ToString for ReadoutError {
    fn to_string(&self) -> String {
        match self {
            ReadoutError::MetricNotAvailable => {
                String::from("Metric is not available on this system.")
            }
            ReadoutError::NotImplemented => {
                String::from("This metric is not available on this platform or is not yet implemented by libmacchina.")
            }
            ReadoutError::Other(s) => s.clone(),
            ReadoutError::Warning(s) => s.clone(),
        }
    }
}

impl From<&ReadoutError> for ReadoutError {
    fn from(r: &ReadoutError) -> Self {
        r.to_owned()
    }
}

/**
This trait provides the necessary functions for querying battery statistics from the host
computer. A desktop computer might not be able to provide values such as `percentage` and
`status`, which means a `ReadoutError` can be returned.

# Example

```
use libmacchina::traits::BatteryReadout;
use libmacchina::traits::ReadoutError;
use libmacchina::traits::BatteryState;

//You can add fields to this struct which will then need to be initialized in the
//BatteryReadout::new() function.
pub struct MacOSBatteryReadout;

impl BatteryReadout for MacOSBatteryReadout {
    fn new() -> Self {
        MacOSBatteryReadout {}
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        //get the battery percentage somehow...
        Ok(100u8) //always fully charged
    }

    fn status(&self) -> Result<BatteryState, ReadoutError> {
        //check if battery is being charged...
        Ok(BatteryState::Charging) //always charging.
    }

    fn health(&self) -> Result<u64, ReadoutError>{
        //check the battery health...
        Ok(100) //totally healtyh
    }
}
```
*/
pub trait BatteryReadout {
    /// Creates a new instance of the structure which implements this trait.
    fn new() -> Self;

    /// This function is used for querying the current battery percentage. The expected value is
    /// a u8 in the range of `0` to `100`.
    fn percentage(&self) -> Result<u8, ReadoutError>;

    /// This function is used for querying the current battery charging state. If the battery is
    /// currently being charged, we expect a return value of `BatteryState::Charging`, otherwise
    /// `BatteryState::Discharging`.
    fn status(&self) -> Result<BatteryState, ReadoutError>;

    /// This function is used for querying the current battery's health in percentage.
    fn health(&self) -> Result<u64, ReadoutError>;
}

/**
This trait is used for implementing common functions for reading kernel properties, such as
kernel name and version.

# Example

```
use libmacchina::traits::KernelReadout;
use libmacchina::traits::ReadoutError;

pub struct MacOSKernelReadout;

impl KernelReadout for MacOSKernelReadout {
    fn new() -> Self {
        MacOSKernelReadout {}
    }

    fn os_release(&self) -> Result<String, ReadoutError> {
        // Get kernel version
        Ok(String::from("20.0.1"))
    }

    fn os_type(&self) -> Result<String, ReadoutError> {
        // Get kernel name
        Ok(String::from("Darwin"))
    }
}
```
*/
pub trait KernelReadout {
    /// Creates a new instance of the structure which implements this trait.
    fn new() -> Self;

    /// This function should return the version of the kernel (e. g. `20.3.0` on macOS for Darwin).
    fn os_release(&self) -> Result<String, ReadoutError>;

    /// This function should return the kernel name as a string (e. g. `Darwin` on macOS).
    fn os_type(&self) -> Result<String, ReadoutError>;

    /// This function is used for getting the kernel name and version in a pretty format.
    fn pretty_kernel(&self) -> Result<String, ReadoutError> {
        let os_type = self.os_type().unwrap_or_default();
        let os_release = self.os_release().unwrap_or_default();

        if !(os_type.is_empty() || os_release.is_empty()) {
            return Ok(format!("{} {}", os_type, os_release));
        }

        Err(ReadoutError::MetricNotAvailable)
    }
}

/**
This trait provides common functions for _querying the current memory state_ of the host device,
most notably `total` and `used`. All other methods exposed by this trait are there in case you're
intending to calculate memory usage on your own.

# Example

```
use libmacchina::traits::MemoryReadout;
use libmacchina::traits::ReadoutError;

pub struct MacOSMemoryReadout;

impl MemoryReadout for MacOSMemoryReadout {
    fn new() -> Self {
        MacOSMemoryReadout {}
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        // Get the total physical memory for the machine
        Ok(512 * 1024) // Return 512mb in kilobytes.
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        // Get the amount of free memory
        Ok(256 * 1024) // Return 256mb in kilobytes.
    }

    fn buffers(&self) -> Result<u64, ReadoutError> {
        // Get the current memory value for buffers
        Ok(64 * 1024) // Return 64mb in kilobytes.
    }

    fn cached(&self) -> Result<u64, ReadoutError> {
        // Get the amount of cached content in memory
        Ok(128 * 1024) // Return 128mb in kilobytes.
    }

    fn reclaimable(&self) -> Result<u64, ReadoutError> {
        // Get the amount of reclaimable memory
        Ok(64 * 1024) // Return 64mb in kilobytes.
    }

    fn used(&self) -> Result<u64, ReadoutError> {
        // Get the currently used memory.
        Ok(256 * 1024) // Return 256mb in kilobytes.
    }
}

```
*/
pub trait MemoryReadout {
    /// Creates a new instance of the structure which implements this trait.
    fn new() -> Self;

    /// This function should return the total available memory in kilobytes.
    fn total(&self) -> Result<u64, ReadoutError>;

    /// This function should return the free available memory in kilobytes.
    fn free(&self) -> Result<u64, ReadoutError>;

    /// This function should return the current memory value for buffers in kilobytes.
    fn buffers(&self) -> Result<u64, ReadoutError>;

    /// This function should return the amount of cached content in memory in kilobytes.
    fn cached(&self) -> Result<u64, ReadoutError>;

    /// This function should return the amount of reclaimable memory in kilobytes.
    fn reclaimable(&self) -> Result<u64, ReadoutError>;

    /// This function should return the amount of currently used memory in kilobytes.
    fn used(&self) -> Result<u64, ReadoutError>;
}

/**
This trait provides an interface to various functions used to _count packages_ on
the host system. Almost all modern operating systems use some kind of package manager.

# Example

```
use libmacchina::traits::{PackageReadout, PackageManager};
use libmacchina::traits::ReadoutError;

pub struct MacOSPackageReadout;

impl PackageReadout for MacOSPackageReadout {
    fn new() -> Self {
        MacOSPackageReadout {}
    }

    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        // Check if homebrew 🍻 is installed and count installed packages...
        vec![(PackageManager::Homebrew, 120)]
    }
}
```
*/
pub trait PackageReadout {
    /// Creates a new instance of the structure which implements this trait.
    fn new() -> Self;

    /// This function should return the number of installed packages.
    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        Vec::new()
    }
}

/**
This trait provides an interface to various networking statistics about the host system.

# Example

```
use libmacchina::traits::NetworkReadout;
use libmacchina::traits::ReadoutError;

pub struct MacOSNetworkReadout;

impl NetworkReadout for MacOSNetworkReadout {
    fn new() -> Self {
        MacOSNetworkReadout {}
    }

    fn tx_bytes(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        todo!()
    }

    fn tx_packets(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        todo!()
    }

    fn rx_bytes(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        todo!()
    }

    fn rx_packets(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        todo!()
    }

    fn logical_address(&self, interface: Option<&str>) -> Result<String, ReadoutError> {
        todo!()
    }

    fn physical_address(&self, interface: Option<&str>) -> Result<String, ReadoutError> {
        todo!()
    }
}
```

*/
pub trait NetworkReadout {
    /// Creates a new instance of the structure which implements this trait.
    fn new() -> Self;

    /// This function should return the number of bytes
    /// transmitted by the interface of the host.
    fn tx_bytes(&self, interface: Option<&str>) -> Result<usize, ReadoutError>;

    /// This function should return the number of packets
    /// transmitted by the interface of the host.
    fn tx_packets(&self, interface: Option<&str>) -> Result<usize, ReadoutError>;

    /// This function should return the number of bytes
    /// received by the interface of the host.
    fn rx_bytes(&self, interface: Option<&str>) -> Result<usize, ReadoutError>;

    /// This function should return the number of packets
    /// received by the interface of the host.
    fn rx_packets(&self, interface: Option<&str>) -> Result<usize, ReadoutError>;

    /// This function should return the logical addess, i.e. _local IPv4/6 address_ of the
    /// specified interface.
    ///
    /// _e.g._ `192.168.1.2`
    fn logical_address(&self, interface: Option<&str>) -> Result<String, ReadoutError>;

    /// This function should return the physical address, i.e. _MAC address_ of the
    /// specified interface.
    ///
    /// _e.g._ `52:9a:d2:d3:b5:fd`
    fn physical_address(&self, interface: Option<&str>) -> Result<String, ReadoutError>;
}

/**
This trait provides the interface for implementing functionality used for getting _product information_
about the host machine.

# Example

```
use libmacchina::traits::ProductReadout;
use libmacchina::traits::ReadoutError;

pub struct MacOSProductReadout;

impl ProductReadout for MacOSProductReadout {
    fn new() -> Self {
        MacOSProductReadout {}
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        Ok(String::from("Apple"))
    }

    fn family(&self) -> Result<String, ReadoutError> {
        Ok(String::from("MacBook Pro"))
    }

    fn product(&self) -> Result<String, ReadoutError> {
        Ok(String::from("MacBookPro16,1"))
    }
}
```
*/
pub trait ProductReadout {
    /// Creates a new instance of the structure which implements this trait.
    fn new() -> Self;

    /// This function should return the vendor name of the host's machine.
    ///
    /// _e.g._ `Lenovo`
    ///
    /// This is set by the machine's manufacturer.
    fn vendor(&self) -> Result<String, ReadoutError>;

    /// This function should return the family name of the host's machine.
    ///
    /// _e.g._ `IdeaPad S540-15IWL GTX`
    ///
    /// This is set by the machine's manufacturer.
    fn family(&self) -> Result<String, ReadoutError>;

    /// This function should return the product name of the host's machine.
    ///
    /// _e.g._ `81SW`
    ///
    /// This is set by the machine's manufacturer.
    fn product(&self) -> Result<String, ReadoutError>;
}

/**
This trait provides the interface for implementing functionality used for querying general
information about the running operating system and current user.

# Example

```
use libmacchina::traits::GeneralReadout;
use libmacchina::traits::ReadoutError;
use libmacchina::traits::ShellFormat;
use libmacchina::traits::ShellKind;

pub struct MacOSGeneralReadout;

impl GeneralReadout for MacOSGeneralReadout {

    fn new() -> Self {
        MacOSGeneralReadout {}
    }

    fn backlight(&self) -> Result<usize, ReadoutError> {
        Ok(100) // Brightness is at its maximum
    }

    fn resolution(&self) -> Result<String, ReadoutError> {
        Ok("1920x1080".to_string())
    }

    fn username(&self) -> Result<String, ReadoutError> {
        //let username = NSUserName();
        Ok(String::from("johndoe"))
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        Ok("supercomputer".to_string())
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        Ok("Arch Linux".to_string())
    }

    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        Ok("Plasma".to_string())
    }

    fn session(&self) -> Result<String, ReadoutError> {
        Ok("Wayland".to_string())
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        Ok("KWin".to_string())
    }

    fn terminal(&self) -> Result<String, ReadoutError> {
        Ok("kitty".to_string())
    }

    fn shell(&self, _shorthand: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError> {
        Ok("bash".to_string())
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        Ok("Intel(R) Core(TM) i5-8265U CPU @ 1.60GHz".to_string())
    }

    fn cpu_usage(&self) -> Result<usize, ReadoutError> {
        Ok(20) //20% CPU usage
    }

    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError> {
        Ok(4)
    }

    fn cpu_cores(&self) -> Result<usize, ReadoutError> {
        Ok(8)
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        Ok(24 * 60 * 60) //1 day
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        Ok("MacBookPro11,5".to_string())
    }

    fn os_name(&self) -> Result<String, ReadoutError> {
        Ok("macOS 11.2.2 Big Sur".to_string())
    }

    fn disk_space(&self) -> Result<(u128, u128), ReadoutError> {
        Ok((50000000,1000000000)) // Used / Total
    }
}

```
*/
pub trait GeneralReadout {
    /// Creates a new instance of the structure which implements this trait.
    fn new() -> Self;

    /// This function should return the backlight (brightness) value of the machine.
    ///
    /// _e.g._ `100`
    fn backlight(&self) -> Result<usize, ReadoutError>;

    /// This function should return the display resolution of the machine.
    ///
    /// _e.g. `1920x1080`
    fn resolution(&self) -> Result<String, ReadoutError>;

    /// This function should return the username of the currently logged on user.
    ///
    /// _e.g._ `johndoe`
    fn username(&self) -> Result<String, ReadoutError>;

    /// This function should return the hostname of the host's computer.
    ///
    /// _e.g._ `supercomputer`
    fn hostname(&self) -> Result<String, ReadoutError>;

    /// This function should return the name of the distribution of the operating system.
    ///
    /// _e.g._ `Arch Linux`
    fn distribution(&self) -> Result<String, ReadoutError>;

    /// This function should return the name of the used desktop environment.
    ///
    /// _e.g._ `Plasma`
    fn desktop_environment(&self) -> Result<String, ReadoutError>;

    /// This function should return the type of session that's in use.
    ///
    /// _e.g._ `Wayland`
    fn session(&self) -> Result<String, ReadoutError>;

    /// This function should return the name of the used window manager.
    ///
    /// _e.g._ `KWin`
    fn window_manager(&self) -> Result<String, ReadoutError>;

    /// This function should return the name of the used terminal emulator.
    ///
    /// _e.g._ `kitty`
    fn terminal(&self) -> Result<String, ReadoutError>;

    /**
    This function should return the currently running shell depending on the `_shorthand` value.

    - If `_shorthand` is `ShellFormat::Relative` the basename of the shell will be returned.

    _e.g._ bash, zsh, etc.

    - If `_shorthand` is `ShellFormat::Absolute` the absolute path of the shell will be returned.

    _e.g._ /bin/bash, /bin/zsh, etc.
    */

    fn shell(&self, _shorthand: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError>;

    /// This function should return the model name of the CPU \
    ///
    /// _e.g._ `Intel(R) Core(TM) i5-8265U CPU @ 1.60GHz`
    fn cpu_model_name(&self) -> Result<String, ReadoutError>;

    /// This function should return the average CPU usage over the last minute.
    fn cpu_usage(&self) -> Result<usize, ReadoutError>;

    /// This function should return the number of physical cores of the host's processor.
    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError>;

    /// This function should return the number of logical cores of the host's processor.
    fn cpu_cores(&self) -> Result<usize, ReadoutError>;

    /// This function should return the uptime of the OS in seconds.
    fn uptime(&self) -> Result<usize, ReadoutError>;

    /// This function should return the name of the physical machine.
    ///
    /// _e.g._ `MacBookPro11,5`
    fn machine(&self) -> Result<String, ReadoutError>;

    /// This function should return the name of the OS in a pretty format.
    ///
    /// _e.g._ `macOS 11.2.2 Big Sur`
    fn os_name(&self) -> Result<String, ReadoutError>;

    /// This function should return the used disk space in a human-readable and desirable format.
    ///
    /// _e.g._ '1.2TB / 2TB'
    fn disk_space(&self) -> Result<(u128, u128), ReadoutError>;
}

/// Holds the possible variants for battery status.
pub enum BatteryState {
    Charging,
    Discharging,
}

impl std::fmt::Display for BatteryState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BatteryState::Charging => {
                write!(f, "Charging")
            }
            BatteryState::Discharging => {
                write!(f, "Discharging")
            }
        }
    }
}

impl From<BatteryState> for &'static str {
    fn from(state: BatteryState) -> &'static str {
        match state {
            BatteryState::Charging => "Charging",
            BatteryState::Discharging => "Discharging",
        }
    }
}

/// The currently running shell is a program, whose path
/// can be _relative_, or _absolute_.
#[derive(Debug)]
pub enum ShellFormat {
    Relative,
    Absolute,
}

#[derive(Debug)]
/// There are two distinct kinds of shells, a so called *"current"* shell, i.e. the shell the user is currently using.
/// And a default shell, i.e. that the user sets for themselves using the `chsh` tool.
pub enum ShellKind {
    Current,
    Default,
}

/// The supported package managers whose packages can be extracted.
pub enum PackageManager {
    Homebrew,
    MacPorts,
    Pacman,
    Portage,
    Dpkg,
    Opkg,
    Xbps,
    Pkgsrc,
    Apk,
    Eopkg,
    Rpm,
    Cargo,
    Flatpak,
    Snap,
    Android,
    Pkg,
    Scoop,
}

impl ToString for PackageManager {
    fn to_string(&self) -> String {
        String::from(match self {
            PackageManager::Homebrew => "Homebrew",
            PackageManager::MacPorts => "MacPorts",
            PackageManager::Pacman => "pacman",
            PackageManager::Portage => "portage",
            PackageManager::Dpkg => "dpkg",
            PackageManager::Opkg => "opkg",
            PackageManager::Xbps => "xbps",
            PackageManager::Pkgsrc => "pkgsrc",
            PackageManager::Apk => "apk",
            PackageManager::Eopkg => "eopkg",
            PackageManager::Rpm => "rpm",
            PackageManager::Cargo => "cargo",
            PackageManager::Flatpak => "flatpak",
            PackageManager::Snap => "snap",
            PackageManager::Android => "Android",
            PackageManager::Pkg => "pkg",
            PackageManager::Scoop => "Scoop",
        })
    }
}
