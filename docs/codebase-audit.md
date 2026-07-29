# Codebase Audit

This audit records the pre-refactor findings and resulting action. Evidence was
collected from the Rust module graph, Axum router, templates, browser assets,
data files, Cargo metadata/tree, strict Clippy, tests, and repository searches.
Deletion decisions are itemised separately in
[code-cleanup-audit.md](code-cleanup-audit.md).

| Category | Finding | Severity | Evidence | Recommended action / status | Files involved |
|---|---|---:|---|---|---|
| Unused Rust code | Former egui application and texture loader were absent from the compiled module graph. | Medium | `src/gui/*`, `src/image_loader.rs`; no `mod gui`, no eframe dependency. | Removed; web app is the only active UI. | `src/gui/*`, `src/image_loader.rs`, `main.rs` |
| Unused Rust code | A desktop simulation writer could append synthetic orders to real CSV. | High | Only old GUI called `simulation.rs`/`append_order_to_csv`. | Removed; maintained simulations clone history. | `src/simulation.rs`, `data_loader.rs` |
| Unused JavaScript | Legacy local identity and browser-side search formula helpers had no callers. | Low | Declaration/call-site search in `static/app.js`. | Removed; server session/search are authoritative. | `static/app.js` |
| Unused CSS | Hidden CSV-tool styles had no remaining template/JS owner. | Low | Route/template/selector cross-check. | Removed with retired CSV maintenance surface. | `static/app.css` |
| Duplicate logic | Cookie parse/build differed between customer and admin handlers. | High | Repeated header parsing and cookie strings. | Consolidated with separate cookie names and shared policy. | `web/session.rs`, handlers |
| Duplicate logic | Full-file replacement logic existed in multiple persistence stores. | High | Learning and order-detail rewrite paths. | Shared atomic replacement helper. | `persistence/atomic_file.rs` |
| Duplicate logic | Frontend request/error parsing varied by panel. | Medium | Multiple direct `fetch` paths with different checks. | Shared `requestJson()` and consistent failure restoration. | `static/app.js` |
| Oversized modules | `web/state.rs`, templates, JS, and CSS remain concentration points. | Medium | Approx. 3,856, 1,932, 2,400+, and 2,500+ lines during audit. | Extracted sessions, validation, and persistence; further split remains incremental debt. | `web/state.rs`, `web/templates.rs`, `static/*` |
| Weak naming/comments | Domain comments still described the retired GUI. | Low | Rustdoc text search. | Updated to web/application terminology. | `models.rs`, `preferences.rs`, ingredient filter |
| Error handling | File replacement and browser response handling could fail inconsistently. | High | Ad hoc rename/write and direct JSON parsing. | Atomic helper, fsync, contextual Rust errors, shared JS error handling. | persistence, `app.js` |
| Error handling | Poisoned in-process locks use invariant `expect` messages. | Medium | `WebState` lock access. | Retained as process-invariant failures; a future state-store abstraction should reduce repetition. | `web/state.rs` |
| State management | Session IDs were timestamp-derived and order APIs accepted arbitrary phone lookup. | Critical | Handler/session generation and query fallback. | OS-random IDs; order ownership comes only from authenticated customer session. | `web/session.rs`, order handlers/state |
| State management | Customer/admin cookies could drift in policy. | High | Repeated cookie construction. | Separate names/maps, shared HttpOnly/SameSite/max-age policy. | session and handlers |
| Data persistence | Runtime contact/order-detail and learning files were tracked. | Critical | Repository data contained session/customer detail. | Removed from Git, ignored, safe schema example retained. | `.gitignore`, `data/*` |
| Data persistence | Replacement writes could leave partial state after failure. | High | Store-specific direct rewrite. | Temp write, sync, backup, replace. | `persistence/*` |
| Data persistence | CSV accepted duplicate IDs, invalid timestamps, unknown references, and repeated basket IDs too permissively. | High | Loader behavior/tests. | Fail contextual corruption; deduplicate basket items; validate references. | `data_loader.rs`, `main.rs` |
| Recommendation calculations | Co-order/popularity/association history work repeated inside candidate scoring. | High | Candidate-loop call graph. | Request-scoped scoring context and precomputed metrics. | `hybrid.rs`, evidence, association, popularity |
| Recommendation calculations | Production and fixed experiment paths risk conceptual confusion. | Medium | Similar output models, different method intent. | Kept deliberately separate and documented/tested. | `hybrid.rs`, `web/state.rs`, docs |
| API consistency | Logout and partial updates used inconsistent methods. | Medium | Router inspection. | Logout POST; status/availability PATCH; dish deletion DELETE. | routes, handlers, JS/templates |
| Security | The requested FYP demo login is predictable. | High | Admin credential resolution. | Explicitly documented as a local demo default; environment overrides are required outside controlled demonstrations. | admin handler, README |
| Security | Admin/customer order access needed stronger separation. | High | Route/handler ownership checks. | Protected admin APIs and customer order ownership scope. | handlers/state/session |
| Performance | The small dataset masked avoidable O(candidates x orders) scans. | Medium | Recommender call graph. | Build reusable indexes once per request; no benchmark framework added. | recommender modules |
| Accessibility | Dish modal could accumulate listeners and not restore trigger focus. | Medium | Modal binding/close flow. | Duplicate-safe binding, labelled dialog, focus return, live checkout status. | templates, `app.js` |
| Accessibility | Some admin action controls visually touched. | Low | UI screenshots and CSS layout. | Shared wrapping/gap rules retained and cleaned. | `app.css` |
| Tests | Baseline covered algorithms but not new security/validation boundaries. | Medium | Initial 105-test suite. | Added random session, validation, atomic write, association equivalence, data corruption, experiment-explanation, and accessibility tests; 112 pass. | module tests |
| Documentation | README and guides duplicated obsolete desktop/debug behavior. | High | Document link/content audit. | Replaced with one indexed current documentation set. | README, `docs/*` |
| Repository quality | No CI or contribution templates. | Medium | `.github` absent. | Added strict Windows CI, issue templates, PR template, contribution guide. | `.github/*`, `CONTRIBUTING.md` |

## Dependency Audit

The direct dependency list remains small. `getrandom` is the only new runtime
dependency and provides operating-system randomness for session identifiers.
No frontend framework, database client, password library, benchmark framework,
LLM SDK, or machine-learning library was added.

## Residual Risks

- `WebState` still combines several service operations and uses repeated
  `RwLock::expect` calls for poisoned-lock invariants.
- Templates, JavaScript, and CSS are large single files.
- No typed global HTTP error enum has replaced every existing response pattern.
- CSV cannot provide database-grade concurrent transactions.
- CSRF protection, rate limiting, TLS, and persistent staff accounts remain
  deployment work.
- Dish edits, prices, availability, sessions, carts, and live orders are not all
  durable across restart.

These are documented rather than hidden; none was papered over with an
unsupported production claim.
