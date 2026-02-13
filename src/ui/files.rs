use crate::app::{OmarchyApp, SortMode};
use crate::theme::scanner::get_extension;
use eframe::egui;

pub fn get_ext_color(
    ext: &str,
    enabled_exts: &std::collections::HashMap<String, crate::app::ExtensionConfig>,
) -> eframe::egui::Color32 {
    if let Some(config) = enabled_exts.get(ext) {
        return config.color;
    }
    crate::utils::color::get_default_ext_color(ext)
}

pub fn ui_files_panel(ctx: &egui::Context, app: &mut OmarchyApp) {
    egui::SidePanel::left("files_panel")
        .min_width(150.0)
        .max_width(250.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Files");
                ui.add_space(10.0);

                let n_active = app.file_sort_mode == SortMode::Name;
                if ui.selectable_label(n_active, "N").clicked() {
                    app.sort_files(SortMode::Name);
                }

                let t_active = app.file_sort_mode == SortMode::Color;
                if ui.selectable_label(t_active, "T").clicked() {
                    app.sort_files(SortMode::Color);
                }

                let l_active = app.file_sort_mode == SortMode::LastOpened;
                if ui.selectable_label(l_active, "L").clicked() {
                    app.sort_files(SortMode::LastOpened);
                }
            });
            ui.separator();

            if app.theme_files.is_empty() {
                ui.label("No files found");
            } else {
                let use_colors = app.file_sort_mode == SortMode::Color;
                let files: Vec<(usize, String)> = app
                    .theme_files
                    .iter()
                    .enumerate()
                    .map(|(i, n)| (i, n.clone()))
                    .collect();

                let mut selected_idx: Option<usize> = None;

                for (i, name) in files {
                    let is_selected = app.selected_file_index == Some(i);

                    if use_colors {
                        let ext = get_extension(&name);
                        let ext_with_dot = format!(".{}", ext);
                        let color = get_ext_color(&ext_with_dot, &app.enabled_extensions);
                        if ui
                            .selectable_label(
                                is_selected,
                                eframe::egui::RichText::new(&name).color(color),
                            )
                            .clicked()
                        {
                            selected_idx = Some(i);
                        }
                    } else {
                        if ui.selectable_label(is_selected, &name).clicked() {
                            selected_idx = Some(i);
                        }
                    }
                }

                if let Some(idx) = selected_idx {
                    app.selected_file_index = Some(idx);
                    app.load_file_content();
                }
            }
        });
}
