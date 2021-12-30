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

/// Holds the possible variants for battery status.
pub enum BatteryState {
    Charging,
    Discharging,
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
