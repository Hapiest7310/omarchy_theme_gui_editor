use crate::app::{OmarchyApp, SortMode};
use eframe::egui;

pub fn ui_themes_panel(ctx: &egui::Context, app: &mut OmarchyApp) {
    egui::SidePanel::left("themes_panel")
        .min_width(150.0)
        .max_width(250.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Themes");
                ui.add_space(10.0);

                let n_active = app.theme_sort_mode == SortMode::Name;
                if ui.selectable_label(n_active, "N").clicked() {
                    app.sort_themes(SortMode::Name);
                }

                let l_active = app.theme_sort_mode == SortMode::LastOpened;
                if ui.selectable_label(l_active, "L").clicked() {
                    app.sort_themes(SortMode::LastOpened);
                }
            });
            ui.separator();

            if let Some(ref err) = app.error_message {
                ui.colored_label(egui::Color32::YELLOW, err);
                ui.separator();
            }

            if app.theme_names.is_empty() {
                ui.label("No themes found");
                ui.label("Click Settings to change path");
            } else {
                let theme_names: Vec<(usize, String)> = app
                    .theme_names
                    .iter()
                    .enumerate()
                    .map(|(i, n)| (i, n.clone()))
                    .collect();

                let mut selected_idx: Option<usize> = None;

                for (i, name) in theme_names {
                    let is_selected = app.selected_theme_index == Some(i);
                    if ui.selectable_label(is_selected, &name).clicked() {
                        selected_idx = Some(i);
                    }
                }

                if let Some(idx) = selected_idx {
                    app.selected_theme_index = Some(idx);
                    app.load_theme_files();
                }
            }
        });
}
