use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

pub(crate) fn apply_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    visuals.dark_mode = true;
    visuals.panel_fill = Color32::from_rgb(15, 21, 30);
    visuals.window_fill = Color32::from_rgb(21, 29, 40);
    visuals.extreme_bg_color = Color32::from_rgb(9, 14, 21);
    visuals.faint_bg_color = Color32::from_rgb(25, 35, 48);
    visuals.override_text_color = Some(Color32::from_rgb(225, 233, 242));
    visuals.selection.bg_fill = Color32::from_rgb(24, 119, 112);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(117, 226, 206));
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(22, 31, 43);
    visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(170, 185, 203);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(29, 41, 55);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(29, 41, 55);
    visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(213, 224, 237);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 59, 73);
    visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
    visuals.widgets.active.bg_fill = Color32::from_rgb(17, 110, 103);
    visuals.widgets.active.fg_stroke.color = Color32::WHITE;
    visuals.widgets.open.bg_fill = Color32::from_rgb(27, 45, 58);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);
    // egui 0.37 stores styles per theme. Select the dark theme explicitly,
    // then mutate that theme's style through the current API.
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_visuals(visuals);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = egui::vec2(9.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.window_margin = egui::Margin::same(16);
    });
}
