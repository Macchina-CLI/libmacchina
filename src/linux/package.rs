use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use crate::traits::PackageReadout;
use crate::enums::PackageManager;
use crate::extra::get_entries;
use crate::extra::path_extension;
use crate::shared;
use crate::extra;

pub struct LinuxPackageReadout;

impl PackageReadout for LinuxPackageReadout {
    fn new() -> Self {
        LinuxPackageReadout
    }

    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();
        let mut home = PathBuf::new();

        // Acquire the value of HOME early on to avoid
        // doing it multiple times.
        if let Ok(path) = std::env::var("HOME") {
            home = PathBuf::from(path);
        }

        if let Some(c) = LinuxPackageReadout::count_pacman() {
            packages.push((PackageManager::Pacman, c));
        }

        if let Some(c) = LinuxPackageReadout::count_dpkg() {
            packages.push((PackageManager::Dpkg, c));
        }

        if let Some(c) = LinuxPackageReadout::count_rpm() {
            packages.push((PackageManager::Rpm, c));
        }

        if let Some(c) = LinuxPackageReadout::count_portage() {
            packages.push((PackageManager::Portage, c));
        }

        if let Some(c) = LinuxPackageReadout::count_cargo() {
            packages.push((PackageManager::Cargo, c));
        }

        if let Some(c) = LinuxPackageReadout::count_xbps() {
            packages.push((PackageManager::Xbps, c));
        }

        if let Some(c) = LinuxPackageReadout::count_eopkg() {
            packages.push((PackageManager::Eopkg, c));
        }

        if let Some(c) = LinuxPackageReadout::count_apk() {
            packages.push((PackageManager::Apk, c));
        }

        if let Some(c) = LinuxPackageReadout::count_flatpak(&home) {
            packages.push((PackageManager::Flatpak, c));
        }

        if let Some(c) = LinuxPackageReadout::count_snap() {
            packages.push((PackageManager::Snap, c));
        }

        if let Some(c) = LinuxPackageReadout::count_homebrew(&home) {
            packages.push((PackageManager::Homebrew, c));
        }

        packages
    }
}

impl LinuxPackageReadout {
    /// Returns the number of installed packages for systems
    /// that utilize `rpm` as their package manager.
    fn count_rpm() -> Option<usize> {
        // Return the number of installed packages using sqlite (~1ms)
        // as directly calling rpm or dnf is too expensive (~500ms)
        let db = "/var/lib/rpm/rpmdb.sqlite";
        if !Path::new(db).is_file() {
            return None;
        }

        let connection = sqlite::open(db);
        if let Ok(con) = connection {
            let statement = con.prepare("SELECT COUNT(*) FROM Installtid");
            if let Ok(mut s) = statement {
                if s.next().is_ok() {
                    return match s.read::<Option<i64>>(0) {
                        Ok(Some(count)) => Some(count as usize),
                        _ => None,
                    };
                }
            }
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that utilize `pacman` as their package manager.
    fn count_pacman() -> Option<usize> {
        let pacman_dir = Path::new("/var/lib/pacman/local");
        if pacman_dir.is_dir() {
            if let Ok(read_dir) = read_dir(pacman_dir) {
                return Some(read_dir.count());
            };
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that utilize `eopkg` as their package manager.
    fn count_eopkg() -> Option<usize> {
        let eopkg_dir = Path::new("/var/lib/eopkg/package");
        if eopkg_dir.is_dir() {
            if let Ok(read_dir) = read_dir(eopkg_dir) {
                return Some(read_dir.count());
            };
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that utilize `portage` as their package manager.
    fn count_portage() -> Option<usize> {
        let pkg_dir = Path::new("/var/db/pkg");
        if pkg_dir.exists() {
            return Some(walkdir::WalkDir::new(pkg_dir).into_iter().count());
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that utilize `dpkg` as their package manager.
    fn count_dpkg() -> Option<usize> {
        let dpkg_dir = Path::new("/var/lib/dpkg/info");

        get_entries(dpkg_dir).map(|entries| {
            entries
                .iter()
                .filter(|x| extra::path_extension(x).unwrap_or_default() == "list")
                .into_iter()
                .count()
        })
    }

    /// Returns the number of installed packages for systems
    /// that have `homebrew` installed.
    fn count_homebrew(home: &Path) -> Option<usize> {
        let mut base = home.join(".linuxbrew");
        if !base.is_dir() {
            base = PathBuf::from("/home/linuxbrew/.linuxbrew");
        }

        match read_dir(base.join("Cellar")) {
            // subtract 1 as ${base}/Cellar contains a ".keepme" file
            Ok(dir) => Some(dir.count() - 1),
            Err(_) => None,
        }
    }

    /// Returns the number of installed packages for systems
    /// that utilize `xbps` as their package manager.
    fn count_xbps() -> Option<usize> {
        if !extra::which("xbps-query") {
            return None;
        }

        let xbps_output = Command::new("xbps-query")
            .arg("-l")
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        extra::count_lines(
            String::from_utf8(xbps_output.stdout)
                .expect("ERROR: \"xbps-query -l\" output was not valid UTF-8"),
        )
    }

    /// Returns the number of installed packages for systems
    /// that utilize `apk` as their package manager.
    fn count_apk() -> Option<usize> {
        if !extra::which("apk") {
            return None;
        }

        let apk_output = Command::new("apk")
            .arg("info")
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        extra::count_lines(
            String::from_utf8(apk_output.stdout)
                .expect("ERROR: \"apk info\" output was not valid UTF-8"),
        )
    }

    /// Returns the number of installed packages for systems
    /// that have `cargo` installed.
    fn count_cargo() -> Option<usize> {
        shared::count_cargo()
    }

    /// Returns the number of installed packages for systems
    /// that have `flatpak` installed.
    fn count_flatpak(home: &Path) -> Option<usize> {
        let global_flatpak_dir = Path::new("/var/lib/flatpak/app");
        let user_flatpak_dir = home.join(".local/share/flatpak/app");

        match (read_dir(global_flatpak_dir), read_dir(user_flatpak_dir)) {
            (Ok(g), Ok(u)) => Some(g.count() + u.count()),
            (Ok(g), _) => Some(g.count()),
            (_, Ok(u)) => Some(u.count()),
            _ => None,
        }
    }

    /// Returns the number of installed packages for systems
    /// that have `snap` installed.
    fn count_snap() -> Option<usize> {
        let snap_dir = Path::new("/var/lib/snapd/snaps");
        if let Some(entries) = get_entries(snap_dir) {
            return Some(
                entries
                    .iter()
                    .filter(|&x| path_extension(x).unwrap_or_default() == "snap")
                    .into_iter()
                    .count(),
            );
        }

        None
    }
}
