use crate::image_loader::DishImageCache;
use crate::models::{Dish, RecommendationResult};
use eframe::egui;

/// Shared colors for selected states and primary actions.
const ACCENT_BLUE: egui::Color32 = egui::Color32::from_rgb(37, 99, 235);
const ACCENT_BLUE_LIGHT: egui::Color32 = egui::Color32::from_rgb(219, 234, 254);
const NEUTRAL_BORDER: egui::Color32 = egui::Color32::from_rgb(226, 232, 240);
const PLACEHOLDER_FILL: egui::Color32 = egui::Color32::from_rgb(248, 250, 252);
const MUTED_TEXT: egui::Color32 = egui::Color32::from_rgb(100, 116, 139);

/// State for the reusable image preview modal.
///
/// The modal is owned by the top-level app, not by individual pages. Menu cards
/// and recommendation cards both set this state when their thumbnail is clicked,
/// which avoids duplicating image-preview logic in multiple UI sections.
#[derive(Debug, Clone)]
pub struct ImagePreviewState {
    pub dish_id: String,
    pub dish_name: String,
}

impl ImagePreviewState {
    /// Creates preview state from a dish without storing image data in UI state.
    ///
    /// The cache can re-resolve the texture from the dish ID when the modal is
    /// rendered. Keeping only IDs/names here keeps the state lightweight.
    pub fn from_dish(dish: &Dish) -> Self {
        Self {
            dish_id: dish.dish_id.clone(),
            dish_name: dish.name.clone(),
        }
    }
}

/// Draws one clickable local dish thumbnail or a stable "No image" placeholder.
///
/// Image rendering lives in the GUI layer, while file loading and texture
/// caching live in `image_loader`. The same helper is reused only in the two
/// requested customer-facing places: menu cards and recommendation cards.
pub fn render_dish_image_thumbnail(
    ui: &mut egui::Ui,
    image_cache: &mut DishImageCache,
    preview_state: &mut Option<ImagePreviewState>,
    dish: &Dish,
    size: f32,
) {
    let size = egui::vec2(size, size);

    if let Some(texture) = image_cache.texture_for_dish(ui.ctx(), dish) {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        let response = response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text("View larger image");

        let image_rect =
            egui::Rect::from_center_size(rect.center(), fit_image_size(texture.size_vec2(), size));

        ui.painter().rect_filled(rect, 8.0, egui::Color32::WHITE);
        ui.painter().image(
            texture.id(),
            image_rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        ui.painter()
            .rect_stroke(rect, 8.0, egui::Stroke::new(1.0, NEUTRAL_BORDER));

        if response.clicked() {
            *preview_state = Some(ImagePreviewState::from_dish(dish));
        }
    } else {
        draw_no_image_placeholder(ui, size);
    }
}

/// Draws the global image preview modal when a thumbnail has been selected.
///
/// The same modal is reused by menu and recommendation cards. It looks up the
/// dish in the loaded dataset, asks the image cache for the local texture, and
/// scales the image to fit inside the current egui window while preserving the
/// original aspect ratio.
pub fn render_image_preview_modal(
    ctx: &egui::Context,
    dishes: &[Dish],
    image_cache: &mut DishImageCache,
    preview_state: &mut Option<ImagePreviewState>,
) {
    let Some(preview) = preview_state.clone() else {
        return;
    };

    let mut open = true;
    let title = format!("{} ({})", preview.dish_name, preview.dish_id);

    egui::Window::new(title)
        .collapsible(false)
        .resizable(true)
        .default_width(720.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .open(&mut open)
        .show(ctx, |ui| {
            let Some(dish) = dishes.iter().find(|dish| dish.dish_id == preview.dish_id) else {
                ui.label("No image available.");
                return;
            };

            if let Some(texture) = image_cache.texture_for_dish(ctx, dish) {
                let max_size = preview_max_size(ctx);
                let image_size = fit_image_size(texture.size_vec2(), max_size);

                ui.vertical_centered(|ui| {
                    ui.add(egui::Image::new((texture.id(), image_size)));
                });
            } else {
                ui.label("No image available.");
            }
        });

    if !open {
        *preview_state = None;
    }
}

/// Draws a non-clickable placeholder for dishes without a local image.
///
/// Missing images should never block the FYP demo. The placeholder keeps card
/// spacing stable while clearly showing that no local asset is available.
fn draw_no_image_placeholder(ui: &mut egui::Ui, size: egui::Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect(
        rect,
        8.0,
        PLACEHOLDER_FILL,
        egui::Stroke::new(1.0, NEUTRAL_BORDER),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "No image",
        egui::FontId::proportional(12.0),
        MUTED_TEXT,
    );
}

/// Returns the largest preview size allowed inside the current window.
fn preview_max_size(ctx: &egui::Context) -> egui::Vec2 {
    let screen_size = ctx.screen_rect().size();
    egui::vec2(
        (screen_size.x * 0.72).max(260.0),
        (screen_size.y * 0.68).max(220.0),
    )
}

/// Fits an image inside a bounding box while preserving aspect ratio.
fn fit_image_size(original: egui::Vec2, max_size: egui::Vec2) -> egui::Vec2 {
    if original.x <= 0.0 || original.y <= 0.0 {
        return egui::vec2(320.0, 240.0);
    }

    let scale = (max_size.x / original.x)
        .min(max_size.y / original.y)
        .min(3.0);

    original * scale
}

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

/// Displays selected dishes as larger, prominent pills.
///
/// These are not checkout items. They show the dish IDs currently used as
/// recommendation input, so they need to stand out clearly during a demo.
pub fn selected_dish_pills(ui: &mut egui::Ui, values: &[String]) {
    ui.horizontal_wrapped(|ui| {
        for value in values {
            egui::Frame::none()
                .fill(ACCENT_BLUE_LIGHT)
                .stroke(egui::Stroke::new(1.0, ACCENT_BLUE))
                .rounding(12.0)
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(value)
                            .strong()
                            .color(egui::Color32::from_rgb(30, 64, 175)),
                    );
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
        (ACCENT_BLUE_LIGHT, egui::Stroke::new(1.0, ACCENT_BLUE))
    } else {
        (
            egui::Color32::from_rgb(248, 250, 252),
            egui::Stroke::new(1.0, NEUTRAL_BORDER),
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
    image_cache: &mut DishImageCache,
    preview_state: &mut Option<ImagePreviewState>,
    rank: usize,
    recommendation: &RecommendationResult,
    related_selected_dish_labels: &[String],
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        // Recommendation cards include thumbnails because this page is where
        // stakeholders inspect the recommendation output. The image is kept to
        // the left and all reasoning text stays on the right for readability.
        ui.horizontal(|ui| {
            render_dish_image_thumbnail(
                ui,
                image_cache,
                preview_state,
                &recommendation.dish,
                96.0,
            );
            ui.add_space(12.0);

            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading(format!("{}. {}", rank, recommendation.dish.name));
                    ui.monospace(format!("({})", recommendation.dish.dish_id));
                    ui.label(format!("Category: {}", recommendation.dish.category));
                });

                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    labelled_value(
                        ui,
                        "Final hybrid score:",
                        format!("{:.2}", recommendation.final_score),
                    );
                    ui.label("|");
                    labelled_value(
                        ui,
                        "Ingredient score:",
                        format!("{:.2}", recommendation.ingredient_score),
                    );
                    ui.label("|");
                    labelled_value(
                        ui,
                        "Co-order score:",
                        format!("{:.2}", recommendation.co_order_score),
                    );
                });

                ui.separator();
                ui.strong("Detailed reason:");
                ui.label(build_detailed_reason(
                    recommendation,
                    related_selected_dish_labels,
                ));
                labelled_value(ui, "Summary:", &recommendation.explanation);

                ui.add_space(4.0);
                labelled_value(
                    ui,
                    "Matched liked ingredients:",
                    display_list(&recommendation.matched_liked_ingredients),
                );
                labelled_value(
                    ui,
                    "Matched preferred tags:",
                    display_list(&recommendation.matched_preferred_tags),
                );
                labelled_value(
                    ui,
                    "Disliked ingredient check:",
                    if recommendation.matched_disliked_ingredients.is_empty() {
                        "No disliked ingredients matched; disliked dishes are excluded before ranking."
                            .to_string()
                    } else {
                        format!(
                            "Matched disliked ingredient(s): {}",
                            recommendation.matched_disliked_ingredients.join(", ")
                        )
                    },
                );
                labelled_value(
                    ui,
                    "Co-order influence from selected dish(es):",
                    display_list(related_selected_dish_labels),
                );
            });
        });
    });
}

/// Displays a bold label followed by a normal value.
///
/// Recommendation cards contain several score and reasoning fields. Separating
/// label styling from value styling makes each topic easier to scan quickly.
fn labelled_value(ui: &mut egui::Ui, label: &str, value: impl AsRef<str>) {
    ui.horizontal_wrapped(|ui| {
        ui.strong(label);
        ui.label(value.as_ref());
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
///
/// Empty recommendation fields use `-` instead of `(none)` because it is shorter
/// and reads better in score breakdowns.
pub fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(", ")
    }
}
