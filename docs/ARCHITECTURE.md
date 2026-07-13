# Architecture

This repository is a lightweight Rust web prototype for QR-based restaurant ordering. The design goal is high cohesion and low coupling: each module owns one clear responsibility.

## Runtime Flow

1. `main.rs` creates sample CSV files if needed.
2. `data_loader.rs` loads and cleans dishes and historical orders.
3. `WebState` stores dishes, historical orders, live checkout orders, and in-memory dish availability.
4. `web::routes` maps URLs to focused handlers.
5. `web::handlers::*` process HTTP requests.
6. `web::templates` renders server-side HTML.
7. `static/app.js` handles browser-only interaction: live search suggestions, chips, cart quantities, modal details, checkout, admin CSV file preview, admin actions, and recommendation refresh.
8. `recommender::*` remains the only place where recommendation scoring is calculated.

## Module Responsibilities

### `models.rs`

Data structures only:

- `DishRow` and `OrderRow` for raw CSV rows.
- `Dish` and `Order` for cleaned app models.
- `UserPreference` for recommendation input.
- `RecommendationResult` for explainable ranking output.

### `data_loader.rs`

CSV and file handling only:

- Generates sample data if files are missing.
- Loads dishes and orders from CSV.
- Reuses the same parsers for admin CSV import.
- Validates required CSV headers before import.
- Exports current in-memory dishes, historical orders, and completed session orders as CSV.

### `recommender/`

Recommendation logic only:

- `ingredient_filter.rs`: liked ingredient, disliked ingredient, and tag scoring.
- `collaborative_filter.rs`: item-item co-order frequency matrix.
- `hybrid.rs`: adaptive hybrid scoring and recommendation ranking.

The web layer passes cleaned `UserPreference` values into these modules. The recommender does not know about HTML, images, routes, or cart rendering.

### `web/state.rs`

Web-facing application state:

- Holds loaded dishes and historical order logs.
- Holds live in-memory orders created by checkout, including dish names, totals, and status.
- Tracks dish availability for admin management.
- Converts domain models into frontend-friendly view models.
- Resolves local dish image URLs.
- Builds recommendation API responses with detailed explanations.

### `web/routes.rs`

URL declaration only. It does not render HTML or implement business logic.

### `web/handlers/`

Focused request handlers:

- `menu.rs`: customer menu page.
- `cart.rs`: cart page and checkout endpoint.
- `orders.rs`: customer orders page.
- `admin.rs`: admin dashboard, order status updates, and dish management.
- `recommendations.rs`: recommendation API bridge.

### `web/templates.rs`

Server-rendered HTML:

- Customer menu.
- Preference panel.
- Recommendation cards.
- Dish cards and dish detail modal.
- Cart/orders pages.
- Admin dashboard, live orders, dish management, historical orders, and recommendation tester.

Templates receive prepared view models. They do not load CSV files or run recommendation algorithms.

### `static/`

Frontend assets:

- `app.css`: orange/white mobile-first theme.
- `app.js`: small browser controller for interaction.

The frontend is intentionally simple. There is no heavy JavaScript framework.

## Data Boundaries

- CSV loading and export stay in `data_loader.rs`.
- Recommendation scoring stays in `recommender/`.
- Mutable web session state stays in `WebState`.
- HTTP request handling stays in `web/handlers/`.
- HTML construction stays in `web/templates.rs`.

This separation keeps the FYP prototype explainable and easier to extend.

## Current Persistence Model

- Startup data comes from `data/dishes.csv` and `data/orders.csv`.
- Admin dish changes are in memory for the running server session.
- Admin CSV export downloads the current in-memory state.
- Checkout creates live in-memory orders.
- Completed checkout orders are appended immediately to the in-memory historical order log and included as session co-order evidence.
- Historical order CSV import/reload replaces historical logs in memory and immediately affects collaborative/hybrid recommendation results.

This is deliberate for a prototype: it demonstrates the workflow without adding database complexity.

## Extension Points

- Add a real `price` column to `dishes.csv`.
- Persist admin dish changes back to CSV automatically.
- Add QR/table/session identifiers.
- Store checkout orders in a database.
- Add authentication for admin pages.
- Add richer evaluation metrics for the FYP report.
