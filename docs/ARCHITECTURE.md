# Architecture

This repository is a Rust desktop GUI prototype, not a web app. The design goal is high cohesion and low coupling: each module owns one clear responsibility.

## Application Flow

1. `main.rs` creates missing sample data.
2. `data_loader.rs` reads CSV files into raw row models, then cleans them into system models.
3. `image_loader.rs` ensures the local dish image folder exists and later caches local thumbnails.
4. `gui::RestaurantOrderingApp` starts the egui desktop application.
5. `gui::state::AppState` owns mutable UI state and triggers recommendation refreshes.
6. `recommender::hybrid` generates ranked recommendation results using ingredient and collaborative scores.
7. `gui::pages` renders Dashboard, Explore & Recommend, Evaluation, and Admin / Demo Tools pages.

## Module Responsibilities

### `models.rs`

Data structures only:

- `DishRow`
- `Dish`
- `OrderRow`
- `Order`
- `UserPreference`
- `RecommendationResult`

`DishRow` and `OrderRow` represent raw CSV rows. `Dish` and `Order` are cleaned models used by the app.

`Dish` also stores optional `image_path` and `image_source_url` values. These are data fields only; image loading is handled by `image_loader.rs`.

### `data_loader.rs`

CSV and file persistence only:

- Generate sample CSV files if missing.
- Load dishes and orders.
- Clean comma-separated ingredients, tags, and dish IDs.
- Append simulated orders to `orders.csv` when enabled.

### `recommender/ingredient_filter.rs`

Content-based recommendation logic:

- Detect disliked ingredients.
- Score liked ingredient matches.
- Add preferred tag bonus.
- Build plain-language ingredient explanations.

### `recommender/collaborative_filter.rs`

Co-ordering-based collaborative filtering:

- Builds an item-item co-order matrix from historical orders.
- Counts dishes that appear together.
- Normalises co-order score between `0.0` and `1.0`.

### `recommender/hybrid.rs`

Recommendation orchestration:

- Applies disliked ingredient exclusion.
- Calculates ingredient and co-order scores.
- Combines them into a hybrid score.
- Stores matched liked ingredients, matched preferred tags, related selected dish IDs, and simple evaluation stats for transparent Evaluation-page explanations.

Default hybrid formula:

```text
final_score = 0.4 * ingredient_score + 0.6 * co_order_score
```

Adaptive behaviour:

- If no selected dishes are entered, ingredient score is used more heavily.
- If no preferences are entered, co-order score is used more heavily.
- If both are available, the normal hybrid score is used.

### `search.rs`

Menu filtering service:

- Parses multi-term search input.
- Supports `Match Any` and `Match All`.
- Searches dish ID, name, category, ingredients, and tags.

This is separate from UI code so it can be tested directly.

### `preferences.rs`

Selectable preference option extraction:

- Extracts all unique ingredients from loaded dishes.
- Extracts all unique tags from loaded dishes.
- Normalizes options by trimming and lowercasing.
- Sorts options alphabetically for predictable GUI display.

This keeps preference option generation outside rendering code.

### `image_loader.rs`

Local dish image loading and caching:

- Creates `assets/dishes/` at startup.
- Uses optional `image_path` when present.
- Falls back to `assets/dishes/{dish_id}.jpg`, `.png`, and `.jpeg`.
- Decodes local JPG/PNG files into egui textures.
- Caches textures by dish ID so images are not reloaded every frame.
- Returns a missing-image state so the GUI can show a `No image` placeholder.

This module does not contain CSV parsing, recommendation scoring, image downloading, or UI page logic.

### `simulation.rs`

Admin/demo order simulation:

- Parses manually entered dish IDs for the admin/demo simulation tool.
- Validates them against known menu IDs.
- Adds a simulated order to in-memory order history.
- Optionally appends the order to `data/orders.csv`.

Simulation is intentionally outside normal menu browsing because it represents demo/testing behaviour, not a normal customer action.

### `gui/`

GUI modules:

- `app.rs`: eframe application loop and navigation.
- `state.rs`: UI state and refresh methods.
- `pages.rs`: page rendering.
- `components.rs`: reusable visual helpers.
- `mod.rs`: module exports.

Dish thumbnails are rendered only by customer-facing components used in:

- Explore & Recommend menu cards.
- Evaluation recommendation result cards.

Dashboard, preferences, admin/demo tools, and page headers do not call the image loader.

## Responsive Layout

The main input workflow is **Explore & Recommend**.

- Wide windows show menu browsing beside preference and cart panels.
- Narrow windows stack the same panels vertically.
- Scroll areas prevent the interface from overflowing on common laptop resolutions such as 1366x768.
- Recommendation results are shown on **Evaluation** so input collection and output analysis stay visually separate.

The layout uses a small threshold only to decide whether to split into columns. The content itself uses flexible egui sizing and scroll areas.

## Extending the System

Recommended extension points:

- Add more dish rows in `data/dishes.csv`.
- Add more historical orders in `data/orders.csv`.
- Adjust scoring constants in `recommender/hybrid.rs` if the FYP evaluation requires different weights.

Avoid placing recommendation logic inside GUI rendering functions. Add or update services first, then call them from `gui::state` or `gui::pages`.
