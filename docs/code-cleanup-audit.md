# Code Cleanup Audit

This inventory records meaningful removals made during the production-quality
cleanup. An item is removed only when the module graph, Axum routes, template
references, JavaScript call sites, tests, and serialization use provide
reasonable evidence that it is unreachable or redundant.

| Removed item | Type | Why it was unused/redundant | Replacement, if any | Risk |
|---|---|---|---|---|
| `src/gui/` | Rust modules | These files implement the former `eframe/egui` desktop application. `main.rs` does not declare the module and `Cargo.toml` no longer contains its GUI dependencies, so the code cannot be compiled or reached by the web application. | The Axum web interface under `src/web/` and `static/`. | Low |
| `src/image_loader.rs` | Rust module | This is the former egui texture cache. It is absent from the module graph and depends on desktop-only image rendering concepts. | Browser-served local images resolved by the web state and `/assets` route. | Low |
| `src/simulation.rs` | Rust module | This desktop-era simulation service was declared only to satisfy the old GUI. The active Recommendation Tester uses isolated simulation methods in web state and never calls this module. | Web experiment and counterfactual services in `src/web/state.rs` and `src/recommender/`. | Low |
| `append_order_to_csv` | Rust function | It was called only by the removed desktop simulation module. Keeping it could also encourage simulated data to be written into real historical orders. | `append_completed_order_to_csv` persists only genuine completed web orders; experiments remain in memory. | Low |
| `CUSTOMER_KEY` and `writeCustomerIdentity` | JavaScript state helper | The customer cookie/session is authoritative, and no caller writes the legacy local-storage identity mirror. | Server-managed customer session exposed through the current page model. | Low |
| `dishSearchHaystack` and `dishMatchReason` | JavaScript search helpers | Both are declaration-only remnants of client-side filtering. Search suggestions now come from the Rust `/api/search` endpoint. | Rust search service plus `setupDishLocator`. | Low |
| `expandedSearchGroups` and `smartSearchResult` | JavaScript recommendation/search helpers | Both are declaration-only remnants of browser-side smart matching and duplicate server logic. | Rust search vocabulary and ranked search result API. | Low |
| `data/recommendation_feedback.csv` | Obsolete runtime data | The active application has no loader, route, or recommender input for this legacy feedback file. Its session identifier should not be tracked as sample data. | No replacement; feedback learning is not claimed by the current prototype. | Low |
| Tracked `data/order_details.csv` and timeline JSONL | Private/generated data | These runtime files can contain customer contact details and session-specific evidence, so committing them is unsafe. | Ignored runtime files plus the header-only `data/order_details.example.csv`. | Low |
| Hidden CSV maintenance page/import endpoints and browser preview code | Route, handlers, template, JavaScript | The visible CSV tools were intentionally retired in an earlier requirement, but a direct route and its implementation remained reachable. They duplicated startup loaders and created an undocumented mutation surface. | Startup CSV loading, focused Dish Management, completed-order persistence, and authenticated CSV export remain. | Low |
| Legacy evaluation handler, DTOs, and aggregate calculations | Rust route-adjacent code | The handler had no Axum route or browser caller. The current Recommendation Experiment Lab provides the maintained ingredient, co-order, and method-comparison workflows. | `experiment_lab`, adaptive inspector, counterfactual, and learning timeline endpoints. | Low |
| CSV-only CSS selectors | CSS | Their sole markup and JavaScript owner was the removed maintenance panel. | Shared admin cards/forms remain. | Low |
| Desktop-era `SearchFilter` and direct dish filter functions | Rust functions/types | Only their own tests used them; the web locator uses ranked `search_dishes`, and the Menu must never be filtered. | Ranked Rust suggestion search with `MatchMode::Any` and `MatchMode::All` tests. | Low |
| Repeated cookie parsing/building and timestamp-derived session IDs | Rust logic | Customer and admin handlers had similar cookie code and identifiers were predictable. | `web/session.rs` provides OS-random IDs and shared cookie policy. | Low |
| Repeated full-file replacement implementations | Rust persistence logic | Timeline and order-detail stores each implemented their own temporary-file behavior. | `persistence/atomic_file.rs` owns safe replacement. | Medium |
| Repeated per-candidate order scans | Recommendation calculation | Association, context frequency, popularity maxima, and pair maxima were recomputed in candidate loops. | One request-scoped `RecommendationScoringContext` and precomputed association indexes. | Medium |
| Repeated browser `fetch` response/error parsing | JavaScript | Different panels handled malformed/non-2xx responses inconsistently. | Shared `requestJson()` used by customer and admin requests. | Low |
| Fragmented legacy documentation | Documentation | Multiple documents described the retired desktop app or duplicated adaptive/experiment material. | Cohesive architecture, recommendation, tester, stakeholder, developer, data, testing, and design-system guides. | Low |

## Deliberately retained

- Compatibility redirects such as `/admin/evaluation` are retained because old
  bookmarks may still use them and they are harmless read-only redirects.
- CSS selectors are not removed from text-search evidence alone because classes
  are generated by Rust templates and JavaScript at runtime.
- Controlled experiment formulas remain separate from production adaptive
  scoring by design; this is research isolation, not accidental duplication.
- Tests and test-only fixture helpers remain even when production code does not
  call them.
- Anonymous historical baskets remain in `data/orders.csv`; removing private
  order-detail records does not erase valid recommendation evidence.

## Validation required after cleanup

The Rust cleanup is accepted only after `cargo fmt`, `cargo check`, strict
Clippy, and `cargo test` pass. Browser results and any residual limitations are
recorded separately in the final report.
