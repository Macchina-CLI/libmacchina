//! This module contains all the traits and types for creating a cross-platform API to query
//! different readouts from various operating systems. For each operating system, there must be an implementation of these traits.
#![allow(unused_variables)]

use crate::enums::*;

/**
This trait provides the necessary functions for querying battery statistics from the host
computer. A desktop computer might not be able to provide values such as `percentage` and
`status`, which means a `ReadoutError` can be returned.

# Example

```
// TODO: Add examples
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
This trait provides the necessary functions for querying the user's graphical environment.

# Example

```
// TODO: Add examples
```
*/
pub trait GraphicalReadout {
    /// Creates a new instance of the structure which implements this trait.
    fn new() -> Self;

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
    /// _e.g._ `Sway`
    fn window_manager(&self) -> Result<String, ReadoutError>;

    /// This function should return the name of the used terminal emulator.
    ///
    /// _e.g._ `kitty`
    fn terminal(&self) -> Result<String, ReadoutError>;
}

/**
This trait provides the necessary functions for querying the host's processor.

# Example

```
// TODO: Add examples
```
*/
pub trait ProcessorReadout {
    /// Creates a new instance of the structure which implements this trait.
    fn new() -> Self;

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
}

/**
This trait is used for implementing common functions for reading kernel properties, such as
kernel name and version.

# Example

```
// TODO: Add examples
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
// TODO: Add examples
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
// TODO: Add examples
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
// TODO: Add examples
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
// TODO: Add examples
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

    /// This function should return the name of the physical machine.
    ///
    /// _e.g._ `MacBookPro11,5`
    fn machine(&self) -> Result<String, ReadoutError>;
}

/**
This trait provides the interface for implementing functionality used for querying general
information about the running operating system and current user.

# Example

```
// TODO: Add examples
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

    /// This function should return the name of the OS in a pretty format.
    ///
    /// _e.g._ `macOS 11.2.2 Big Sur`
    fn operating_system(&self) -> Result<String, ReadoutError>;

    /// This function should return the name of the distribution of the operating system.
    ///
    /// _e.g._ `Arch Linux`
    fn distribution(&self) -> Result<String, ReadoutError>;

    /**
    This function should return the currently running shell depending on the `_shorthand` value.

    - If `_shorthand` is `ShellFormat::Relative` the basename of the shell will be returned.

    _e.g._ bash, zsh, etc.

    - If `_shorthand` is `ShellFormat::Absolute` the absolute path of the shell will be returned.

    _e.g._ /bin/bash, /bin/zsh, etc.
    */
    fn shell(&self, _shorthand: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError>;

    /// This function should return the uptime of the OS in seconds.
    fn uptime(&self) -> Result<usize, ReadoutError>;

    /// This function should return the used disk space in a human-readable and desirable format.
    ///
    /// _e.g._ '1.2TB / 2TB'
    fn disk_space(&self) -> Result<(u128, u128), ReadoutError>;
}
