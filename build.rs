use std::env;
use std::io;

fn distribution() -> io::Result<String> {
    #[cfg(target_os = "linux")]
    {
        use os_release::OsRelease;
        let content = OsRelease::new()?;
        return Ok(content.name.to_lowercase());
    }
    return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Not a linux system",
    ));
}

fn build_windows() {
    #[cfg(windows)]
    windows::build!(
        windows::win32::windows_programming::{GetUserNameA, GetComputerNameExA, GetTickCount64},
        windows::win32::system_services::{GlobalMemoryStatusEx, GetSystemPowerStatus},
    );
}

fn build_macos() {
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=IOKit");
}

fn build_linux() {
    if let Ok(distro) = distribution() {
        if distro == String::from("openwrt") {
            println!("cargo:rustc-cfg=feature=openwrt");
        }
    }
}

fn main() {
    match env::var("CARGO_CFG_TARGET_OS").as_ref().map(|x| &**x) {
        Ok("macos") => build_macos(),
        Ok("windows") => build_windows(),
        Ok("linux") => build_linux(),
        _ => {}
    }
}
