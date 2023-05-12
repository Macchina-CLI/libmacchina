//! This module provides additional functionalities
#![allow(dead_code)]

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/**
This function pops `\n` from the end of a given `String` if it is found.

This can come in handy when reading the contents of a file that might
contain a newline control character at the end of the line.

Files of this kind are very common on GNU/Linux systems.
*/
pub fn pop_newline<T>(input: T) -> String
where
    T: std::string::ToString,
{
    let mut output = input.to_string();
    if output.ends_with('\n') {
        output.pop();
    }

    output
}

/// Uppercase the first letter of a `String` or `&str`.
pub fn ucfirst<S: AsRef<str>>(s: S) -> String {
    let mut c = s.as_ref().chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/**
Search all directories in __PATH__ for a program e.g. _ps_, _grep_, etc.

This can be used to check if a particular program exists
before running the command associated with said program.

- Returns `true` if a given program is in __PATH__, and `false` if it isn't.
*/
pub fn which<P>(input: P) -> bool
where
    P: AsRef<Path>,
{
    env::var_os("PATH")
        .and_then(|paths| env::split_paths(&paths).find(|dir| dir.join(&input).is_file()))
        .is_some()
}

// Returns the number of newlines in a buffer
pub fn count_lines<T>(buffer: T) -> Option<usize>
where
    T: std::string::ToString,
{
    let buf = buffer.to_string().trim().to_owned();

    if !buf.is_empty() {
        return Some(buf.as_bytes().iter().filter(|&&c| c == b'\n').count() + 1);
    }

    None
}

/**
Returns the entries of a given `Path`.

- If `Path` is not a directory, the function will return an empty `Vec`.
*/
pub fn get_entries(path: &Path) -> Option<Vec<PathBuf>> {
    if let Ok(dir) = std::fs::read_dir(path) {
        let mut entries: Vec<PathBuf> = Vec::new();
        dir.flatten().for_each(|x| entries.push(x.path()));
        return Some(entries);
    }

    None
}

/// Returns the extension of a given path.
pub fn path_extension(path: &Path) -> Option<&str> {
    path.extension().and_then(OsStr::to_str)
}

pub fn common_shells() -> [&'static str; 10] {
    [
        "sh", "su", "nu", "bash", "fish", "dash", "tcsh", "zsh", "ksh", "csh",
    ]
}

#[cfg(test)]
#[cfg(not(target_os = "netbsd"))]
mod tests {
    use super::*;

    #[test]
    fn test_ucfirst() {
        assert_eq!(ucfirst("lorem"), "Lorem");
        assert_eq!(ucfirst("Ipsum"), "Ipsum");
    }

    #[test]
    fn test_pop_newline() {
        assert_eq!(pop_newline(String::from("Lorem ipsum\n")), "Lorem ipsum");
    }

    #[test]
    fn test_path_extension() {
        assert_eq!(path_extension(Path::new("foo.rs")).unwrap(), "rs");
        assert!(path_extension(Path::new("bar"))
            .unwrap_or_default()
            .is_empty());
    }

    #[test]
    #[cfg(not(feature = "openwrt"))]
    fn test_which() {
        assert!(which("sh"));
        assert!(!which("not_a_real_command"));
    }
}
