use crate::utils::path::expand_tilde;

pub fn scan_themes_dir(path: &str) -> Vec<String> {
    let expanded = expand_tilde(path);
    if !expanded.exists() {
        return vec![];
    }
    if let Ok(entries) = std::fs::read_dir(&expanded) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| !name.starts_with('.'))
            .collect()
    } else {
        vec![]
    }
}

pub fn scan_theme_files(theme_path: &str) -> Vec<String> {
    let expanded = expand_tilde(theme_path);
    if !expanded.exists() {
        return vec![];
    }
    if let Ok(entries) = std::fs::read_dir(&expanded) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| !name.starts_with('.'))
            .filter(|name| !name.ends_with(".png") && !name.ends_with(".jpg"))
            .collect()
    } else {
        vec![]
    }
}

pub fn get_extension(name: &str) -> String {
    name.rsplit('.').next().unwrap_or("").to_lowercase()
}
