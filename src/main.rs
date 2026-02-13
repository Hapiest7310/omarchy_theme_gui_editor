mod app;
mod config;
mod theme;
mod ui;
mod utils;

use app::OmarchyApp;
use eframe::egui;
use ui::{ui_colors_panel, ui_files_panel, ui_settings_panel, ui_themes_panel};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 650.0])
            .with_min_inner_size([700.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Omarchy Theme Maker",
        options,
        Box::new(|_cc| Ok(Box::new(OmarchyApp::new()))),
    )
}

impl eframe::App for OmarchyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let settings_btn = ui.button("⚙ Settings");
                if settings_btn.clicked() {
                    self.enter_settings();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let has_theme = self.selected_theme_index.is_some();
                    let has_changes = self.has_unsaved_changes;

                    let overwrite_btn = ui.add_enabled(has_theme, egui::Button::new("Overwrite"));
                    if overwrite_btn.clicked() {
                        if let Err(e) = self.overwrite_theme() {
                            self.error_message = Some(e);
                        }
                    }

                    let save_btn =
                        ui.add_enabled(has_theme && has_changes, egui::Button::new("Save"));
                    if save_btn.clicked() {
                        if let Err(e) = self.save_as_new() {
                            self.error_message = Some(e);
                        }
                    }
                });
            });
        });

        if self.show_settings {
            ui_settings_panel(ctx, self);
        } else {
            if self.theme_names.is_empty() {
                self.load_themes();
            }

            ui_themes_panel(ctx, self);

            if self.selected_theme_index.is_some() {
                ui_files_panel(ctx, self);
            }

            if self.selected_file_index.is_some() {
                ui_colors_panel(ctx, self);
            }
        }
    }
}
