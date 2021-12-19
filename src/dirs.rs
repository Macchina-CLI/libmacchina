use crate::extra::read_lines;
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
        for line in lines.flatten() {
            if line.starts_with("PKG_DBDIR") {
                let pkg_db = PathBuf::from(line.split('=').nth(1).unwrap().trim().to_string());
                if pkg_db.is_dir() {
                    return Some(pkg_db);
                }
            }

            continue;
        }
    }

    Some(PathBuf::from("/usr/pkg/pkgdb"))
}

/// Returns the value of LOCALBASE if exists or a default if not.
pub fn localbase_dir() -> Option<PathBuf> {
    if let Ok(lines) = read_lines("/etc/mk.conf") {
        for line in lines.flatten() {
            if line.starts_with("LOCALBASE") {
                let localbase = PathBuf::from(line.split('=').nth(1).unwrap().trim().to_string());
                if localbase.is_dir() {
                    return Some(localbase);
                }
            }

            continue;
        }
    }

    Some(PathBuf::from("/usr/pkg"))
}
