# Developer Guide

## Prerequisites

- Stable Rust 1.85 or newer
- Git
- A modern browser
- PowerShell or another shell for environment variables

## Setup

```powershell
git clone https://github.com/prestzy/Preference-Restaurant-System.git
cd Preference-Restaurant-System
$env:ADMIN_USERNAME="restaurant-admin"
$env:ADMIN_PASSWORD="choose-a-long-local-password"
cargo run
```

Use `APP_PORT` to avoid a busy port. Use `APP_HOST=0.0.0.0` only when LAN access
is required.

## Configuration

| Environment variable | Requirement |
|---|---|
| `ADMIN_USERNAME` | Required to log in as staff |
| `ADMIN_PASSWORD` | Required; never commit it |
| `APP_HOST` | Optional, default `127.0.0.1` |
| `APP_PORT` | Optional, default `3000` |
| `APP_COOKIE_SECURE` | Enable when HTTPS terminates at the app |

The project intentionally does not parse a `.env` file. Set variables in the
process environment or deployment service.

## Project Structure

- `main.rs`: startup only.
- `models.rs`: cleaned and raw CSV domain models.
- `data_loader.rs`: sample creation, CSV parse/export, completed append.
- `search.rs`: menu-aware locator ranking and concepts.
- `preferences.rs`: available ingredient/tag vocabulary.
- `agent/`: deterministic Smart Menu Assistant parsing.
- `recommender/`: algorithms and explanations.
- `persistence/`: atomic replacement, detail store, learning events.
- `web/routes.rs`: route registration.
- `web/handlers/`: domain HTTP handlers.
- `web/state.rs`: synchronized application state and service orchestration.
- `web/session.rs`: cookie/session primitives.
- `web/validation.rs`: repeated customer input validation.
- `web/templates.rs`: server-rendered page components.
- `static/`: browser CSS and JavaScript.

## Application State

`WebState` wraps shared state used by Axum handlers. Read locks should be held
only long enough to clone/build the required view. Write locks protect order,
dish, session, and timeline mutation. Recommendation algorithms receive slices
and request structs rather than depending on Axum.

Do not place new formulas in `WebState`; add them to a focused recommender
module and call that module from state orchestration.

## Route Organisation

Routes are grouped conceptually by customer, recommendation, orders, and admin.
Use:

- `GET` for reads;
- `POST` for create/calculate;
- `PATCH` for partial state changes;
- `PUT` for full replacement; and
- `DELETE` for deletion.

Add explicit Serde request/response types near the owning state/handler API.
Protect all `/api/admin/*` mutations with admin session validation.

## Session Model

Customer and admin sessions are independent in-memory maps with different
cookie names. Session identifiers come from the operating system random source.
Use helpers in `web/session.rs`; do not create ad hoc cookies or predictable
IDs. Customer order reads must remain ownership-scoped.

Restarting the process ends browser sessions and removes uncompleted in-memory
orders. This is expected in the prototype.

## Persistence

- Parse initial dishes/orders through `data_loader`.
- Append a real order to `orders.csv` only on its first transition to
  `Completed`.
- Never write simulated/counterfactual baskets to real history.
- Use `atomic_file::replace_file` for full-file rewrites.
- Sync append-only writes before reporting success.
- Treat learning events as derived data.

## Recommendation Pipeline

1. Normalise and validate preference/context input.
2. Exclude unavailable, selected, and disliked-ingredient dishes.
3. Build one `RecommendationScoringContext`.
4. Calculate component scores and candidate evidence.
5. Combine with request-level adaptive weights.
6. Sort deterministically.
7. Apply diversity reranking and Top-K.
8. Build plain-language view explanations.

See [recommendation-system.md](recommendation-system.md).

## Adding a Dish Field

1. Decide whether it is domain data or presentation-only.
2. Add an optional/defaulted field to `DishRow` for CSV compatibility.
3. Add the cleaned type to `Dish`.
4. Update loader validation/conversion and export.
5. Update admin request/view forms.
6. Update data documentation.
7. Test both legacy CSV and the new column.

Use integer cents for a future persisted price rather than floating-point RM.

## Adding a Recommendation Signal

1. State the evidence source and its limitation.
2. Add a focused module with a normalised `[0,1]` result.
3. Calculate reusable indexes once in `RecommendationScoringContext`.
4. Add a documented adaptive/config weight.
5. Keep score separate from confidence.
6. Add characterization, boundary, determinism, and empty-data tests.
7. Update explanations, API response, tester, and technical documentation.

## Adding an Experiment

Experiments must:

- clone data for temporary changes;
- use explicit controlled inputs/weights;
- return before/after or comparable method output;
- avoid `orders.csv` and timeline writes;
- document safe and unsafe conclusions; and
- include a non-mutation test.

Add the state operation, protected handler/route, accessible tester panel, JS
binding, tests, and guide chapter together.

## Adding an Admin Tool

1. Add an authenticated route with the correct HTTP method.
2. Keep validation/business logic outside template rendering.
3. Use `requestJson()` in the browser.
4. Restore disabled/loading state in `finally`.
5. Render inline errors or a toast; do not rely on `alert()`.
6. Test authorization and the domain result.

## Testing

```powershell
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Run browser checks from [testing-guide.md](testing-guide.md). For persistence QA,
use a copied runtime data directory so test completion does not alter repository
fixtures.

## Debugging

- Port access denied/busy: stop only the known process or set another
  `APP_PORT`; do not delete a running Windows executable.
- Startup CSV error: read the contextual row/column message and validate IDs.
- Admin configuration error: set both admin variables before launch.
- Browser API failure: inspect Network and Console; `requestJson()` reports
  malformed and non-2xx responses.
- Timeline warning: use protected Rebuild Timeline; historical orders remain
  authoritative.

## Safe Data Reset

1. Stop the server.
2. Back up private runtime files if required.
3. Remove ignored `data/order_details.csv`.
4. Remove ignored learning-event JSONL only if the explanatory timeline should
   reset.
5. Do not delete `data/orders.csv` unless deliberately resetting historical
   evidence.
6. Restart and use Rebuild Timeline when needed.

## Coding Conventions

- Format with rustfmt and keep strict Clippy clean.
- Private by default; expose only intentional module interfaces.
- Prefer focused domain helpers over a generic `utils.rs`.
- Explain heuristics and data-safety boundaries, not obvious syntax.
- Keep identical requests deterministic.
- Avoid `unwrap`, `expect`, and `panic` in request/persistence paths.
- Use semantic CSS tokens and 44px touch targets.
- Keep search as a locator; the customer Menu must stay complete and static.

