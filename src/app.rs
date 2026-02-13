use std::collections::HashMap;
use std::fs;
use std::path::Path;

use eframe::egui;

use crate::config;
use crate::theme::detect_colors_in_content;
use crate::theme::scanner::{scan_theme_files, scan_themes_dir};
use crate::utils::color::DetectedColor;

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum SortMode {
    Name,
    Color,
    #[default]
    LastOpened,
}

#[derive(Clone, Debug)]
pub struct ExtensionConfig {
    pub enabled: bool,
    pub color: egui::Color32,
}

#[derive(Clone, Debug)]
pub struct ColorEditTarget {
    pub color_id: String,
    pub file_name: String,
    pub original_value: egui::Color32,
    pub hex_text: String,
    pub original_format: ColorFormat,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColorFormat {
    Hex3,
    Hex6,
    Hex8,
    Rgb,
    Rgba,
}

pub fn detect_color_format(text: &str) -> ColorFormat {
    let text = text.trim();
    if text.starts_with("rgba") {
        ColorFormat::Rgba
    } else if text.starts_with("rgb") {
        ColorFormat::Rgb
    } else if text.starts_with('#') {
        match text.len() - 1 {
            3 => ColorFormat::Hex3,
            8 => ColorFormat::Hex8,
            _ => ColorFormat::Hex6,
        }
    } else {
        ColorFormat::Hex6
    }
}

pub fn color_to_format(color: egui::Color32, format: &ColorFormat) -> String {
    match format {
        ColorFormat::Hex3 => {
            let r = (color.r() / 17).to_string();
            let g = (color.g() / 17).to_string();
            let b = (color.b() / 17).to_string();
            format!("#{}{}{}", r, g, b)
        }
        ColorFormat::Hex6 => {
            format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
        }
        ColorFormat::Hex8 => {
            format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                color.r(),
                color.g(),
                color.b(),
                color.a()
            )
        }
        ColorFormat::Rgb => {
            format!("rgb({}, {}, {})", color.r(), color.g(), color.b())
        }
        ColorFormat::Rgba => {
            format!(
                "rgba({}, {}, {}, {})",
                color.r(),
                color.g(),
                color.b(),
                color.a() as f32 / 255.0
            )
        }
    }
}

pub struct OmarchyApp {
    pub themes_path: String,
    pub themes_path_backup: String,
    pub save_prefix: String,
    pub save_prefix_backup: String,
    pub show_settings: bool,
    pub theme_names: Vec<String>,
    pub selected_theme_index: Option<usize>,
    pub theme_files: Vec<String>,
    pub selected_file_index: Option<usize>,
    pub error_message: Option<String>,
    pub theme_sort_mode: SortMode,
    pub file_sort_mode: SortMode,
    pub theme_last_opened: HashMap<String, u64>,
    pub file_last_opened: HashMap<String, u64>,
    pub open_counter: u64,
    pub enabled_extensions: HashMap<String, ExtensionConfig>,
    pub file_content: String,
    pub file_cache: HashMap<String, String>,
    pub detected_colors: Vec<DetectedColor>,
    pub selected_color_id: Option<String>,
    pub config_source: Option<String>,

    // Color editing
    pub color_edit_target: Option<ColorEditTarget>,
    pub picker_color: Option<egui::Color32>,
    pub modified_colors: HashMap<String, String>, // color_id -> new hex value
    pub has_unsaved_changes: bool,
}

impl OmarchyApp {
    pub fn new() -> Self {
        let (config, config_source) = config::load_config();

        let mut enabled_extensions = HashMap::new();
        for (ext, setting) in config.extensions {
            let color = config::color_from_hex(&setting.color);
            enabled_extensions.insert(
                ext,
                ExtensionConfig {
                    enabled: setting.enabled,
                    color,
                },
            );
        }

        Self {
            themes_path: config.general.themes_path.clone(),
            themes_path_backup: config.general.themes_path.clone(),
            save_prefix: config.general.save_prefix.clone(),
            save_prefix_backup: config.general.save_prefix.clone(),
            show_settings: false,
            theme_names: vec![],
            selected_theme_index: None,
            theme_files: vec![],
            selected_file_index: None,
            error_message: None,
            theme_sort_mode: SortMode::LastOpened,
            file_sort_mode: SortMode::LastOpened,
            theme_last_opened: HashMap::new(),
            file_last_opened: HashMap::new(),
            open_counter: 0,
            enabled_extensions,
            file_content: String::new(),
            file_cache: HashMap::new(),
            detected_colors: vec![],
            selected_color_id: None,
            config_source,
            color_edit_target: None,
            picker_color: None,
            modified_colors: HashMap::new(),
            has_unsaved_changes: false,
        }
    }

    pub fn load_themes(&mut self) {
        self.error_message = None;
        eprintln!(
            "[DEBUG] load_themes() called with themes_path: {}",
            self.themes_path
        );
        self.theme_names = scan_themes_dir(&self.themes_path);
        eprintln!("[DEBUG] Found {} themes", self.theme_names.len());

        let mode = self.theme_sort_mode;
        let last_opened = &self.theme_last_opened;
        match mode {
            SortMode::Name => {
                self.theme_names
                    .sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            }
            SortMode::Color => {
                self.theme_names.sort_by(|a, b| {
                    let ext_a = a.rsplit('.').next().unwrap_or("").to_lowercase();
                    let ext_b = b.rsplit('.').next().unwrap_or("").to_lowercase();
                    ext_a.cmp(&ext_b)
                });
            }
            SortMode::LastOpened => {
                self.theme_names.sort_by(|a, b| {
                    let order_a = last_opened.get(a).unwrap_or(&0);
                    let order_b = last_opened.get(b).unwrap_or(&0);
                    order_b.cmp(order_a)
                });
            }
        }

        if self.theme_names.is_empty() {
            let expanded = crate::utils::path::expand_tilde(&self.themes_path);
            if !expanded.exists() {
                self.error_message = Some(format!("Path does not exist: {}", self.themes_path));
            } else {
                self.error_message = Some("No theme folders found in this directory".to_string());
            }
        }

        self.selected_theme_index = None;
        self.theme_files.clear();
        self.file_cache.clear();
        self.selected_file_index = None;
        self.file_content.clear();
        self.detected_colors.clear();
    }

    pub fn load_theme_files(&mut self) {
        self.error_message = None;
        self.file_content.clear();
        self.detected_colors.clear();
        self.file_cache.clear();

        if let Some(idx) = self.selected_theme_index {
            if let Some(name) = self.theme_names.get(idx) {
                self.open_counter += 1;
                self.theme_last_opened
                    .insert(name.clone(), self.open_counter);

                let theme_path = format!("{}/{}", self.themes_path.trim_end_matches('/'), name);
                eprintln!("[DEBUG] Scanning theme directory: {}", theme_path);
                self.theme_files = scan_theme_files(&theme_path);
                eprintln!("[DEBUG] Found {} files in theme", self.theme_files.len());

                // Preload all files into cache
                for file_name in &self.theme_files {
                    let file_path = format!("{}/{}", theme_path, file_name);
                    let expanded = crate::utils::path::expand_tilde(&file_path);
                    eprintln!("[DEBUG] Preloading file: {:?}", expanded);
                    if let Ok(content) = std::fs::read_to_string(&expanded) {
                        let bytes = content.len();
                        self.file_cache.insert(file_name.clone(), content);
                        eprintln!("[DEBUG] Cached {} ({} bytes)", file_name, bytes);
                    }
                }
                eprintln!("[DEBUG] Cache now contains {} files", self.file_cache.len());

                let mode = self.file_sort_mode;
                let last_opened = &self.file_last_opened;
                match mode {
                    SortMode::Name => {
                        self.theme_files
                            .sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
                    }
                    SortMode::Color => {
                        self.theme_files.sort_by(|a, b| {
                            let ext_a = a.rsplit('.').next().unwrap_or("").to_lowercase();
                            let ext_b = b.rsplit('.').next().unwrap_or("").to_lowercase();
                            ext_a.cmp(&ext_b)
                        });
                    }
                    SortMode::LastOpened => {
                        self.theme_files.sort_by(|a, b| {
                            let order_a = last_opened.get(a).unwrap_or(&0);
                            let order_b = last_opened.get(b).unwrap_or(&0);
                            order_b.cmp(order_a)
                        });
                    }
                }

                if self.theme_files.is_empty() {
                    self.error_message = Some(format!("No config files found in {}", name));
                }

                self.selected_file_index = None;
            }
        }
    }

    pub fn load_file_content(&mut self) {
        self.file_content.clear();
        self.detected_colors.clear();

        if let (Some(ti), Some(fi)) = (self.selected_theme_index, self.selected_file_index) {
            if let (Some(_theme_name), Some(file_name)) =
                (self.theme_names.get(ti), self.theme_files.get(fi))
            {
                let ext = file_name.rsplit('.').next().unwrap_or("").to_lowercase();
                let ext_with_dot = format!(".{}", ext);

                if let Some(config) = self.enabled_extensions.get(&ext_with_dot) {
                    if !config.enabled {
                        self.error_message = Some(format!("Color parsing disabled for .{}", ext));
                        return;
                    }
                } else {
                    self.error_message = Some(format!("Color parsing disabled for .{}", ext));
                    return;
                }

                // Read from cache instead of disk
                if let Some(content) = self.file_cache.get(file_name) {
                    eprintln!(
                        "[DEBUG] Loading from cache: {} ({} bytes)",
                        file_name,
                        content.len()
                    );
                    self.file_content = content.clone();
                    self.detected_colors = detect_colors_in_content(&self.file_content);
                } else {
                    eprintln!("[DEBUG] File NOT in cache: {}", file_name);
                    self.error_message = Some(format!("File not in cache: {}", file_name));
                }
            }
        }
    }

    pub fn enter_settings(&mut self) {
        self.themes_path_backup = self.themes_path.clone();
        self.save_prefix_backup = self.save_prefix.clone();
        self.show_settings = true;
    }

    pub fn settings_ok(&mut self) {
        self.show_settings = false;

        let mut extensions = HashMap::new();
        for (ext, cfg) in &self.enabled_extensions {
            extensions.insert(
                ext.clone(),
                config::ExtensionSetting {
                    enabled: cfg.enabled,
                    color: config::color_to_hex(cfg.color),
                },
            );
        }

        let config = config::AppConfig {
            general: config::GeneralConfig {
                themes_path: self.themes_path.clone(),
                save_prefix: self.save_prefix.clone(),
            },
            extensions,
        };

        if let Err(e) = config::save_config(&config) {
            self.error_message = Some(format!("Failed to save config: {}", e));
        }

        self.load_themes();
    }

    pub fn settings_cancel(&mut self) {
        self.themes_path = self.themes_path_backup.clone();
        self.save_prefix = self.save_prefix_backup.clone();
        self.show_settings = false;
    }

    pub fn sort_themes(&mut self, mode: SortMode) {
        self.theme_sort_mode = mode;
        let last_opened = &self.theme_last_opened;
        match mode {
            SortMode::Name => {
                self.theme_names
                    .sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            }
            SortMode::Color => {
                self.theme_names.sort_by(|a, b| {
                    let ext_a = a.rsplit('.').next().unwrap_or("").to_lowercase();
                    let ext_b = b.rsplit('.').next().unwrap_or("").to_lowercase();
                    ext_a.cmp(&ext_b)
                });
            }
            SortMode::LastOpened => {
                self.theme_names.sort_by(|a, b| {
                    let order_a = last_opened.get(a).unwrap_or(&0);
                    let order_b = last_opened.get(b).unwrap_or(&0);
                    order_b.cmp(order_a)
                });
            }
        }
    }

    pub fn sort_files(&mut self, mode: SortMode) {
        self.file_sort_mode = mode;
        let last_opened = &self.file_last_opened;
        match mode {
            SortMode::Name => {
                self.theme_files
                    .sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            }
            SortMode::Color => {
                self.theme_files.sort_by(|a, b| {
                    let ext_a = a.rsplit('.').next().unwrap_or("").to_lowercase();
                    let ext_b = b.rsplit('.').next().unwrap_or("").to_lowercase();
                    ext_a.cmp(&ext_b)
                });
            }
            SortMode::LastOpened => {
                self.theme_files.sort_by(|a, b| {
                    let order_a = last_opened.get(a).unwrap_or(&0);
                    let order_b = last_opened.get(b).unwrap_or(&0);
                    order_b.cmp(order_a)
                });
            }
        }
    }

    pub fn start_color_edit(
        &mut self,
        color_id: String,
        file_name: String,
        original_value: egui::Color32,
        hex_text: String,
    ) {
        let original_format = detect_color_format(&hex_text);
        eprintln!(
            "[DEBUG] Color format detected: {:?} from '{}'",
            original_format, hex_text
        );
        self.color_edit_target = Some(ColorEditTarget {
            color_id,
            file_name,
            original_value,
            hex_text,
            original_format,
        });
    }

    pub fn close_color_edit(&mut self) {
        self.color_edit_target = None;
        self.picker_color = None;
    }

    pub fn update_color(&mut self, new_color: egui::Color32) {
        if let Some(ref target) = self.color_edit_target {
            let new_formatted = color_to_format(new_color, &target.original_format);
            eprintln!(
                "[DEBUG] Changing color {} -> {} (format: {:?}) (in memory only)",
                target.hex_text, new_formatted, target.original_format
            );
            self.modified_colors
                .insert(target.color_id.clone(), new_formatted.clone());
            self.has_unsaved_changes = true;

            let color_id = target.color_id.clone();
            let old_hex = target.hex_text.clone();
            self.apply_color_change(&color_id, &old_hex, &new_formatted);

            self.detected_colors = detect_colors_in_content(&self.file_content);
            self.color_edit_target = None;
        }
    }

    pub fn apply_color_change(&mut self, color_id: &str, old_hex: &str, new_hex: &str) {
        // Find the position from detected_colors
        if let Some(color_info) = self.detected_colors.iter().find(|c| c.id == color_id) {
            let line_num = color_info.line;
            let col_start = color_info.start_col;

            let lines: Vec<&str> = self.file_content.lines().collect();
            if let Some(line) = lines.get(line_num) {
                if col_start < line.len() {
                    // Replace the old hex with new hex in place
                    let prefix = &line[..col_start];
                    let old_len = old_hex.len();
                    let suffix = if col_start + old_len < line.len() {
                        &line[col_start + old_len..]
                    } else {
                        ""
                    };
                    let new_line = format!("{}{}{}", prefix, new_hex, suffix);

                    // Rebuild the file content
                    let mut new_lines = lines.to_vec();
                    new_lines[line_num] = &new_line;
                    self.file_content = new_lines.join("\n");
                }
            }
        }
    }

    pub fn rebuild_file_content(&mut self) {
        let mut content = String::new();

        if let (Some(ti), Some(fi)) = (self.selected_theme_index, self.selected_file_index) {
            if let (Some(theme_name), Some(file_name)) =
                (self.theme_names.get(ti), self.theme_files.get(fi))
            {
                let file_path = format!(
                    "{}/{}/{}",
                    self.themes_path.trim_end_matches('/'),
                    theme_name,
                    file_name
                );
                let expanded = crate::utils::path::expand_tilde(&file_path);
                if let Ok(original) = std::fs::read_to_string(&expanded) {
                    content = original;
                }
            }
        }

        for (color_id, new_hex) in &self.modified_colors {
            let parts: Vec<&str> = color_id.split('_').collect();
            if parts.len() >= 2 {
                if let (Ok(line_num), Ok(col_start)) =
                    (parts[0].parse::<usize>(), parts[1].parse::<usize>())
                {
                    let lines: Vec<char> = content.chars().collect();
                    let mut current_line = 0;
                    let mut line_start = 0;

                    for (i, c) in lines.iter().enumerate() {
                        if *c == '\n' {
                            if current_line == line_num {
                                break;
                            }
                            current_line += 1;
                            line_start = i + 1;
                        }
                    }

                    if current_line == line_num {
                        let mut col_end = line_start + col_start;
                        while col_end < lines.len()
                            && !lines[col_end].is_whitespace()
                            && lines[col_end] != '\n'
                        {
                            col_end += 1;
                        }

                        if line_start + col_start <= col_end && col_end <= lines.len() {
                            let old_color = self.detected_colors.iter().find(|c| c.id == *color_id);
                            if let Some(old) = old_color {
                                let old_len = old.hex_text.len();
                                if line_start + col_start + old_len <= lines.len() {
                                    let mut new_content: String =
                                        lines[..line_start + col_start].iter().collect();
                                    new_content.push_str(&new_hex);
                                    new_content
                                        .extend(lines[line_start + col_start + old_len..].iter());
                                    content = new_content;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.file_content = content.clone();
        self.detected_colors = detect_colors_in_content(&content);
    }

    pub fn save_file(&mut self) -> Result<(), String> {
        if let (Some(ti), Some(fi)) = (self.selected_theme_index, self.selected_file_index) {
            if let (Some(theme_name), Some(file_name)) =
                (self.theme_names.get(ti), self.theme_files.get(fi))
            {
                let file_path = format!(
                    "{}/{}/{}",
                    self.themes_path.trim_end_matches('/'),
                    theme_name,
                    file_name
                );
                let expanded = crate::utils::path::expand_tilde(&file_path);
                std::fs::write(&expanded, &self.file_content).map_err(|e| e.to_string())?;
                self.modified_colors.clear();
                self.has_unsaved_changes = false;
                return Ok(());
            }
        }
        Err("No file selected".to_string())
    }

    pub fn save_as_new(&mut self) -> Result<(), String> {
        if let Some(ti) = self.selected_theme_index {
            if let Some(theme_name) = self.theme_names.get(ti) {
                let new_theme_name = format!("{}{}", self.save_prefix, theme_name);
                let new_theme_path = format!(
                    "{}/{}",
                    self.themes_path.trim_end_matches('/'),
                    new_theme_name
                );
                let expanded = crate::utils::path::expand_tilde(&new_theme_path);

                std::fs::create_dir_all(&expanded).map_err(|e| e.to_string())?;

                // Copy backgrounds folder from original theme
                let original_backgrounds = format!(
                    "{}/{}/backgrounds",
                    self.themes_path.trim_end_matches('/'),
                    theme_name
                );
                let original_backgrounds_expanded =
                    crate::utils::path::expand_tilde(&original_backgrounds);
                let new_backgrounds_str = format!("{}/backgrounds", expanded.display());
                let new_backgrounds = Path::new(&new_backgrounds_str);

                if original_backgrounds_expanded.exists() && original_backgrounds_expanded.is_dir()
                {
                    copy_dir_all(&original_backgrounds_expanded, new_backgrounds)
                        .map_err(|e| format!("Failed to copy backgrounds: {}", e))?;
                    eprintln!("[DEBUG] Copied backgrounds folder");
                }

                for (file_name, content) in &self.file_cache {
                    let file_path = format!("{}/{}", expanded.display(), file_name);
                    std::fs::write(&file_path, content).map_err(|e| e.to_string())?;
                }

                eprintln!("[DEBUG] Saved new theme: {}", new_theme_path);
                self.has_unsaved_changes = false;
                self.load_themes();
                return Ok(());
            }
        }
        Err("No theme selected".to_string())
    }

    pub fn overwrite_theme(&mut self) -> Result<(), String> {
        if let Some(ti) = self.selected_theme_index {
            if let Some(theme_name) = self.theme_names.get(ti) {
                let theme_path =
                    format!("{}/{}", self.themes_path.trim_end_matches('/'), theme_name);
                let expanded = crate::utils::path::expand_tilde(&theme_path);

                if !expanded.exists() {
                    return Err(format!("Theme folder does not exist: {}", theme_path));
                }

                for (file_name, content) in &self.file_cache {
                    let file_path = format!("{}/{}", expanded.display(), file_name);
                    std::fs::write(&file_path, content).map_err(|e| e.to_string())?;
                }

                eprintln!("[DEBUG] Overwrote theme: {}", theme_path);
                self.has_unsaved_changes = false;
                return Ok(());
            }
        }
        Err("No theme selected".to_string())
    }
}
