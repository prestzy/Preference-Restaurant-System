# Recommendation System

## Purpose and Scope

The recommendation engine is a deterministic decision-support component for a
small restaurant. It does not train a statistical model and does not estimate
the probability that a customer will like a dish. It combines understandable
signals, exposes their contributions, and falls back safely when order evidence
is sparse.

## Inputs

One production request can include:

- liked ingredients;
- disliked ingredients;
- preferred tags;
- currently selected/context dish IDs;
- optional time context;
- diversity mode; and
- Top-K output size.

Menu dishes contribute normalised ingredient, tag, and category features.
Historical orders contribute popularity and item-item co-order counts.

## Hard Eligibility Rules

Unavailable dishes, already selected dishes, and dishes containing any disliked
ingredient are removed before ranking. This is a hard rule:

```text
eligible(dish) =
    available
    AND dish not already selected
    AND intersection(dish.ingredients, disliked_ingredients) is empty
```

No popularity, co-order, or time score can reintroduce an excluded dish.

## Content Score

For an eligible dish:

```text
liked_match_ratio =
    matched_liked_ingredients / max(number_of_dish_ingredients, 1)

tag_bonus = 0.10 * number_of_matched_preferred_tags

content_score = clamp(liked_match_ratio + tag_bonus, 0, 1)
```

The ratio rewards specific overlap without treating a long ingredient list as
automatically better. The tag term is a small prototype bonus. These are
heuristics, not nutrition or taste predictions.

## Co-Order Score

Every historical order is treated as a basket. For each unordered pair in one
basket, the pair frequency is incremented once. Duplicate dish IDs in malformed
rows are removed by the loader so they cannot inflate evidence.

For selected context dishes `S` and candidate `c`:

```text
raw_co_order(c) = sum(pair_count(s, c) for s in S)
co_order_score(c) = raw_co_order(c) / maximum_candidate_raw_co_order
```

The denominator is calculated once per request. A score of `1.0` means the
candidate has the strongest observed co-order signal among that request's
eligible candidates, not universal certainty.

## Popularity Score

```text
popularity_count(c) = number of baskets containing c
popularity_score(c) = popularity_count(c) / maximum_dish_popularity
```

Popularity is the main fallback when explicit preferences and useful selected
context are absent. It is also retained at a smaller weight in other profiles.

## Time/Business Context

The time score is a small rule-based signal derived from the configured or local
meal period and dish metadata. It supports sensible fallback ordering but is not
a learned demand forecast.

## Association Metrics

For anchor `A`, candidate `B`, and `N` baskets:

```text
support(A,B) = pair_count(A,B) / N
confidence(A -> B) = pair_count(A,B) / popularity_count(A)
lift(A -> B) =
    confidence(A -> B) / (popularity_count(B) / N)
```

- **Support** is how often the pair appears in all baskets.
- **Association confidence** is how often B appears when A appears.
- **Lift** compares the pair to chance based on B's popularity.

Association confidence is not the evidence meter and not a probability of
preference. A rare pair can have high lift after one occurrence, so pair count
and dataset coverage must also be considered.

The implementation derives these metrics from the request-scoped co-order and
popularity indexes rather than rescanning orders per candidate.

## Request Evidence Profile

The adaptive model uses four saturation targets:

| Evidence | Prototype target |
|---|---:|
| Total historical baskets | 50 |
| Baskets containing selected context | 10 |
| Strongest observed context pair count | 5 |
| Candidate popularity count | 10 |

For an observed count `x` and target `t`:

```text
strength(x,t) = clamp(x / t, 0, 1)
```

Collaborative confidence is zero without selected context or without an observed
pair. Otherwise:

```text
collaborative_confidence =
    0.20 * dataset_strength
  + 0.35 * context_strength
  + 0.45 * pair_strength
```

This confidence controls how much the production scorer trusts co-ordering. The
targets are documented research heuristics, not fitted parameters.

## Adaptive Weights

Let `q` be collaborative confidence.

### Preferences and selected context

```text
content    = 0.70 - 0.30q
co_order   = 0.05 + 0.35q
popularity = 0.15 - 0.05q
time       = 0.10
```

### Preferences only

```text
content=0.70, co_order=0.00, popularity=0.20, time=0.10
```

### Selected context only

```text
content    = 0.00
co_order   = 0.10 + 0.55q
popularity = 0.80 - 0.55q
time       = 0.10
```

### No preferences and no selected context

```text
content=0.00, co_order=0.00, popularity=0.85, time=0.15
```

All values are cleaned, clamped, and normalised to sum to one.

## Hybrid Score

```text
base_score =
    w_content * content_score
  + w_co_order * co_order_score
  + w_popularity * popularity_score
  + w_time * time_score
```

Scores are finite and clamped. Final sorting uses explicit tie-breakers for
repeatability.

## Score Versus Evidence Confidence

Ranking score answers: "Which eligible dish should appear higher under the
current rules?"

Evidence confidence answers: "How much support exists for the signals behind
this recommendation?"

Candidate evidence combines weighted content, collaborative, popularity, and
time evidence. Its bands are:

| Normalised evidence | Label |
|---|---|
| `< 0.15` | Insufficient |
| `0.15` to `< 0.40` | Low |
| `0.40` to `< 0.70` | Medium |
| `>= 0.70` | High |

A new dish can have a strong content score and rank well while retaining low
historical evidence. This is expected and transparent.

## Diversity and Discovery

Candidate similarity is:

```text
similarity =
    0.50 * ingredient_jaccard
  + 0.30 * tag_jaccard
  + 0.20 * category_match
```

The reranker compares base relevance, novelty (`1 - max_similarity`), category
coverage, and popularity according to:

- **Familiar**: emphasises original relevance and familiar demand.
- **Balanced**: trades a small amount of redundancy for variety.
- **Discover**: gives novelty more influence.

A relevance floor prevents a weak but unusual dish from displacing all suitable
options. Eligibility restrictions remain unchanged in every mode.

## Budget-Aware Meal Sets

The bounded set search keeps only combinations that:

- remain within budget;
- include selected dishes;
- satisfy hard disliked-ingredient restrictions;
- meet requested categories where possible;
- avoid duplicate dish IDs; and
- stay within the prototype's candidate/search limits.

Set score:

```text
set_score =
    0.45 * average_dish_utility
  + 0.15 * preference_coverage
  + 0.15 * category_coverage
  + 0.10 * pair_compatibility
  + 0.10 * set_diversity
  + 0.05 * budget_utilisation
```

Party size influences bounded heuristics; it is not a serving-size or nutrition
guarantee. Menu prices are prototype values and are not currently persisted in
`dishes.csv`.

## Counterfactual Comparison

"What Would Change?" runs the exact production pipeline twice:

1. baseline preferences/context/history;
2. one temporary changed scenario.

It reports rank movement, entered/left Top-K dishes, exclusions, and adaptive
weight deltas. Temporary co-orders are applied only to cloned in-memory orders.
No counterfactual request writes `orders.csv` or the learning timeline.

## Controlled Experiments

Production adaptive scoring and academic method comparison are intentionally
separate.

- **Ingredient Impact** compares neutral and preference-shaped rankings.
- **Co-Order Impact** adds temporary baskets and compares pair metrics/rank.
- **Method Comparison** hides one dish from a known basket and compares
  ingredient-only, co-order-only, and fixed controlled hybrid output.

Method Comparison reports Hit@K and hidden-target rank for one selected case.
One case cannot establish global model superiority.

## Learning Timeline

Only real orders moved to `Completed` create durable historical evidence and a
derived explanatory event. Events show order-history size, popularity deltas,
pair deltas, and possible rank changes. Rebuild is chronological and
deterministic. Clearing the timeline does not clear order history.

## Request-Scoped Optimisation

`RecommendationScoringContext` calculates reusable information once:

- co-order matrix;
- popularity counts;
- candidate context-basket counts;
- normalisation maxima;
- time context;
- request evidence profile;
- adaptive weights; and
- selected-dish set.

The previous pattern of rescanning all historical baskets for each candidate was
removed. This changes execution cost, not ranking semantics. An equivalence test
protects association metrics computed from prebuilt indexes.

## Determinism and Numerical Safety

- Duplicate basket items are deduplicated on load.
- Scores and weights are finite and clamped to `[0,1]`.
- Zero denominators return zero evidence rather than NaN.
- Weight normalisation has a defined popularity/time fallback.
- Floating-point tests use tolerances.
- Sorting includes stable dish-ID/name tie-breakers.
- Simulations use deterministic seeded generation where applicable.

## Limitations

- Heuristics are not calibrated probabilities.
- Order baskets do not represent explicit negative feedback.
- The dataset is too small for broad claims about accuracy.
- Ingredient matching depends on CSV vocabulary and normalisation.
- Co-order association does not prove causation.
- Popularity can reinforce exposure; diversity modes mitigate but do not remove
  that feedback loop.
- No serving size, nutrition, allergy certification, stock, or margin model is
  included.

