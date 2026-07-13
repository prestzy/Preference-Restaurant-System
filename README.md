# Preference-Driven Restaurant Ordering System

Rust web prototype for a Final Year Project restaurant ordering and recommendation system.

The project direction is now QR-based ordering: customers scan a QR code with their own phone, open a responsive restaurant menu, choose preferences, receive explainable dish recommendations, add items to cart, and place a prototype order. This avoids the cost of placing one tablet at every table.

## Current Features

- Mobile-first orange/white customer menu.
- Search by dish name, dish ID, ingredient, category, or tag.
- Category chips for All, Main, Side, Appetizer, and Dessert.
- Local dish image support with a graceful placeholder.
- “Recommended for You” cards powered by Rust recommendation logic.
- Preference chips generated from the CSV dataset:
  - liked ingredients
  - disliked ingredients
  - preferred tags
- Cart with quantities, total price placeholder, and prototype checkout.
- Live in-memory orders created from checkout.
- Staff/admin page for dashboard metrics, live order status, dish management, CSV tools, and recommendation testing.
- CSV-based data loading, import, and export.

## Run Locally

```powershell
cargo run
```

Open:

```text
http://127.0.0.1:3000/
```

Useful routes:

```text
http://127.0.0.1:3000/        Customer menu
http://127.0.0.1:3000/cart    Cart and checkout
http://127.0.0.1:3000/orders  Customer order placeholder
http://127.0.0.1:3000/admin   Staff/admin tools
```

On startup the app creates `data/dishes.csv` and `data/orders.csv` if they are missing.

## Data and Images

CSV files:

```text
data/dishes.csv
data/orders.csv
```

Dish images:

```text
assets/dishes/
```

Image lookup order:

1. `image_path` column in `data/dishes.csv`, if present and the file exists.
2. `assets/dishes/{dish_id}.jpg`
3. `assets/dishes/{dish_id}.png`
4. `assets/dishes/{dish_id}.jpeg`
5. Orange/white placeholder if no local image exists.

Image sources can be documented in:

```text
assets/dish_image_sources.csv
```

See [docs/DATA_FORMAT.md](docs/DATA_FORMAT.md) for exact columns and examples.

## Recommendation Approach

The recommender stays lightweight and explainable:

- Ingredient/content filtering uses liked ingredients, disliked ingredients, and preferred tags.
- Collaborative filtering builds an item-item co-order matrix from `orders.csv`.
- Hybrid scoring combines content and co-order scores.
- Result cards and the admin recommendation tester show score breakdowns and plain-language explanations.

No heavy machine learning libraries are used.

## Project Structure

- `src/models.rs`: data structures only.
- `src/data_loader.rs`: CSV loading/import/export helpers.
- `src/recommender/`: content, collaborative, and hybrid recommendation logic.
- `src/web/state.rs`: shared web state and view-model preparation.
- `src/web/routes.rs`: Axum route declarations.
- `src/web/handlers/`: focused HTTP handlers for menu, cart, orders, admin, and recommendations.
- `src/web/templates.rs`: server-rendered HTML templates.
- `static/app.css`: orange/white responsive UI styling.
- `static/app.js`: lightweight browser behavior for search, preferences, cart, checkout, admin tools, and recommendation refresh.

Legacy desktop GUI files remain in `src/gui/` for reference, but `cargo run` now starts the web server.

## Validation

```powershell
cargo check
cargo test
```

Tests cover CSV parsing, search/filter logic, preference option extraction, recommendation behavior, checkout/live orders, admin availability, and dish management state.
