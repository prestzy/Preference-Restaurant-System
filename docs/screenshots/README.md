# Screenshot Capture Checklist

This directory intentionally contains no fabricated product screenshots. Capture
screens from a running build after the associated flow has been verified.

## Required Screens

1. Customer start/registration at 390x844.
2. Home with search suggestions and full static Menu count.
3. Recommended for You rail and evidence explanation.
4. Personalisation chips and selected dishes.
5. Meal-set result under a visible budget.
6. Cart and order tracking.
7. Admin dashboard.
8. Admin orders with status controls.
9. Dish management with image preview.
10. Adaptive Scoring result.
11. Ingredient and Co-Order Impact results.
12. Method Comparison and What Would Change?.
13. Learning Timeline.

## Capture Rules

- Use synthetic customer details only.
- Hide browser bookmarks, unrelated tabs, filesystem paths, and credentials.
- Do not show cookies, session IDs, private logs, or real phone numbers.
- Use the current commit and record its SHA in the PR/notes.
- Do not crop away warnings or empty states that materially affect the claim.
- Name files descriptively, for example
  `customer-search-static-menu-390x844.png`.
- Add useful alt text when embedding a screenshot.
- State when sample data or a temporary simulation is shown.

## README Use

Only add screenshots to the repository landing page after:

1. the flow passes the manual checklist;
2. the image is privacy-safe;
3. the image accurately reflects the current UI; and
4. its relative link works on GitHub.

