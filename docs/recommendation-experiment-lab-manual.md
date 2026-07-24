# Recommendation Experiment Lab Manual

## Purpose of the Lab

The Experiment Lab demonstrates how different recommendation inputs affect dish rankings in a restaurant with limited historical order data.

It contains three experiments:

1. Ingredient Impact
2. Co-Order Impact
3. Method Comparison

Simulation data is temporary and does not modify the real `data/orders.csv`.

## Experiment 1: Ingredient Impact

### What it demonstrates

This experiment shows how liked and disliked ingredients influence recommendation rankings:

- Dishes with liked ingredients can move upward.
- Dishes containing disliked ingredients are excluded.
- Selecting too many ingredients can make dishes harder to distinguish.

### How to use it

1. Open **Recommendation Tester**.
2. Select the **Ingredient Impact** tab.
3. Choose a Top-K value, such as Top 5.
4. Select one or more liked ingredients.
5. Optionally select disliked ingredients.
6. Press **Run Ingredient Experiment**.
7. Review the before-and-after rankings.
8. Read the generated conclusion.
9. Press **Reset** to restore default inputs.
10. Press **Clear Result** to remove output without changing selected preferences.

### Preset buttons

**No Preferences** removes liked and disliked ingredients and provides a neutral comparison.

**Example Preferences** selects a small demonstration set for a quick stakeholder walkthrough.

**All Ingredients** selects every available liked ingredient. This shows that ingredient scoring becomes less discriminative when almost everything is preferred.

### How to interpret the results

Review:

- Before rank
- After rank
- Ingredient score
- Matched ingredients
- Exclusion reason
- Rank change

Example interpretations:

> Chicken Satay moved from rank 7 to rank 2 because chicken was selected as a liked ingredient.

> A dish was excluded because it contained peanuts, which were selected as disliked.

This experiment supports the research question about how content-based ingredient filtering affects recommendation relevance.

## Experiment 2: Co-Order Impact

### What it demonstrates

This experiment shows how historical dish combinations affect collaborative recommendations. It uses temporary simulated co-orders to demonstrate what happens when two dishes are ordered together more frequently.

### How to use it

1. Select the **Co-Order Impact** tab.
2. Choose an Anchor Dish.
3. Choose a different Candidate Dish.
4. Enter additional simulated co-orders:
   - `0` for the baseline.
   - `5` for a small increase.
   - `10` or more for a stronger test.
5. Choose Top-K.
6. Press **Run Co-Order Experiment**.
7. Review the before-and-after metrics.
8. Press **Reset** to discard the temporary configuration.

The Anchor Dish and Candidate Dish must be different.

Example:

```text
Anchor: Nasi Lemak
Candidate: Chicken Satay
Additional simulated co-orders: 10
```

### How to interpret the results

Review:

- Pair count
- Co-order score
- Support
- Confidence
- Lift
- Candidate rank

**Support** shows how often both dishes appear together across all orders.

**Confidence** shows how often the candidate appears when the anchor dish is ordered.

**Lift** shows whether the pairing is stronger than expected from the candidate's overall popularity. A lift greater than 1 suggests a positive association.

Example interpretation:

> After adding ten temporary co-orders, Chicken Satay moved from rank 4 to rank 1 for Nasi Lemak.

The generated orders exist only in memory and are never written to `data/orders.csv`.

This experiment supports the research question about how co-order history affects collaborative-filtering scores and rankings in a limited-data environment.

## Experiment 3: Method Comparison

### What it demonstrates

This experiment compares:

1. Ingredient-only recommendation
2. Co-order-only recommendation
3. Hybrid recommendation

It uses a hidden-dish test. One dish is removed from an existing historical order, and the three methods attempt to recommend it again.

### How to use it

1. Select the **Method Comparison** tab.
2. Choose a historical order containing at least two dishes.
3. Select one dish from that order as the Hidden Target.
4. Optionally choose liked and disliked ingredients.
5. Select Top-K, preferably Top 3.
6. Press **Run Method Comparison**.
7. Compare the three result tables and summary.

### Controlled experiment settings

```text
Ingredient-only:
Ingredient = 1.0
Co-order = 0.0

Co-order-only:
Ingredient = 0.0
Co-order = 1.0

Hybrid:
Ingredient = 0.4
Co-order = 0.6
```

These controlled settings do not replace the production customer recommendation weights.

### How to interpret the results

**Hit@K** shows whether the hidden target appears in the Top-K recommendations. `Hit@3 = Yes` means the dish appeared in the Top 3.

**Hidden dish rank** is the target's exact recommendation position. A smaller rank is better.

**Preference match rate** shows how many Top-K recommendations match at least one liked ingredient.

**Restriction violations** counts recommendations containing a disliked ingredient. The expected value is zero because disliked ingredients are hard exclusions.

Example conclusion:

> The hybrid method recovered the hidden dish at rank 1. Ingredient-only recovered it at rank 3. Co-order-only did not recover it in the Top 3.

Only make this conclusion when the actual results support it.

This experiment supports the research question about whether combining ingredient and co-order signals improves recommendation quality.

## Suggested Stakeholder Demonstration

### Part 1: Ingredient influence

1. Run No Preferences and note the baseline.
2. Select a liked ingredient and run again.
3. Show the rank changes.
4. Add a disliked ingredient.
5. Show the exclusion.

### Part 2: Co-order influence

1. Select an anchor/candidate pair.
2. Run with zero additional orders.
3. Run with ten additional temporary orders.
4. Compare rank and association metrics.
5. Confirm the real CSV was not changed.

### Part 3: Method comparison

1. Select a historical order.
2. Hide one dish.
3. Run all three methods.
4. Compare Hit@K and hidden-dish rank.
5. Explain whether Hybrid produced a better balance in that test.

## Common Errors

### Anchor and candidate are the same

Choose two different dishes.

### Historical order has too few dishes

Choose an order containing at least two dishes.

### No hidden target is available

Select a valid historical order first.

### No result appears

Check that required fields are selected, then read the specific error shown in the experiment panel.

### Simulation changed the real CSV

This must never happen. Treat it as a defect and stop the test.

## Presentation Reminder

Do not describe these experiments as proving commercial accuracy. Describe them as controlled prototype experiments demonstrating:

- Ingredient influence
- Co-order influence
- Behaviour under limited data
- Differences between recommendation methods
