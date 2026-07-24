# Preference-Driven Restaurant Ordering System

Rust web prototype for a Final Year Project restaurant ordering and recommendation system. Customer pages use the restaurant brand **Preston's Restaurant**; admin research pages retain the academic project name.

The project direction is now QR-based ordering: customers scan a QR code with their own phone, open a responsive restaurant menu, choose preferences, receive explainable dish recommendations, add items to cart, and place a prototype order. This avoids the cost of placing one tablet at every table.

## Current Features

- Mobile-first orange/white customer menu.
- Separate customer and staff/admin interfaces.
- Prototype admin login using `ADMIN_USERNAME` and `ADMIN_PASSWORD`.
- Unified search and Smart Menu Assistant input for dish keywords or phrases such as `spicy chicken but no beef`.
- Smart Search by dish name, dish ID, ingredient, category, tag, alias, and curated food concept, with live suggestions and match reasons.
- Local dish image support with a graceful placeholder.
- “Recommended for You” cards powered by Rust recommendation logic.
- Preference chips generated from the CSV dataset:
  - liked ingredients
  - disliked ingredients
  - preferred tags
- First-stage temporary customer registration at `/start` with name, phone number, and table number.
- Cart with quantities, total price placeholder, optional order note, and prototype checkout.
- Checkout uses the server-side customer session instead of asking for contact details again.
- Customer Profile page tracks only that customer's active and completed/cancelled session orders.
- Customer Profile polls order status automatically and updates without a page refresh.
- Operational order details are stored separately in `data/order_details.csv`.
- Completed checkout orders are appended to `data/orders.csv`, then shown immediately in Historical Orders and reused by future recommendation calculations.
- Smart Menu Assistant that parses simple customer text such as “spicy chicken but no beef” into structured recommendation preferences.
- “Why recommended?” explanations remain available on recommendation cards and dish details.
- Staff/admin page for dashboard metrics, live order status, dish management, historical orders, and recommendation testing.
- Admin Orders polls live order status automatically and remains protected by staff login.
- Dish Management now starts with search/filter/list controls and opens Add/Edit in a modal form.
- Recommendation Tester includes controlled in-memory random co-order simulation for limited-data demonstrations.
- Simulation data does not modify `data/orders.csv`.
- Recommendation Experiment Lab contains three controlled experiments: Ingredient Impact, Co-Order Impact, and Method Comparison.
- Stakeholder instructions are available in `docs/recommendation-experiment-lab-manual.md`.
- CSV-based data loading, import, and export.
- Popularity fallback so recommendations do not go empty when preference input is limited.
- Association-rule metrics for co-ordering: support, confidence, and lift.
- Simple time-context boost for breakfast, lunch, dinner, and snack/dessert testing.

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
http://127.0.0.1:3000/start   Temporary customer registration
http://127.0.0.1:3000/        Customer menu, after registration
http://127.0.0.1:3000/profile Customer profile and order tracking
http://127.0.0.1:3000/cart    Cart and checkout
http://127.0.0.1:3000/orders  Redirects to /profile
http://127.0.0.1:3000/admin/login Staff login
http://127.0.0.1:3000/admin   Protected staff/admin tools
```

On startup the app creates `data/dishes.csv` and `data/orders.csv` if they are missing.

Admin credentials are read from environment variables:

```powershell
$env:ADMIN_USERNAME="admin"
$env:ADMIN_PASSWORD="change-me"
cargo run
```

When both variables are absent in a debug build, the documented local fallback
is `admin` / `admin`. If only one variable is set, empty, or incomplete, the
login page reports that server credentials are not configured. This remains
prototype-level access control, not production security.

### Test From a Phone

Bind the server to all local network interfaces:

```powershell
$env:APP_HOST="0.0.0.0"
$env:APP_PORT="3000"
$env:ADMIN_USERNAME="admin"
$env:ADMIN_PASSWORD="change-me"
cargo run
```

Find the computer's LAN IPv4 address with `ipconfig`, then open
`http://<computer-lan-ip>:3000/` on a phone connected to the same network.
Allow TCP port 3000 through Windows Firewall if prompted. Browser requests use
relative URLs, so registration and admin login stay on the phone-visible host.

Local HTTP cookies are host-only, `HttpOnly`, `SameSite=Lax`, and intentionally
omit `Secure`. Set `APP_COOKIE_SECURE=true` only when the application is served
through HTTPS. Customer and admin sessions use separate cookies, so either
session can be ended without clearing the other.

## Data and Images

CSV files:

```text
data/dishes.csv
data/orders.csv
data/order_details.csv
data/search_aliases.csv
```

`data/orders.csv` remains recommendation-history only:

```text
order_id,session_user_id,ordered_dishes,timestamp
```

Customer name, phone, table number, order notes, and live status are stored in `data/order_details.csv`, not in recommendation history.

Data roles:

- `data/orders.csv`: real completed historical order baskets used by recommendation logic.
- `data/order_details.csv`: operational customer order details and live/completed status.
- `data/recommendation_feedback.csv`, when present, is archived legacy prototype data. The current application does not load, update, or evaluate it.
- In-memory simulation orders: generated only inside Recommendation Tester and never appended to real history.
- `data/search_aliases.csv`: maintainable food-domain search aliases such as `mee -> noodle`, `ayam -> chicken`, and `pisang -> banana`.

## Smart Search

Smart Search is a lightweight rule-based retrieval layer. It is separate from the recommendation score.

Pipeline:

```text
raw query -> normalisation -> multi-term parsing -> alias expansion -> concept expansion -> weighted matching -> match reason
```

Examples:

- `spicy, mee` can match Laksa because `mee` expands to noodle/noodles and spicy expands to chili/sambal/curry-related terms.
- `fruit` can match Pisang Goreng because `pisang -> banana` and banana is part of the curated fruit concept.
- `ayam` can match chicken dishes through the alias dictionary.

These mappings are curated rules, not AI-generated facts. Unknown terms simply behave as normal keyword search terms.

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
- Popularity fallback uses historical/completed order frequency.
- Association metrics calculate support, confidence, and lift for selected dish to candidate dish.
- Time-context rules add a small explainable business boost.
- Hybrid scoring uses:

```text
0.45 content + 0.25 co-order + 0.20 popularity + 0.10 time/business
```

- Result cards and the admin recommendation tester show score breakdowns, association metrics, and plain-language explanations.
- The Smart Menu Assistant is rule-based and is now merged with the main customer search input. It only extracts ingredients, tags, categories, and dish names that exist in the loaded menu vocabulary. No external LLM API is required.
- Recommendation Experiment Lab:
  - Ingredient Impact shows how liked/disliked ingredients alter ingredient scores and exclusions.
  - Co-Order Impact adds temporary in-memory co-orders to show pair-count/ranking sensitivity.
  - Method Comparison compares controlled experiment settings: Ingredient-only `1.0/0.0`, Co-order-only `0.0/1.0`, and Hybrid `0.4/0.6`.
- Production customer ranking remains `0.45 content + 0.25 co-order + 0.20 popularity + 0.10 time/business`.

No heavy machine learning libraries are used.

## Project Structure

- `src/models.rs`: data structures only.
- `src/data_loader.rs`: CSV loading/import/export helpers.
- `src/persistence/`: operational order-detail CSV persistence.
- `src/agent/`: rule-based Smart Menu Assistant preference parser.
- `src/recommender/`: content, collaborative, association metrics, popularity, time context, and hybrid recommendation logic.
- `src/web/state.rs`: shared web state and view-model preparation.
- `src/web/routes.rs`: Axum route declarations.
- `src/web/handlers/`: focused HTTP handlers for menu, cart, orders, admin, assistant, and recommendations.
- `src/web/templates.rs`: server-rendered HTML templates.
- `static/app.css`: orange/white responsive UI styling.
- `static/app.js`: lightweight browser behavior for search, preferences, cart, checkout, admin tools, and recommendation refresh.

Legacy desktop GUI files remain in `src/gui/` for reference, but `cargo run` now starts the web server.

## Validation

```powershell
cargo check
cargo test
```

Tests cover CSV parsing/validation, persistent completed-order append, operational order details, smart-search vocabulary/normalisation, search behavior, preference option extraction, assistant parsing, recommendation behavior, popularity fallback, association metrics, hybrid scoring, registration/session checkout, customer-scoped order lookup, completed order lifecycle, image fallback, admin availability, dish management state, and in-memory recommendation simulation.

## Limitations

- This is an FYP prototype, not a production POS system.
- No real payment processing is implemented.
- Admin login is lightweight prototype access control.
- Customer identity is temporary, stored in server memory plus an HTTP-only session cookie, and intended only for order operations.
- Refreshing keeps the session while the server is running; restarting the server clears active customer sessions but keeps persisted order details.
- Recommendation data is limited to the CSV dataset and completed prototype orders.
- Evaluation metrics are controlled system/proxy metrics and do not claim commercial recommendation accuracy.
