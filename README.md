# Preference-Driven Restaurant Ordering System

Rust desktop GUI prototype for a Final Year Project restaurant ordering and recommendation system.

The application helps a single restaurant demonstrate explainable dish recommendations using:

- Ingredient-based filtering from liked ingredients, disliked ingredients, and preferred tags.
- Co-ordering-based collaborative filtering from historical order logs.
- Hybrid scoring that combines both signals when both are available.

The app is intentionally lightweight. It uses CSV files, standard Rust collections, and `eframe/egui` for the desktop GUI. It does not use heavy machine learning libraries.

## Run

```powershell
cargo run
```

On startup the app:

1. Creates `data/dishes.csv` and `data/orders.csv` if they are missing.
2. Creates `assets/dishes/` if it is missing.
3. Loads dishes and orders from CSV.
4. Starts the Rust desktop GUI.

## Main Workflow

Open **Explore & Recommend**.

There you can:

- Browse the menu.
- Search by multiple terms.
- Select dishes directly from the menu cards.
- Select liked ingredients, disliked ingredients, and preferred tags from generated options.
- Open Evaluation to see updated recommendations and reasoning.

On wider windows the menu and preference/selected-dish panels appear side by side. On narrower windows the same sections stack vertically.

## Dish Images

Dish images are optional local assets stored in:

```text
assets/dishes/
```

The app never hotlinks online images at runtime. For each dish it first uses an optional `image_path` column from `data/dishes.csv`. If that is missing or points to a missing file, it tries these fallback names:

```text
assets/dishes/{dish_id}.jpg
assets/dishes/{dish_id}.png
assets/dishes/{dish_id}.jpeg
```

For example, `D01` can use `assets/dishes/D01.jpg`. If no file exists, the GUI shows a clean `No image` placeholder.

Images are shown only in:

- **Explore & Recommend** menu dish cards.
- **Evaluation** recommendation result cards.

They are not shown in Dashboard, Preference Panel, Admin / Demo Tools, or page headers because images are only meant to help customers recognize menu and recommended dishes.

Image source records are stored in:

```text
assets/dish_image_sources.csv
```

To replace or add an image, place a JPG or PNG file in `assets/dishes/` using the dish ID filename, then update `assets/dish_image_sources.csv` with the source URL, license, source page, and local path.

## Data Files

CSV files are stored in:

```text
data/dishes.csv
data/orders.csv
```

See [docs/DATA_FORMAT.md](docs/DATA_FORMAT.md) for exact CSV columns, optional image fields, and examples.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [User Guide](docs/USER_GUIDE.md)
- [Stakeholder Overview](docs/STAKEHOLDER_OVERVIEW.md)
- [Data Format](docs/DATA_FORMAT.md)

## Validation

Run:

```powershell
cargo test
cargo check
```

Tests cover multi-term filtering, match-any/match-all search logic, recommendation refresh behaviour, dish selection input, and order simulation updates.

## Dependency Note

The project keeps dependencies minimal. Recommendation logic remains custom Rust code and does not use heavy machine learning libraries. The `image` crate is used only to decode local JPG/PNG dish thumbnails for egui.
