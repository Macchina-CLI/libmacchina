use std::env;
use vergen::{Config, ShaKind};
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

fn main() {
    match env::var("CARGO_CFG_TARGET_OS").as_ref().map(|x| &**x) {
        Ok("macos") => build_macos(),
        Ok("windows") => build_windows(),
        Ok("netbsd") => return,
        _ => {}
    }

    let mut config = Config::default();
    *config.git_mut().sha_kind_mut() = ShaKind::Short;

    if let Err(e) = vergen::vergen(config) {
        eprintln!("{}", e);
    }
}
