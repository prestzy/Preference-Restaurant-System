use crate::models::Dish;
use anyhow::{Context, Result};
use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Folder used for locally stored dish thumbnails.
///
/// The runtime GUI loads images from this folder only. Online URLs are kept in
/// `assets/dish_image_sources.csv` for traceability, not for hotlinking.
pub const DISH_IMAGE_DIR: &str = "assets/dishes";

/// In-memory image cache for egui textures.
///
/// egui textures should be created once and reused. This cache keeps image
/// loading separate from recommendation logic and prevents the app from reading
/// and decoding image files every frame.
#[derive(Default)]
pub struct DishImageCache {
    textures: HashMap<String, egui::TextureHandle>,
    missing_or_invalid: HashSet<String>,
}

impl DishImageCache {
    /// Creates an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a cached texture for a dish, loading it from disk if needed.
    ///
    /// `None` means the dish has no usable local image. The caller should show a
    /// lightweight placeholder instead of crashing or trying to fetch the web.
    pub fn texture_for_dish(
        &mut self,
        ctx: &egui::Context,
        dish: &Dish,
    ) -> Option<egui::TextureHandle> {
        if let Some(texture) = self.textures.get(&dish.dish_id) {
            return Some(texture.clone());
        }

        if self.missing_or_invalid.contains(&dish.dish_id) {
            return None;
        }

        let Some(path) = resolve_dish_image_path(dish) else {
            self.missing_or_invalid.insert(dish.dish_id.clone());
            return None;
        };

        match load_color_image(&path) {
            Ok(color_image) => {
                let texture = ctx.load_texture(
                    format!("dish-image-{}", dish.dish_id),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                self.textures.insert(dish.dish_id.clone(), texture.clone());
                Some(texture)
            }
            Err(_) => {
                // Invalid or unsupported image files are treated like missing
                // images. The GUI will still render a stable "No image" box.
                self.missing_or_invalid.insert(dish.dish_id.clone());
                None
            }
        }
    }
}

/// Ensures the local image folder exists.
///
/// This is called at startup so users can immediately see where dish images
/// should be placed even before any images are added.
pub fn ensure_dish_image_folder() -> Result<()> {
    fs::create_dir_all(DISH_IMAGE_DIR).context("failed to create assets/dishes folder")
}

/// Resolves the best local image path for a dish.
///
/// Priority:
/// 1. Use `image_path` from CSV if it exists and points to a real file.
/// 2. Try fallback filenames based on dish ID: `.jpg`, `.png`, then `.jpeg`.
/// 3. Return `None` so the UI can show the placeholder.
pub fn resolve_dish_image_path(dish: &Dish) -> Option<PathBuf> {
    if let Some(path) = dish.image_path.as_deref() {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    fallback_image_paths(&dish.dish_id)
        .into_iter()
        .find(|path| path.exists())
}

/// Builds the fallback dish-ID image paths supported by the prototype.
fn fallback_image_paths(dish_id: &str) -> Vec<PathBuf> {
    let dish_id = dish_id.trim().to_uppercase();
    ["jpg", "png", "jpeg"]
        .into_iter()
        .map(|extension| PathBuf::from(DISH_IMAGE_DIR).join(format!("{dish_id}.{extension}")))
        .collect()
}

/// Loads an image file into an egui `ColorImage`.
///
/// The `image` crate is used only for decoding local JPG/PNG files. This keeps
/// image support lightweight and compatible with the existing egui stack.
fn load_color_image(path: &Path) -> Result<egui::ColorImage> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let image = image::load_from_memory(&bytes)
        .with_context(|| format!("failed to decode {}", path.display()))?
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.into_raw();

    Ok(egui::ColorImage::from_rgba_unmultiplied(size, &pixels))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dish_with_image_path(path: Option<&str>) -> Dish {
        Dish {
            dish_id: "D999".to_string(),
            name: "Test Dish".to_string(),
            ingredients: vec!["rice".to_string()],
            category: "main".to_string(),
            tags: vec!["test".to_string()],
            image_path: path.map(str::to_string),
            image_source_url: None,
        }
    }

    #[test]
    fn missing_image_resolves_to_none() {
        let dish = dish_with_image_path(None);

        assert!(resolve_dish_image_path(&dish).is_none());
    }

    #[test]
    fn missing_explicit_image_path_falls_back_without_panicking() {
        let dish = dish_with_image_path(Some("assets/dishes/not-real.jpg"));

        assert!(resolve_dish_image_path(&dish).is_none());
    }
}
