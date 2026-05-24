use crate::models::RecommendationResult;
use eframe::egui;

/// Displays comma-separated values as compact chips.
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

/// Shows a labelled text input and returns whether the user changed it.
pub fn labelled_text_input(ui: &mut egui::Ui, label: &str, hint: &str, value: &mut String) -> bool {
    ui.label(label);
    ui.add(egui::TextEdit::singleline(value).hint_text(hint))
        .changed()
}

/// Converts a string list into stakeholder-friendly text.
pub fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(", ")
    }
}
