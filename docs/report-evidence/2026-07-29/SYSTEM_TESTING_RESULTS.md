# System Testing Results

This document records the system tests executed for Section 4.3 of the
Preference-Driven Restaurant Ordering System report. The results were obtained
on 29 July 2026 from repository revision `207ccc9`.

## Test Environment

- Platform: Windows local development environment
- Rust compiler: `rustc 1.92.0`
- Cargo: `cargo 1.92.0`
- Web server: local loopback server
- Browser automation: Chromium through Playwright
- Dataset: 30 dishes and 60 initial historical orders
- Responsive viewports:
  - Mobile: 390 x 844 pixels
  - Tablet: 768 x 1024 pixels
  - Desktop: 1440 x 900 pixels

The browser tests used a disposable copy of the CSV files. This prevented the
test order from changing the repository's working dataset while still testing
the real CSV persistence and restart behaviour.

## Important Test-Case Correction

The draft security table described ST-04 as starting the application without
administrator credentials. That case no longer matches the implemented
prototype because the project intentionally provides the demonstration
credentials `admin` / `admin`.

ST-04 was therefore replaced with a relevant session-security test:

> Administrator logs out and attempts to reuse the previous session.

The expected result is that the old session can no longer access protected
administrator pages.

## 4.3.1 Overall Testing Summary

The automated quality gate consisted of:

```text
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

All four commands completed successfully. The Rust test suite reported **115
passed, 0 failed, and 0 ignored**.

The 115 Rust tests exercise implementation details across the codebase. Table
4.2 instead counts the selected report-level test scenarios from Tables
4.3-4.7. This avoids incorrectly presenting every Rust test as a separate
end-to-end or responsive test.

**Table 4.2: Overall System Testing Results**

| Test category | Number of test cases | Passed | Failed | Pass rate |
|---|---:|---:|---:|---:|
| Unit and component testing | 12 | 12 | 0 | 100% |
| Integration testing | 8 | 8 | 0 | 100% |
| End-to-end workflow testing | 3 | 3 | 0 | 100% |
| Security and access-control testing | 8 | 8 | 0 | 100% |
| Responsive-interface inspection | 24 | 24 | 0 | 100% |
| **Total** | **55** | **55** | **0** | **100%** |

The responsive category contains eight interface areas inspected at three
viewports, giving 24 checks. The pass rate was calculated as:

```text
Pass rate = (number of passed tests / total number of tests) x 100
```

For Figure 4.5, run `cargo test` and capture the command plus the final
`115 passed; 0 failed` summary from the terminal. A terminal screenshot is not
included here because it should be captured directly from the report author's
own execution environment.

## 4.3.2 Unit and Component Test Results

**Table 4.3: Unit and Component Test Results**

| Test ID | Component tested | Expected result | Actual result | Status |
|---|---|---|---|---|
| UT-01 | Missing dish CSV columns | Invalid dataset is rejected | The loader returned a validation error identifying missing required columns. | Pass |
| UT-02 | Duplicate dish identifiers | Duplicate records are rejected | Dataset validation detected duplicate dish IDs and rejected the invalid dataset. | Pass |
| UT-03 | Invalid historical timestamp | Invalid record is rejected | The order loader reported the invalid timestamp instead of adding the record to valid history. | Pass |
| UT-04 | Unknown dish reference | Historical record is rejected | The loader reported the unknown dish ID and excluded the invalid basket from accepted history. | Pass |
| UT-05 | Search processing | Relevant dishes are located from valid search terms | Searching `laksa` returned Laksa, Curry Laksa, and Asam Laksa as suggestions while the static Menu remained at 30 dishes. | Pass |
| UT-06 | Conflicting preferences | Ingredient cannot remain both liked and disliked | Selecting `chicken` as disliked cleared its liked state; the final states were liked=`false` and disliked=`true`. | Pass |
| UT-07 | Unavailable-dish restriction | Unavailable dish is excluded | State tests confirmed that unavailable dishes were removed from customer-facing availability and eligible menu search results. | Pass |
| UT-08 | Disliked-ingredient restriction | Dish containing disliked ingredient is excluded | Counterfactual and ingredient-restriction tests excluded dishes containing the disliked term without modifying order history. | Pass |
| UT-09 | Adaptive weighting | Component weights sum to 1.0 | All tested evidence situations produced normalised content, co-order, popularity, and time-context weights with a total of 1.0. | Pass |
| UT-10 | Deterministic ranking | Repeated identical input produces the same ranking | Deterministic reranking and meal-set tests produced the same ordered result for identical inputs. | Pass |
| UT-11 | Diversity reranking | Only eligible dishes remain after reranking | All diversity modes preserved the eligible candidate set; reranking changed order only and introduced no excluded dish. | Pass |
| UT-12 | Meal-set constraints | Generated set remains within budget and restrictions | Generated sets respected the selected budget, category constraints, uniqueness, and active exclusions. | Pass |

The principal automated evidence is provided by the following Rust tests:

- `dish_csv_reports_missing_required_columns`
- `duplicate_ids_and_invalid_timestamps_are_rejected`
- `duplicate_basket_items_are_deduplicated_and_unknown_dishes_are_reported`
- `all_adaptive_situations_sum_to_one`
- `all_modes_are_deterministic_and_preserve_candidates`
- `disliked_change_excludes_without_mutating_orders`
- `valid_sets_respect_budget_category_uniqueness_and_determinism`
- `dish_availability_removes_dish_from_customer_menu`
- `menu_search_excludes_unavailable_dishes`

All 12 selected unit and component scenarios passed. The results show that
malformed records were stopped at the data boundary, hard restrictions were
preserved during ranking, adaptive weights remained valid, and identical
inputs produced repeatable outputs.

## 4.3.3 Integration Test Results

A synthetic customer named `FYP Test Customer` was registered at table `T08`.
The customer selected Nasi Lemak (`D01`) and Chicken Satay (`D09`) and submitted
a basket worth RM18.00. The test used synthetic contact information only.

**Table 4.4: Integration Test Results**

| Test ID | Integration path | Expected result | Actual result | Status |
|---|---|---|---|---|
| IT-01 | Customer checkout to live order | Valid checkout creates a new live order | Checkout created live order `WEB001` with status Pending and dishes `D01,D09`. | Pass |
| IT-02 | Live order to administrator board | New order becomes visible to the administrator | `WEB001` appeared immediately in Admin Orders with customer, table, dish, time, total, and status data. | Pass |
| IT-03 | Administrator update to customer tracking | Customer view reflects the updated status | After the administrator selected Preparing, the customer Profile displayed `Preparing` for `WEB001`. | Pass |
| IT-04 | Completed order to historical basket | Completed order is added to historical evidence | Completion appended `O061,U21,"D01,D09","2026-07-29 16:48"` and increased valid historical rows from 60 to 61. | Pass |
| IT-05 | Repeated completion request | Historical basket is not duplicated | Repeating Completed returned an already-saved response; the CSV remained at 61 historical rows. | Pass |
| IT-06 | Dish availability to customer menu | Disabled dish is unavailable for ordering | Automated state tests removed an unavailable dish from the customer menu and search results. | Pass |
| IT-07 | Dish availability to recommender | Disabled dish is removed from candidates | Recommendation generation used the availability-filtered dish set, preventing unavailable items from entering customer results. | Pass |
| IT-08 | Controlled experiment to operational history | Temporary experiment does not alter real history | Experiment-isolation tests left the operational order history and learning timeline unchanged. | Pass |

After `WEB001` was completed, the local server was stopped and restarted using
the same disposable data directory. Historical order `O061` was still present,
confirming that the newly completed basket was loaded from CSV after restart.
Consequently, the basket becomes available to popularity, co-order,
association, learning-timeline, and hybrid recommendation calculations.

Use the following evidence as Figure 4.6:

- [(a) Customer order submitted](figure-4-6a-customer-order-submitted.png)
- [(b) Order visible on the administrator board](figure-4-6b-admin-live-order.png)
- [(c) Updated status shown to the customer](figure-4-6c-customer-status-updated.png)

Figure 4.6 demonstrates that customer ordering, administrator processing, and
customer tracking operated on the same live order. The persistence check also
confirmed that completion transferred the basket into historical evidence only
once.

## 4.3.4 End-to-End Workflow Results

**Table 4.5: End-to-End Workflow Results**

| Scenario | Starting condition | Final expected outcome | Actual outcome | Status |
|---|---|---|---|---|
| Customer workflow | No active customer session | Customer registers, selects dishes, checks out, and views the submitted order | The customer registered, selected `D01` and `D09`, checked out for RM18.00, and viewed Pending order `WEB001` on the Profile page. | Pass |
| Administrator workflow | Administrator not authenticated | Administrator logs in, views the order, updates its status, and completes it | Login with the prototype credentials succeeded; `WEB001` was changed through Preparing, Ready, and Completed and was persisted as `O061`. | Pass |
| Connected workflow | Valid customer and administrator sessions | Customer order passes through the complete operational lifecycle | The status changed in both interfaces, completion updated historical CSV data once, and the historical record survived a server restart. | Pass |

These results were produced in a local single-server environment. They
demonstrate functional workflow integration but do not represent simultaneous
restaurant-scale traffic or deployment over a public network.

## 4.3.5 Security and Access-Control Results

**Table 4.6: Security and Access-Control Test Results**

| Test ID | Test condition | Expected behaviour | Actual behaviour | Status |
|---|---|---|---|---|
| ST-01 | Administrator page accessed without login | Request is redirected or rejected | `GET /admin` returned HTTP 302 and redirected to `/admin/login`. | Pass |
| ST-02 | Invalid administrator credentials | Login is rejected | The server returned HTTP 401 and displayed `Invalid username or password.` | Pass |
| ST-03 | Customer cookie used for administrator API | Request is rejected | `/api/admin/orders` returned `ok=false` with `Admin login required.` | Pass |
| ST-04 | Logged-out administrator reuses previous session | Protected access is rejected | Logout returned HTTP 303; reusing the old cookie redirected `/admin` to `/admin/login`. | Pass |
| ST-05 | Customer requests another session's order | Access is denied | The API returned `ok=false` with `Order was not found in this customer session.` | Pass |
| ST-06 | Unknown dish identifier submitted | Request is rejected safely | Checkout returned `ok=false` with `No valid available menu item was submitted.` | Pass |
| ST-07 | Invalid order-status value submitted | Status change is rejected | The administrator API returned `ok=false` with `Unknown order status.` | Pass |
| ST-08 | Unauthenticated access to experiment tools | Access is rejected | The protected experiment endpoint returned `ok=false` with `Admin login required.` | Pass |

The tests confirmed separation between customer and administrator sessions and
rejection of invalid protected operations. They do not constitute a production
security certification. HTTPS termination, hashed staff credentials, CSRF
protection, rate limiting, and multiple role-based staff accounts remain
outside the current prototype scope.

## 4.3.6 Responsive-Interface Inspection Results

Each interface area was inspected at 390 x 844, 768 x 1024, and 1440 x 900
pixels. Browser measurements confirmed that none of the inspected pages created
full-page horizontal overflow. Horizontal recommendation rails and dense admin
tables retained local scrolling where appropriate.

**Table 4.7: Responsive-Interface Inspection Results**

| Interface area | Mobile | Tablet | Desktop | Main observation |
|---|---|---|---|---|
| Customer navigation | Pass | Pass | Pass | Bottom navigation remained visible and its destinations remained accessible. |
| Menu cards | Pass | Pass | Pass | All 30 dishes remained available; cards adapted from compact mobile presentation to wider desktop columns. |
| Preference controls | Pass | Pass | Pass | The collapsible preference panel remained within the viewport and option groups wrapped without page overflow. |
| Recommendation cards | Pass | Pass | Pass | The horizontal rail retained readable card widths and local horizontal scrolling. |
| Cart and checkout | Pass | Pass | Pass | Item details, quantities, total, and checkout action remained visible without page overflow. |
| Order tracking | Pass | Pass | Pass | Profile details, filters, status badge, dishes, and progress controls remained readable. |
| Administrator order board | Pass | Pass | Pass | Dense order data stayed within a local table wrapper instead of widening the whole page. |
| Dish-management interface | Pass | Pass | Pass | Search, Add Dish, records, and actions remained reachable; the table used local scrolling on narrow screens. |

Use these images for Figure 4.7:

- [(a) Mobile customer interface, 390 x 844](figure-4-7-mobile-390x844.png)
- [(b) Tablet customer interface, 768 x 1024](figure-4-7-tablet-768x1024.png)
- [(c) Desktop customer interface, 1440 x 900](figure-4-7-desktop-1440x900.png)

Additional supporting evidence:

- [Mobile administrator order board](responsive-admin-orders-mobile-390x844.png)

The visual evidence supports responsive compatibility, not user satisfaction.
A separate participant usability study would be required to claim that
customers found the interface easy to use.

## 4.3.7 Testing Findings

Overall, **55 of 55 selected system-test checks passed**, producing an overall
pass rate of **100%**. In addition, the underlying automated Rust suite passed
all **115 tests**. The results confirmed that the main customer and
administrator workflows operated correctly, protected operations required the
appropriate session, and completed orders entered historical evidence without
duplication. Recommendation restrictions, adaptive-weight normalisation,
deterministic behaviour, experiment isolation, and responsive presentation also
operated according to the tested requirements.

The outcome should not be interpreted as evidence that the prototype is
production-ready. The testing was conducted by the developer on a local
single-server environment with synthetic customer data. It did not include
concurrent load testing, public-network deployment, advanced penetration
testing, or a formal study with external restaurant users.

System testing establishes whether the artefact functions correctly.
Recommendation-method effectiveness, ranking quality, and comparisons between
content-based, co-order, and hybrid methods should remain in Section 4.4.

## Reproduction Checklist

1. Open a terminal in the repository.
2. Run:

   ```powershell
   cargo fmt --check
   cargo check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   ```

3. Capture the final `cargo test` summary for Figure 4.5.
4. Start the web application with `cargo run`.
5. Register a synthetic customer, select at least two dishes, and check out.
6. Log in as the prototype administrator and record the live order.
7. Change its status and verify the same status on the customer Profile page.
8. Mark the order Completed and record the new row in `data/orders.csv`.
9. Repeat the Completed request and confirm that no duplicate row appears.
10. Restart the application and confirm that the historical row remains.
11. Inspect the customer and administrator pages at the three stated viewport
    sizes and record any overflow or inaccessible control.

Do not use real customer names or phone numbers in report screenshots.
