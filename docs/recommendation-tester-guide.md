# Recommendation Tester Guide

The Recommendation Tester is an authenticated admin area for explaining and
evaluating the FYP recommender. It is organized so stakeholders see one primary
workspace at a time.

## Category Overview

1. **Production Recommendation** inspects the live adaptive customer pipeline.
2. **Controlled Experiments** runs fixed research comparisons.
3. **Explainability and Simulation** tests temporary alternative scenarios.
4. **Learning History** shows evidence changes from real completed orders.

The selected category and tool are stored in the URL hash, for example:

```text
/admin/recommendations#experiments/ingredient-impact
```

This supports direct presentation links, refresh restoration, and normal
back/forward navigation.

## Production Tools

### Adaptive Scoring, Confidence, and Diversity

The Adaptive Scoring Inspector shows production weights, dataset/context/pair
strength, confidence, diversity evidence, score contributions, and ranked
dishes. Confidence measures evidence strength; it is not a probability that a
customer will like a dish.

### Budget Meal Set Tester

This sends a bounded budget, party size, optional target count, preferences,
context dishes, and diversity mode to the same meal-set service used by the
customer page. It does not write preferences or orders.

## Controlled Experiments

- **Ingredient Impact** compares a neutral ranking with explicit likes and
  dislikes.
- **Co-Order Impact** adds temporary pair baskets and compares collaborative
  evidence.
- **Method Comparison** evaluates ingredient-only, co-order-only, and fixed
  hybrid methods against a hidden historical basket item.

Controlled experiments do not silently inherit production adaptive weights.

## Explainability Tools

**What Would Change?** compares a production baseline with temporary preference,
context, diversity, or co-order changes. **Temporary Co-Order Simulation**
generates deterministic in-memory baskets. Neither operation updates
`data/orders.csv`, timeline events, customer preferences, or production state.

## Learning History

The timeline lists privacy-safe evidence changes keyed by historical order ID.
Filters include search, date, dish, sort order, and visible event limit.

### Reset Filters

Restores filter defaults and collapses event details. No event is deleted.

### Clear Timeline

`DELETE /api/admin/recommendations/learning-timeline` requires an authenticated
admin session and confirmation. It safely replaces only
`data/recommendation_learning_events.jsonl` with an empty timeline.

It does **not** change:

- `data/orders.csv`;
- historical or completed orders;
- popularity counts;
- co-order counts or association metrics;
- current production recommendations.

If clearing fails, existing rendered and in-memory events are retained.

### Rebuild Timeline

`POST /api/admin/recommendations/learning-timeline/rebuild` replays durable
historical orders chronologically and replaces timeline events only after the
new JSONL payload is valid. Historical orders are never appended or modified.

## Stakeholder Presentation Sequence

1. Open the overview and explain the four categories.
2. Use Production to show adaptive weights and evidence confidence.
3. Use Meal Sets to demonstrate a bounded customer decision.
4. Run the three Controlled Experiments for component comparison.
5. Use What Would Change? to explain one rank movement.
6. Open Learning History, filter one dish, and inspect an evidence event.
7. Demonstrate Reset Filters.
8. Explain, but only execute when appropriate, Clear and Rebuild Timeline.
