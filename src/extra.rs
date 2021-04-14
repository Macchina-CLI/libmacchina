//! This module provides additional functionalities

use std::env;
use std::path::{Path, PathBuf};

/**
This function pops `\n` from the end of a given `String` if it is found.

This can come in handy when reading the contents of a file that might
contain a newline control character at the end of the line.

Files of this kind are very common on GNU/Linux systems.

# Example

```
use libmacchina::extra::pop_newline;

let a = String::from("Foobar\n");
let b = String::from("Foobar");

assert_eq!(pop_newline(a), b);
```
*/
pub fn pop_newline(mut string: String) -> String {
    if string.ends_with('\n') {
        string.pop();
    }

    string
}

/**
This function checks if the given `String` is a valid integer,
returning an error message if the check fails.

# Example

```
use libmacchina::extra::is_int;

let a = String::from("123");
let b = String::from("ABC123");

assert_eq!(is_int(a).is_ok(), true);
assert_eq!(is_int(b).is_ok(), false);

```
*/
pub fn is_int(s: String) -> Result<(), String> {
    if s.chars().all(char::is_numeric) {
        return Ok(());
    }

    Err(String::from("this argument only accepts integers."))
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

This can be used to check if a particular program exists before running a command \
that could return an error in case the program is not installed.

- Returns `true` if a given program is in __PATH__, and `false` if it isn't.

# Example
```
use libmacchina::extra::which;

if which("grep") {
    println!("grep is installed.");
} else {
    println!("grep is not installed.");
}
```
*/
pub fn which<P>(program_name: P) -> bool
where
    P: AsRef<Path>,
{
    let exists = env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(&program_name);
                if full_path.exists() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    });

    exists.is_some()
}

/**
Returns the entries of a given `Path`.

- If `Path` is not a directory, the function will return an empty `Vec`.
*/
pub fn list_dir_entries(path: &Path) -> Vec<PathBuf> {
    let mut directory_entries: Vec<PathBuf> = Vec::new();
    let directory = std::fs::read_dir(path);

    if let Ok(dir) = directory {
        for entry in dir.flatten() {
            directory_entries.push(entry.path())
        }
    }
    directory_entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ucfirst() {
        assert_eq!(ucfirst("testString"), "TestString");
    }

    #[test]
    fn test_is_int() {
        assert_eq!(is_int(String::from("1")).is_ok(), true);
    }

    #[test]
    fn test_pop_newline() {
        assert_eq!(pop_newline(String::from("Haha\n")), "Haha");
    }
}
