# Testing Guide

## Automated Quality Gate

Run from repository root:

```powershell
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Tests live beside their owning modules. They cover CSV compatibility and
validation, normalisation/search, sessions, authorization, checkout, order
completion, static Menu behavior, recommendation formulas, adaptive boundaries,
evidence bands, diversity, meal sets, simulations, counterfactuals, and timeline
isolation.

## Safe Test Environment

Browser tests that complete orders can append historical CSV. To protect fixture
data:

1. Build the current binary.
2. Create a temporary runtime directory.
3. Copy `data/`, `assets/`, and `static/` into it.
4. Launch the binary with that temporary directory as its working directory.
5. Use a non-production `APP_PORT` and synthetic credentials/data.

Never use real phone numbers or customer names.

## Customer Flow Checklist

1. Open `/`; verify unregistered users reach `/start`.
2. Submit invalid fields; verify specific errors and preserved safe values.
3. Register synthetic details.
4. Confirm Home shows all available dishes.
5. Type `laksa`; suggestions should update while the permanent Menu count and
   order remain unchanged.
6. Click a suggestion; verify smooth scroll and temporary highlight.
7. Select liked/disliked/tag chips; verify liked/disliked conflict prevention.
8. Select a dish context; confirm recommendations refresh.
9. Compare confidence explanation and diversity modes.
10. Generate and clear a meal set.
11. Add multiple quantities to Cart; verify line subtotal and total.
12. Checkout; verify order appears in Orders/Profile.
13. From admin, update status; verify customer status refresh.

## Admin Flow Checklist

1. Open `/admin`; log in with environment credentials.
2. Verify customer cookie alone cannot access admin APIs.
3. Review dashboard counts, popular dishes, and named pairs.
4. Review live, completed-session, and CSV historical orders.
5. Move a temporary order Pending -> Preparing -> Ready -> Completed.
6. Verify one historical append, immediate count/insight refresh, and no duplicate
   append after a repeated Completed request.
7. Add/edit a synthetic dish, change availability, preview its image, export,
   and delete only when unreferenced.
8. Run Adaptive, Confidence, Diversity, and Meal Set tools.
9. Run Ingredient Impact, Co-Order Impact, and Method Comparison.
10. Run What Would Change? and simulation; historical count must not change.
11. Clear learning timeline; historical count must not change.
12. Rebuild timeline; verify deterministic restoration.
13. Log out; verify the login page links back to the customer menu.

## Viewport Matrix

| Viewport | Primary checks |
|---|---|
| 360x800 | one-column content, search dropdown, bottom nav, no page overflow |
| 390x844 | customer happy path, cards, preference chips |
| 414x896 | recommendation rail swipe, modal fit |
| 768x1024 | tablet columns and admin controls |
| 820x1180 | tablet table containment |
| 1024x1366 | admin tester navigation/workspace |
| Desktop | carousel arrows, max-width, table actions, modal focus |

For each width, inspect `document.documentElement.scrollWidth <=
document.documentElement.clientWidth`. Horizontal recommendation rails and table
wrappers may scroll locally.

## Recommendation Characterization

Check:

- identical request produces identical ranking;
- disliked ingredients always exclude;
- context selection changes co-order input;
- no history gives finite deterministic fallback;
- adaptive weights sum to one;
- co-order weight rises with repeated relevant evidence;
- confidence remains separate from score;
- all diversity modes preserve the eligible candidate set;
- meal sets respect budget and restrictions;
- experiment/simulation/counterfactual history counts do not change.

## Persistence Checks

After one synthetic completion in a temporary runtime:

- new row uses `Oxxx,Uxx,"Dxx,...","YYYY-MM-DD HH:MM"`;
- no `WEB`, `QR-CUSTOMER`, or `unix:` value is written;
- completing the same live order again adds no row;
- restart reloads the appended basket;
- dashboard counts and co-order/popularity calculations include it.

## Accessibility Checks

- Complete customer and admin navigation using keyboard.
- Confirm visible focus.
- Confirm tab state is announced.
- Open/close dish dialog with keyboard and verify focus returns.
- Confirm icon-only controls have accessible names.
- Confirm errors/status changes use text, not colour only.

## Performance Checks

This small dataset does not justify a benchmark framework. For repeatable local
measurements:

1. Use a release build.
2. Warm the endpoint once.
3. Record at least 20 same-origin requests for production Top-K, meal sets,
   method comparison, counterfactual, and timeline rebuild.
4. Report median and slowest observation with hardware, dataset counts, build
   profile, and date.
5. Keep measurement claims environment-specific.

The main code-level optimization is structural: one recommendation request
builds popularity/co-order/evidence indexes once rather than once per candidate.
Do not turn a debug localhost latency into a production SLA.

### Recorded Local Measurement

On 2026-07-25, a Windows debug build with 30 dishes and a 60-historical-basket
QA snapshot was exercised through a headless Chromium browser on loopback. Each
operation received one warm-up request. These numbers describe that development
machine and snapshot, not a production guarantee:

| Operation | Samples | Median | Maximum |
|---|---:|---:|---:|
| Production Top-K plus Balanced diversity | 20 | 6.6 ms | 9.0 ms |
| Budget meal-set generation | 20 | 17.2 ms | 17.9 ms |
| Three-method hidden-target comparison | 20 | 4.8 ms | 5.7 ms |
| Production counterfactual comparison | 20 | 7.8 ms | 8.6 ms |
| Timeline rebuild from the 60-basket QA snapshot | 5 | 51.6 ms | 53.6 ms |

The timeline sample is intentionally smaller because it rewrites a derived
JSONL file on every run. Release builds and different hardware must be measured
separately before making any service-level claim.

## Documentation Checks

Run a relative-link checker or manually verify every Markdown link. Confirm:

- the `admin` / `admin` FYP demo default and production override warning agree
  across the README, developer guide, and code;
- no fake screenshots or coverage badges;
- no accuracy/probability claim is attached to evidence confidence;
- deleted desktop modules are not described as active;
- document names match case-sensitive GitHub paths.
