use std::env;

fn build_windows() {
    #[cfg(windows)]
    windows::build!(
        Windows::Win32::WindowsProgramming::{GetUserNameA, GetComputerNameExA, GetTickCount64},
        Windows::Win32::WindowsAndMessaging::MessageBoxA,
        Windows::Win32::SystemServices::{
            GlobalMemoryStatusEx, GetSystemPowerStatus
        },
    );
}

fn build_macos() {
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=IOKit");
}

fn main() {
    match env::var("CARGO_CFG_TARGET_OS").as_ref().map(|x| &**x) {
        Ok("macos") => build_macos(),
        Ok("windows") => build_windows(),
        _ => {}
    }
}
