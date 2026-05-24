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
///
/// The Evaluation page uses this detailed card because recommendations were
/// moved out of Explore. This keeps the normal browsing page focused on input,
/// while Evaluation explains how each score was produced for lecturers or demo
/// viewers.
pub fn recommendation_card(
    ui: &mut egui::Ui,
    rank: usize,
    recommendation: &RecommendationResult,
    related_selected_dish_labels: &[String],
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.heading(format!("{}. {}", rank, recommendation.dish.name));
            ui.monospace(format!("({})", recommendation.dish.dish_id));
            ui.label(format!("Category: {}", recommendation.dish.category));
        });

        ui.add_space(4.0);
        ui.label(format!(
            "Final hybrid score: {:.2} | Ingredient score: {:.2} | Co-order score: {:.2}",
            recommendation.final_score,
            recommendation.ingredient_score,
            recommendation.co_order_score
        ));

        ui.separator();
        ui.label("Detailed reason:");
        ui.label(build_detailed_reason(
            recommendation,
            related_selected_dish_labels,
        ));
        ui.label(format!("Summary: {}", recommendation.explanation));

        ui.add_space(4.0);
        ui.label(format!(
            "Matched liked ingredients: {}",
            display_list(&recommendation.matched_liked_ingredients)
        ));
        ui.label(format!(
            "Matched preferred tags: {}",
            display_list(&recommendation.matched_preferred_tags)
        ));
        ui.label(format!(
            "Disliked ingredient check: {}",
            if recommendation.matched_disliked_ingredients.is_empty() {
                "No disliked ingredients matched; disliked dishes are excluded before ranking."
                    .to_string()
            } else {
                format!(
                    "Matched disliked ingredient(s): {}",
                    recommendation.matched_disliked_ingredients.join(", ")
                )
            }
        ));
        ui.label(format!(
            "Co-order influence from selected dish(es): {}",
            display_list(related_selected_dish_labels)
        ));
    });
}

/// Builds a lecturer-friendly explanation from explicit recommendation fields.
///
/// The recommender stores matched ingredients, matched tags, and related cart
/// dish IDs. This function turns those technical details into a transparent
/// sentence instead of vague wording such as "good match".
fn build_detailed_reason(
    recommendation: &RecommendationResult,
    related_selected_dish_labels: &[String],
) -> String {
    let mut reasons = Vec::new();

    if !recommendation.matched_liked_ingredients.is_empty() {
        reasons.push(format!(
            "matches liked ingredient(s): {}",
            recommendation.matched_liked_ingredients.join(", ")
        ));
    }

    if !recommendation.matched_preferred_tags.is_empty() {
        reasons.push(format!(
            "matches preferred tag(s): {}",
            recommendation.matched_preferred_tags.join(", ")
        ));
    }

    if !related_selected_dish_labels.is_empty() {
        reasons.push(format!(
            "is often ordered together with selected dish(es): {}",
            related_selected_dish_labels.join(", ")
        ));
    }

    if reasons.is_empty() {
        format!(
            "{} ({}) is shown because its hybrid score is {:.2}, based on the available preference and order-history signals.",
            recommendation.dish.name, recommendation.dish.dish_id, recommendation.final_score
        )
    } else {
        format!(
            "{} ({}) is recommended because it {}.",
            recommendation.dish.name,
            recommendation.dish.dish_id,
            reasons.join("; ")
        )
    }
}

/// Converts a string list into stakeholder-friendly text.
pub fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(", ")
    }
}
