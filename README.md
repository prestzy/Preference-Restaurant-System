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
2. Loads dishes and orders from CSV.
3. Starts the Rust desktop GUI.

## Main Workflow

Open **Explore & Recommend**.

There you can:

- Browse the menu.
- Search by multiple terms.
- Select dishes directly from the menu cards.
- Enter liked ingredients, disliked ingredients, and preferred tags.
- See recommendations update automatically.

On wider windows the menu and recommendation panels appear side by side. On narrower windows the same sections stack vertically.

## Data Files

CSV files are stored in:

```text
data/dishes.csv
data/orders.csv
```

See [docs/DATA_FORMAT.md](docs/DATA_FORMAT.md) for exact CSV columns and examples.

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

The project keeps dependencies minimal. Recommendation logic remains custom Rust code and does not use heavy machine learning libraries.
