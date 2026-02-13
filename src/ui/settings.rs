use crate::app::OmarchyApp;
use eframe::egui;

pub fn ui_settings_panel(ctx: &egui::Context, app: &mut OmarchyApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Settings");
        ui.separator();

        if let Some(ref source) = app.config_source {
            ui.colored_label(
                egui::Color32::GRAY,
                format!("Config loaded from: {}", source),
            );
        } else {
            ui.colored_label(egui::Color32::GRAY, "Config loaded from: defaults");
        }
        ui.separator();

        ui.label("Omarchy Themes Directory:");
        ui.add(egui::TextEdit::singleline(&mut app.themes_path).desired_width(400.0));

        ui.separator();

        ui.label("Save Prefix (for new themes):");
        ui.add(egui::TextEdit::singleline(&mut app.save_prefix).desired_width(200.0));

        ui.separator();
        ui.label("Enabled File Extensions:");

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut extensions: Vec<_> = app.enabled_extensions.keys().cloned().collect();
            extensions.sort();

            for ext in &extensions {
                ui.horizontal(|ui| {
                    if let Some(config) = app.enabled_extensions.get_mut(ext) {
                        let mut check = config.enabled;
                        if ui.checkbox(&mut check, ext).changed() {
                            config.enabled = check;
                        }
                        ui.add_space(10.0);

                        let mut color_array =
                            [config.color.r(), config.color.g(), config.color.b()];
                        egui::color_picker::color_edit_button_srgb(ui, &mut color_array);
                        config.color =
                            egui::Color32::from_rgb(color_array[0], color_array[1], color_array[2]);
                    }
                });
            }
        });

        if let Some(ref err) = app.error_message {
            ui.separator();
            ui.colored_label(egui::Color32::RED, err);
        }

        ui.add_space(ui.available_height() - 30.0);

        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                app.settings_cancel();
            }
            if ui.button("OK").clicked() {
                app.settings_ok();
            }
        });
    });
}
