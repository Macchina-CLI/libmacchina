use std::fs::read_dir;

pub(crate) fn count_cargo() -> Option<usize> {
    let bin = home::cargo_home().ok()?.join("bin");
    let read_dir = read_dir(bin).ok()?;

    match read_dir.count() {
        0 => None,
        pkgs => Some(pkgs),
    }
}
