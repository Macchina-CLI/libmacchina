#![allow(dead_code)]
use crate::extra::read_lines;
use std::path::PathBuf;

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

    return Some(PathBuf::from("/usr/pkg/pkgdb"));
}

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

    return Some(PathBuf::from("/usr/pkg"));
}
