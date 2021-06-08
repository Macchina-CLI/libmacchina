use std::env;

fn build_windows() {
    #[cfg(windows)]
    windows::build!(
        windows::win32::windows_programming::{GetUserNameA, GetComputerNameExA, GetTickCount64},
        windows::win32::system_services::{GlobalMemoryStatusEx, GetSystemPowerStatus},
    );
}

fn build_linux_netbsd() {
    // The user can specify the path to the X11 library
    // or we can search a hardcoded directory.
    #[cfg(target_os = "netbsd")]
    if let Some(x11_dir) = option_env!("LIBMAC_X11_LIB_PATH") {
        println!("cargo:rustc-link-search=native={}", x11_dir);
    } else {
        println!("cargo:rustc-link-search=native={}", "/usr/X11R7/lib");
    }

    #[cfg(any(target_os = "linux", target_os = "netbsd"))]
    match pkg_config::probe_library("x11") {
        Ok(_) => {
            if cfg!(target_os = "netbsd") {
                println!("cargo:rustc-link-lib=static=xcb");
                println!("cargo:rustc-link-lib=static=Xau");
                println!("cargo:rustc-link-lib=static=Xdmcp");
                println!("cargo:rustc-link-lib=static=X11");
            } else if cfg!(target_os = "linux"){
                println!("cargo:rustc-link-lib=X11");
            }

            println!("cargo:rustc-cfg=feature=\"xserver\"");
        }
        Err(_) => println!("X11 not present"),
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
