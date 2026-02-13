use egui::Color32;
use regex::Regex;

pub const DEFAULT_EXT_COLORS: &[Color32] = &[
    Color32::from_rgb(255, 107, 107),
    Color32::from_rgb(78, 205, 196),
    Color32::from_rgb(255, 230, 109),
    Color32::from_rgb(26, 83, 92),
    Color32::from_rgb(255, 159, 67),
    Color32::from_rgb(84, 160, 255),
    Color32::from_rgb(95, 39, 205),
    Color32::from_rgb(29, 209, 161),
    Color32::from_rgb(255, 159, 243),
    Color32::from_rgb(34, 166, 179),
    Color32::from_rgb(244, 180, 26),
    Color32::from_rgb(163, 152, 173),
    Color32::from_rgb(206, 147, 216),
    Color32::from_rgb(129, 236, 182),
    Color32::from_rgb(250, 177, 133),
    Color32::from_rgb(127, 143, 166),
];

#[derive(Clone, Debug)]
pub struct DetectedColor {
    pub id: String,
    pub value: Color32,
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub hex_text: String,
}

pub fn get_contrast_color(color: Color32) -> Color32 {
    let r = color.r() as u32;
    let g = color.g() as u32;
    let b = color.b() as u32;
    let brightness = (r + g + b) / 3;
    if brightness > 128 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

pub fn get_default_ext_color(name: &str) -> Color32 {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    let hash = ext.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
    DEFAULT_EXT_COLORS[(hash as usize) % DEFAULT_EXT_COLORS.len()]
}

pub fn parse_hex_color(hex: &str) -> Option<Color32> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color32::from_rgba_unmultiplied(r, g, b, a))
        }
        _ => None,
    }
}

pub fn detect_colors_in_content(content: &str) -> Vec<DetectedColor> {
    let mut colors = Vec::new();

    let hex_regex = Regex::new(r"#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b").unwrap();
    let rgb_regex =
        Regex::new(r"rgba?\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*([\d.]+))?\s*\)").unwrap();

    for (line_idx, line) in content.lines().enumerate() {
        for mat in hex_regex.find_iter(line) {
            if let Some(color) = parse_hex_color(mat.as_str()) {
                colors.push(DetectedColor {
                    id: format!("{}_{}", line_idx, mat.start()),
                    value: color,
                    line: line_idx,
                    start_col: mat.start(),
                    end_col: mat.end(),
                    hex_text: mat.as_str().to_string(),
                });
            }
        }

        for mat in rgb_regex.find_iter(line) {
            let caps = rgb_regex.captures(mat.as_str()).unwrap();
            let r: u8 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
            let g: u8 = caps.get(2).unwrap().as_str().parse().unwrap_or(0);
            let b: u8 = caps.get(3).unwrap().as_str().parse().unwrap_or(0);
            let a: u8 = caps
                .get(4)
                .and_then(|m| m.as_str().parse::<f32>().ok())
                .map(|f| (f * 255.0) as u8)
                .unwrap_or(255);

            colors.push(DetectedColor {
                id: format!("{}_{}", line_idx, mat.start()),
                value: Color32::from_rgba_unmultiplied(r, g, b, a),
                line: line_idx,
                start_col: mat.start(),
                end_col: mat.end(),
                hex_text: mat.as_str().to_string(),
            });
        }
    }

    colors
}
