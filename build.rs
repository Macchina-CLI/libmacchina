use std::env;

#[cfg(feature = "version")]
fn commit_hash() {
    use vergen::{Config, ShaKind};

    let mut config = Config::default();
    *config.git_mut().sha_kind_mut() = ShaKind::Short;

    if let Err(e) = vergen::vergen(config) {
        eprintln!("{}", e);
    }
}

fn build_macos() {
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=IOKit");
}

fn main() {
    match env::var("CARGO_CFG_TARGET_OS").as_ref().map(|x| &**x) {
        Ok("macos") => build_macos(),
        Ok("netbsd") => {}
        _ => {}
    }

    #[cfg(feature = "version")]
    commit_hash()
}
