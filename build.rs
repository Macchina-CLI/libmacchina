use std::env;

fn build_windows() {
    #[cfg(windows)]
    windows::build!(
        windows::win32::windows_programming::{GetUserNameA, GetComputerNameExA, GetTickCount64},
        windows::win32::system_services::{GlobalMemoryStatusEx, GetSystemPowerStatus},
    );
}

fn build_linux_netbsd() {
    #[cfg(any(target_os = "linux", target_os = "netbsd"))]
    if let Ok(_) = pkg_config::probe_library("x11") {
        println!("cargo:rustc-link-lib=dylib=X11");
        println!("cargo:rustc-cfg=feature=\"xserver\"");
    }
}

fn build_macos() {
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=IOKit");
}

fn main() {
    match env::var("CARGO_CFG_TARGET_OS").as_ref().map(|x| &**x) {
        Ok("macos") => build_macos(),
        Ok("windows") => build_windows(),
        Ok("linux") | Ok("netbsd") => build_linux_netbsd(),
        _ => {}
    }
}
