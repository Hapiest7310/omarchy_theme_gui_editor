use std::path::PathBuf;

pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(path.strip_prefix('~').unwrap_or(""));
        }
    }
    PathBuf::from(path)
}
