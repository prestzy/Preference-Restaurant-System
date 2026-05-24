use crate::models::RecommendationResult;
use eframe::egui;

/// Displays a list of values as compact chips.
pub fn chip_row(ui: &mut egui::Ui, values: &[String]) {
    ui.horizontal_wrapped(|ui| {
        for value in values {
            egui::Frame::none()
                .fill(ui.visuals().extreme_bg_color)
                .rounding(4.0)
                .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                .show(ui, |ui| {
                    ui.small(value);
                });
        }
    });
}

/// Renders one selectable option chip.
///
/// The selected state uses a light accent fill and border so the white theme
/// remains clean while still making active choices obvious.
pub fn option_chip(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    let (fill, stroke) = if selected {
        (
            egui::Color32::from_rgb(219, 234, 254),
            egui::Stroke::new(1.0, egui::Color32::from_rgb(37, 99, 235)),
        )
    } else {
        (
            egui::Color32::from_rgb(248, 250, 252),
            egui::Stroke::new(1.0, egui::Color32::from_rgb(226, 232, 240)),
        )
    };

    ui.add(
        egui::Button::new(label)
            .fill(fill)
            .stroke(stroke)
            .rounding(6.0),
    )
    .clicked()
}

/// Renders one recommendation in an explainable score format.
pub fn recommendation_card(ui: &mut egui::Ui, rank: usize, recommendation: &RecommendationResult) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!(
                "{}. {} ({})",
                rank, recommendation.dish.name, recommendation.dish.dish_id
            ));
            ui.label(format!("Category: {}", recommendation.dish.category));
        });

        ui.label(format!(
            "Ingredient {:.2} | Co-order {:.2} | Hybrid {:.2}",
            recommendation.ingredient_score,
            recommendation.co_order_score,
            recommendation.final_score
        ));
        ui.label(&recommendation.explanation);
    });
}

/// Converts a string list into stakeholder-friendly text.
pub fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(", ")
    }
}
