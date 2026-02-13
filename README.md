# Omarchy Theme GUI Editor

A graphical theme editor for Omarchy Linux configurations. Built with Rust and eframe/egui.

## Features

- Browse and select theme folders from your omarchy themes directory
- View and edit configuration files (hyprland.conf, waybar.css, alacritty.toml, etc.)
- Detect and modify colors in config files (hex, rgb, rgba formats)
- Save modifications as new themes or overwrite existing ones
- In-memory editing with preview before saving

## Installation

```bash
# Clone the repository
git clone https://github.com/Hapiest7310/omarchy_theme_gui_editor.git
cd omarchy_theme_gui_editor

# Build
cargo build --release

# Run
cargo run
```

## Usage

1. Launch the application
2. Select a theme from the left panel
3. Select a config file from the middle panel
4. Click on any color in the right panel to edit it
5. Use "Save" to create a new theme or "Overwrite" to update the original

### Settings

- **Themes Directory**: Path to your omarchy themes folder (default: `~/.config/omarchy/themes`)
- **Save Prefix**: Prefix for new theme names when saving

## Configuration

The app stores its config in `~/.config/omarchy-theme-maker/config.toml`

Supported file extensions for color detection:
- `.css` - Waybar, SwayOSD styles
- `.toml` - Alacritty, colors
- `.theme` - Btop themes
- `.conf` - Hyprland, Ghostty, Hyprlock
- `.lua` - Neovim
- `.ini` - Mako

## License

MIT
