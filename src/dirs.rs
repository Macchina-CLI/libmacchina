use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

// https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Returns the value of PKG_DBDIR if exists or a default if not.
pub fn pkgdb_dir() -> Option<PathBuf> {
    if let Ok(lines) = read_lines("/etc/mk.conf") {
        let line = lines.flatten().find(|l| l.starts_with("PKG_DBDIR"));

        if let Some(pkg_dbdir) = line {
            if let Some(value) = pkg_dbdir.split('=').nth(1) {
                return Some(PathBuf::from(value.trim()));
            }
        };
    }

    Some(PathBuf::from("/usr/pkg/pkgdb"))
}

/// Returns the value of LOCALBASE if exists or a default if not.
pub fn localbase_dir() -> Option<PathBuf> {
    if let Ok(lines) = read_lines("/etc/mk.conf") {
        let line = lines.flatten().find(|l| l.starts_with("LOCALBASE"));

        if let Some(pkg_dbdir) = line {
            if let Some(value) = pkg_dbdir.split('=').nth(1) {
                return Some(PathBuf::from(value.trim()));
            }
        };
    }

    Some(PathBuf::from("/usr/pkg"))
}
