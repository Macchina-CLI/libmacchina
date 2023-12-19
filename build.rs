use std::{env, error::Error};

fn build_macos() {
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=IOKit");
    println!("cargo:rustc-link-lib=framework=CoreVideo");
    println!("cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks");
    println!("cargo:rustc-link-lib=framework=DisplayServices");
}

fn main() -> Result<(), Box<dyn Error>> {
    match env::var("CARGO_CFG_TARGET_OS").as_ref().map(|x| &**x) {
        Ok("macos") => build_macos(),
        Ok("netbsd") => {}
        _ => {}
    }

    #[cfg(feature = "version")]
    vergen::EmitBuilder::builder().emit()?;

    Ok(())
}
