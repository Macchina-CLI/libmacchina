//! Helpers for working with `uv`-managed tools, including directory resolution and counting installed tools.

use std::fs::read_dir;

/// Returns the number of installed tools managed by `uv tool` (similar scope to cargo-installed binaries).
/// Storage layout per uv docs: https://docs.astral.sh/uv/reference/storage/#tools and
/// https://docs.astral.sh/uv/reference/storage/#persistent-data-directory.
/// Priority order of persistent data dir (uv_dirs crate handles resolution):
///   * $XDG_DATA_HOME/uv
///   * $HOME/.local/share/uv
///   * $CWD/.uv (Unix); on Windows: %APPDATA%\uv\data and .\.uv
///
/// The tools are stored under <persistent-data-dir>/tools; each subdir represents one installed tool.
pub(crate) fn count_uv() -> Option<usize> {
    // Priority per uv docs: UV_TOOL_DIR if set; otherwise the persistent state dir (XDG_DATA_HOME/uv,
    // HOME/.local/share/uv, CWD/.uv; on Windows: %APPDATA%\uv\data, .\.uv) with a trailing
    // "tools" component. uv_dirs gives the preferred state dir; if that fails, fall back to legacy
    // to cover older layouts."tools".
    let tools_dir = std::env::var_os("UV_TOOL_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            uv_dirs::user_state_dir()
                .or_else(uv_dirs::legacy_user_state_dir)
                .map(|p| p.join("tools"))
        })?;

    let entries = read_dir(&tools_dir).ok()?;
    let count = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .count();

    if count == 0 {
        None
    } else {
        Some(count)
    }
}

/// Minimal reimplementation of the bits we used from the `uv-dirs` crate, kept together for clarity.
///
/// Source: Adapted from https://github.com/astral-sh/uv/tree/main/crates/uv-dirs (MIT), pruned to
/// only the helpers needed by libmacchina. The behavior matches uv-dirs v0.0.14 for:
/// - user executable dir resolution (override / XDG / $HOME fallback)
/// - user state dir and legacy state dir (XDG vs native)
mod uv_dirs {
    use std::{
        ffi::OsString,
        path::{Path, PathBuf},
    };

    use etcetera::BaseStrategy;

    // Environment variable names used by uv for directory resolution.
    const XDG_BIN_HOME: &str = "XDG_BIN_HOME";
    const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

    /// Returns an appropriate user-level directory for storing executables.
    ///
    /// Order (same as uv-dirs): override var → XDG_BIN_HOME → XDG_DATA_HOME/../bin → $HOME/.local/bin.
    /// Returns `None` if `$HOME` cannot be resolved. Does not
    /// check if the directory exists.
    pub fn user_executable_directory(override_variable: Option<&'static str>) -> Option<PathBuf> {
        override_variable
            .and_then(std::env::var_os)
            .and_then(parse_path)
            .or_else(|| std::env::var_os(XDG_BIN_HOME).and_then(parse_xdg_path))
            .or_else(|| {
                std::env::var_os(XDG_DATA_HOME)
                    .and_then(parse_xdg_path)
                    .map(|path| path.join("../bin"))
            })
            .or_else(|| {
                let home_dir = etcetera::home_dir().ok();
                home_dir.map(|path| path.join(".local").join("bin"))
            })
    }

    /// `$XDG_DATA_HOME/uv` (or platform-native equivalent) per uv-dirs behavior.
    pub fn user_state_dir() -> Option<PathBuf> {
        etcetera::base_strategy::choose_base_strategy()
            .ok()
            .map(|dirs| dirs.data_dir().join("uv"))
    }

    /// Legacy location (`~/Library/Application Support/uv` on macOS; native strategy elsewhere).
    pub fn legacy_user_state_dir() -> Option<PathBuf> {
        etcetera::base_strategy::choose_native_strategy()
            .ok()
            .map(|dirs| dirs.data_dir().join("uv"))
            .map(|dir| if cfg!(windows) { dir.join("data") } else { dir })
    }

    /// Return a [`PathBuf`] from the given [`OsString`], if non-empty.
    ///
    /// Unlike [`parse_xdg_path`], this accepts relative paths (used for uv override vars).
    fn parse_path(path: OsString) -> Option<PathBuf> {
        if path.is_empty() {
            None
        } else {
            Some(PathBuf::from(path))
        }
    }

    /// Return a [`PathBuf`] if the given [`OsString`] is an absolute path, per XDG spec.
    /// Relative paths are treated as invalid.
    fn parse_xdg_path(path: OsString) -> Option<PathBuf> {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            Some(path)
        } else {
            None
        }
    }
}
