use std::env;
use std::path::PathBuf;

/// Returns `true` if the host is using Sway (A wayland window manager).
pub fn is_running_sway() -> bool {
    if let Ok(socket) = env::var("SWAYSOCK")
    {
        if PathBuf::from(socket).is_file() { 
            return true;
        }
    }

    false
}
