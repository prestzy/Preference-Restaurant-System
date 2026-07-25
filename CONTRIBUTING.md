# Contributing

This repository is an academic FYP prototype. Contributions should keep it
lightweight, deterministic, explainable, and suitable for a single SME
restaurant.

## Development Setup

1. Install stable Rust 1.85 or newer.
2. Clone the repository.
3. Set non-public admin credentials in the shell:

   ```powershell
   $env:ADMIN_USERNAME="local-staff"
   $env:ADMIN_PASSWORD="use-a-long-local-password"
   ```

4. Run `cargo run` and open `http://127.0.0.1:3000/`.

Do not commit `.env`, customer contact data, cookies, passwords, runtime order
details, or generated learning-event logs.

## Change Principles

- Keep recommendation formulas in Rust, not JavaScript.
- Preserve deterministic ordering and explicit tie-breakers.
- Treat disliked ingredients as hard exclusions.
- Keep production adaptive scoring separate from fixed controlled experiments.
- Never write simulated or counterfactual baskets to `data/orders.csv`.
- Keep the customer Menu static; search is a locator only.
- Prefer focused modules and existing dependencies.
- Document heuristic thresholds and avoid unsupported accuracy claims.

## Required Checks

```powershell
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

For interface changes, also test customer and admin flows at phone, tablet, and
desktop widths. See [docs/testing-guide.md](docs/testing-guide.md).

## Data Fixtures

Unit tests must use in-memory fixtures or temporary files. They must never
modify repository `data/orders.csv`. Sample records must be synthetic and must
not contain real customer names or phone numbers.

## Pull Requests

Keep pull requests focused. Explain scoring or persistence changes, list
commands actually run, include privacy-safe screenshots for UI changes, and
update the relevant guide when behaviour changes.
