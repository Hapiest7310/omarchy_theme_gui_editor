use crate::app::OmarchyApp;
use crate::utils::color::get_contrast_color;
use eframe::egui;

pub fn ui_colors_panel(ctx: &egui::Context, app: &mut OmarchyApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Colors");
        if app.has_unsaved_changes {
            ui.label(egui::RichText::new(" *").color(egui::Color32::RED));
        }
        ui.separator();

        if app.file_content.is_empty() {
            ui.label("No content loaded");
            return;
        }

        let editing_color = app.color_edit_target.clone();
        let modified_colors = app.modified_colors.clone();
        let detected_colors = app.detected_colors.clone();
        let file_content = app.file_content.clone();
        let file_index = app.selected_file_index;
        let theme_files = app.theme_files.clone();
        let themes_path = app.themes_path.clone();
        let theme_index = app.selected_theme_index;
        let theme_names = app.theme_names.clone();

        let file_path = if let (Some(ti), Some(fi)) = (theme_index, file_index) {
            if let (Some(theme_name), Some(file_name)) = (theme_names.get(ti), theme_files.get(fi))
            {
                Some(format!(
                    "{}/{}/{}",
                    themes_path.trim_end_matches('/'),
                    theme_name,
                    file_name
                ))
            } else {
                None
            }
        } else {
            None
        };

        if let Some(ref path) = file_path {
            ui.label(egui::RichText::new(path).color(egui::Color32::GRAY).small());
        }

        if let Some(ref target) = editing_color {
            if app.picker_color.is_none() {
                app.picker_color = Some(target.original_value);
            }
            let mut color_value = app.picker_color.unwrap();
            ui.horizontal(|ui| {
                ui.label("Edit color:");
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut color_value,
                    egui::color_picker::Alpha::BlendOrAdditive,
                );
                app.picker_color = Some(color_value);

                if ui.button("Done").clicked() {
                    app.update_color(color_value);
                    app.close_color_edit();
                }
            });
            ui.separator();
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            let line_height = 20.0;
            let line_number_width = 50.0;
            let default_text_color = egui::Color32::from_gray(204);

            let lines: Vec<&str> = file_content.lines().collect();

            for (line_idx, line) in lines.iter().enumerate() {
                let line_colors: Vec<_> = detected_colors
                    .iter()
                    .filter(|c| c.line == line_idx)
                    .collect();

                let get_effective_color =
                    |color_info: &&crate::utils::color::DetectedColor| -> egui::Color32 {
                        if let Some(new_hex) = modified_colors.get(&color_info.id) {
                            crate::config::color_from_hex(new_hex)
                        } else {
                            color_info.value
                        }
                    };

                let line_color = line_colors.first().map(get_effective_color);

                if let Some(bg_color) = line_color {
                    let rect = egui::Rect::from_min_size(
                        ui.next_widget_position(),
                        egui::vec2(ui.available_width(), line_height),
                    );
                    let (r, g, b) = (bg_color.r(), bg_color.g(), bg_color.b());
                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(r, g, b, 60),
                    );
                }

                ui.horizontal(|ui| {
                    ui.set_min_height(line_height);

                    ui.add_sized(
                        [line_number_width, line_height],
                        egui::Label::new(
                            eframe::egui::RichText::new(format!("{}", line_idx + 1))
                                .color(egui::Color32::GRAY),
                        ),
                    );

                    if line_colors.is_empty() {
                        ui.colored_label(default_text_color, *line);
                    } else {
                        let mut last_end = 0;
                        for color_info in &line_colors {
                            if color_info.start_col > last_end {
                                let text = &line[last_end..color_info.start_col];
                                ui.colored_label(default_text_color, text);
                            }

                            let effective_color = get_effective_color(color_info);
                            let is_modified = modified_colors.contains_key(&color_info.id);

                            let contrast = get_contrast_color(effective_color);
                            let label = if is_modified {
                                format!("*{}", color_info.hex_text)
                            } else {
                                color_info.hex_text.clone()
                            };

                            let color_text = eframe::egui::RichText::new(label).color(contrast);

                            if ui.selectable_label(false, color_text).clicked() {
                                let file_name = theme_files
                                    .get(file_index.unwrap_or(0))
                                    .cloned()
                                    .unwrap_or_default();
                                app.start_color_edit(
                                    color_info.id.clone(),
                                    file_name,
                                    color_info.value,
                                    color_info.hex_text.clone(),
                                );
                            }

                            last_end = color_info.end_col;
                        }

                        if last_end < line.len() {
                            let text = &line[last_end..];
                            ui.colored_label(default_text_color, text);
                        }
                    }
                });
            }
        });
    });
}
