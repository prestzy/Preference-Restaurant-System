# Advanced Recommendation Features

This document describes four deterministic, explainable extensions to the
Preference-Driven Restaurant Ordering System. They use ordinary Rust
collections and the existing CSV history. No external model API or heavy
machine-learning library is used.

## Production Pipeline

Customer recommendation requests run in this order:

```text
hard disliked-ingredient and availability exclusions
-> adaptive base scoring and evidence calculation
-> diversity/discovery reranking
-> top recommendation display
-> optional budget-aware meal-set optimisation
```

The controlled Experiment Lab continues to use the original base scorer and
its fixed comparison weights. Diversity reranking does not leak into those
experiments.

## Base Score, Confidence, and Reranked Score

- **Base score** combines content preference, co-order, popularity, and
  time/context signals using request-level adaptive weights.
- **Evidence confidence** measures how much preference and historical evidence
  supports the result. It does not mean probability of satisfaction.
- **Reranked score** adjusts the base score for novelty, category
  representation, and similarity to results already selected for the list.

Hard exclusions happen before every score. A reranker or meal-set optimizer
cannot restore a dish containing a disliked ingredient.

## Diversity and Discovery

Dish similarity uses weighted Jaccard overlap:

```text
similarity =
0.50 * ingredient Jaccard
+ 0.30 * tag Jaccard
+ 0.20 * category match
```

Sequential reranking evaluates each remaining candidate against results
already chosen. Novelty is `1 - normalized popularity`; category bonus is `1`
when the category is not yet represented; similarity penalty is the maximum
similarity to an already chosen result.

Modes:

```text
Familiar = 0.85*base + 0.05*novelty + 0.05*category - 0.05*similarity
Balanced = 0.70*base + 0.10*novelty + 0.10*category - 0.10*similarity
Discover = 0.55*base + 0.20*novelty + 0.10*category - 0.15*similarity
```

A candidate must pass the larger of the absolute `0.10` base-score floor and
`45%` of the best base score to enter the reranked top pool. Below-floor dishes
remain in the complete result list after the qualified pool; they are not
silently deleted.

Reported list metrics are category diversity, intra-list similarity, average
novelty, and popularity concentration.

## Budget-Aware Meal Sets

The customer supplies an integer budget, party size, optional target dish
count, optional required categories, preferences, and selected cart dishes.
Web prices are converted to integer cents before comparison.

Prototype target heuristic:

```text
1 person -> 2 dishes
2 people -> 3 dishes
3-4 people -> 4 dishes
5-6 people -> 5 dishes
7-8 people -> 6 dishes
9-12 people -> 8 dishes
```

This is not a serving-size or nutrition estimate. The CSV dataset has no
portion metadata.

Hard constraints:

- available dishes only;
- no disliked ingredients;
- total price at or below the exact budget;
- selected/cart dishes remain included;
- required categories must be present;
- no duplicate dishes;
- target size and request bounds are enforced.

The optimizer uses deterministic beam search with a candidate pool near 20,
beam width 200, maximum 8 dishes, and maximum 5 returned sets. This bounds the
request cost for the current 30-dish catalogue.

Dish utility is:

```text
0.80 * reranked recommendation score + 0.20 * evidence confidence
```

Set objective:

```text
0.45 * average dish utility
+ 0.15 * preference coverage
+ 0.15 * category coverage
+ 0.10 * pair compatibility
+ 0.10 * set diversity
+ 0.05 * budget utilisation
```

Pair compatibility is the average normalized historical co-order count across
all unordered pairs. Set diversity is one minus average pairwise dish
similarity. Budget utilization is intentionally a small signal; the system
does not force a customer to spend the full budget.

## Recommendation Learning Timeline

When staff complete a real checkout:

1. the historical order is appended to `data/orders.csv`;
2. in-memory history is updated;
3. a learning event compares history before and after that durable order;
4. the event is appended to
   `data/recommendation_learning_events.jsonl`.

Each event records unique dish popularity deltas, unordered pair count and
association deltas, and meaningful co-order rank changes. The durable
historical order ID is the idempotency key.

Timeline failure never rolls back a successfully completed order. Admin receives
a warning and can rebuild the entire timeline by replaying historical orders in
timestamp/order-ID order. Rebuild writes the separate JSON Lines file only
after all events have been generated and serialized.

Timeline events contain no customer name, phone, table, session user ID, or
order note. They show that **evidence changed**; they do not prove that
recommendation quality or accuracy improved.

## Counterfactual Explorer

The admin can compare a baseline with temporary changes to:

- liked/disliked ingredients;
- preferred tags;
- selected dish context;
- diversity mode;
- bounded simulated co-orders.

The service validates vocabulary, dish IDs, add/remove conflicts, Top-K
(`1-20`), distinct pair IDs, and simulated count (`0-100` per pair). It clones
real order history, adds temporary baskets to that clone, and calls the exact
production pipeline for both scenarios.

Comparison uses full eligible lists before Top-K truncation and reports rank,
score, confidence, Top-K entry/exit, hard-exclusion, and adaptive-weight
deltas. Simulated data never changes `orders.csv`, live state, or the learning
timeline. CSV export is labeled as a temporary comparison.

## APIs

```text
POST /api/recommendations/meal-set
POST /api/admin/recommendations/counterfactual
GET  /api/admin/recommendations/learning-timeline
POST /api/admin/recommendations/learning-timeline/rebuild
```

The three admin endpoints require the existing admin session.

## Limitations

- Prices are whole-RM prototype values stored in web state, then converted to
  cents for set calculation.
- Serving size, nutrition, tax, service charge, and inventory quantity are not
  modeled.
- Similarity and objective weights are documented heuristics, not learned
  parameters.
- Diversity does not guarantee serendipity or satisfaction.
- Timeline events explain data changes, not causal model improvement.
- Counterfactual results describe this deterministic prototype and are not
  predictions of future customer behavior.
