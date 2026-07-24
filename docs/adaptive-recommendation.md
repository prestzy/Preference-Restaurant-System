# Data-Aware Adaptive Recommendation

## 1. Research Motivation

A single SME restaurant normally has much less behavioural data than a delivery
platform. A fixed hybrid formula can over-trust co-ordering when only a few
baskets exist, or under-use it after repeated relationships become available.
This prototype therefore adjusts production weights from observable evidence.
It remains deterministic and uses no external ML or LLM API.

## 2. Fixed-Weight Limitation

The previous production formula always combined content, co-order, popularity,
and context using fixed percentages. That formula could not distinguish a new
pair observed once from a mature pair observed repeatedly. Production Hybrid
mode now uses adaptive weights. The Experiment Lab deliberately retains fixed
weights for controlled comparison.

## 3. Evidence Inputs And Thresholds

The central `AdaptiveScoringConfig` defaults are heuristic saturation targets:

| Target | Default | Meaning |
|---|---:|---|
| Total historical orders | 50 | Dataset evidence is treated as mature |
| Context orders | 10 | Baskets containing any selected dish |
| Pair co-orders | 5 | Pair evidence reaches full strength |
| Candidate appearances | 10 | Popularity evidence reaches full strength |

Counts are basket-based. Duplicate dish IDs in one basket are counted once.
With multiple selected dishes, each historical basket qualifies at most once.

## 4. Collaborative Confidence

```text
dataset_strength = min(total_orders / 50, 1)
context_strength = min(context_orders / 10, 1)
pair_strength = min(strongest_pair_count / 5, 1)

collaborative_confidence =
    0.20 * dataset_strength
  + 0.35 * context_strength
  + 0.45 * pair_strength
```

Collaborative confidence is forced to zero when there is no selected dish
context or when no selected-to-candidate pair has been observed. Lift remains
visible but does not increase evidence maturity because rare pairs can have
unstable lift.

## 5. Adaptive Weight Formulas

Let `c` be collaborative confidence.

### A. Preferences And Selected Context

```text
content    = 0.70 - 0.30c
co-order   = 0.05 + 0.35c
popularity = 0.15 - 0.05c
time       = 0.10
```

### B. Preferences Without Selected Context

```text
content 0.70, co-order 0.00, popularity 0.20, time 0.10
```

### C. Selected Context Without Preferences

```text
content    = 0.00
co-order   = 0.10 + 0.55c
popularity = 0.80 - 0.55c
time       = 0.10
```

### D. No Preferences And No Context

```text
content 0.00, co-order 0.00, popularity 0.85, time 0.15
```

All weights are validated, normalized, finite, clamped to `0..1`, and sum to
one. UI percentages use adjusted rounding so they total exactly 100%.

## 6. Candidate Evidence Calculation

Positive preference evidence counts liked ingredients and preferred tags.
Disliked ingredients are not positive evidence; they remain hard exclusions.

```text
input_strength =
  0.00 for no positive preference
  0.75 for one preference
  min(0.75 + 0.125 * (count - 1), 1.00) otherwise

content_evidence =
  input_strength * match_coverage * content_score

collaborative_evidence =
  collaborative_confidence
  * min(candidate_pair_count / 5, 1)
  * co_order_score

popularity_evidence =
  dataset_strength
  * min(candidate_appearances / 10, 1)
  * popularity_score
```

Time evidence equals the positive time score only when a specific context such
as Lunch is supplied. `Any` creates no time evidence.

## 7. Overall Confidence

```text
overall_confidence =
    adaptive_content_weight * content_evidence
  + adaptive_co_order_weight * collaborative_evidence
  + adaptive_popularity_weight * popularity_evidence
  + adaptive_time_weight * time_evidence
```

This is separate from the final recommendation score. A candidate can rank well
from metadata while showing limited historical evidence.

## 8. Confidence Bands

| Label | Range |
|---|---:|
| Insufficient / Limited | `< 0.15` |
| Low | `0.15 .. < 0.40` |
| Medium | `0.40 .. < 0.70` |
| High | `>= 0.70` |

**Interpretation warning:** This confidence score represents the strength of
available recommendation evidence. It is not a probability of customer
satisfaction.

## 9. Worked Low-Data Example

Assume 5 historical orders, one selected dish appearing in 2 baskets, and one
observed pair:

```text
dataset = 5/50 = 0.10
context = 2/10 = 0.20
pair = 1/5 = 0.20
collaborative confidence =
  0.20(0.10) + 0.35(0.20) + 0.45(0.20) = 0.18
```

With preferences and context, content remains dominant:

```text
content 64.6%, co-order 11.3%, popularity 14.1%, time 10.0%
```

## 10. Worked High-Data Example

Historical orders: 62. Selected context: Nasi Lemak. Context orders: 9.
Strongest pair count: 3.

```text
dataset_strength = min(62/50, 1) = 1.00
context_strength = min(9/10, 1) = 0.90
pair_strength = min(3/5, 1) = 0.60

collaborative_confidence =
  0.20(1.00) + 0.35(0.90) + 0.45(0.60)
  = 0.785
```

For preferences plus selected context:

```text
content = 0.70 - 0.30(0.785) = 0.4645
co-order = 0.05 + 0.35(0.785) = 0.32475
popularity = 0.15 - 0.05(0.785) = 0.11075
time = 0.10
```

Displayed percentages total 100%: Content 47%, Co-order 32%, Popularity 11%,
Time/context 10%.

## 11. New-Dish Example

A new chicken dish can rank from a strong chicken metadata match even with zero
popularity and pair counts. Its notes explain that it is preference-led and has
limited historical evidence. It is not hidden merely because it is new.

## 12. Limitations

- Thresholds are heuristic prototype settings.
- Confidence is not a calibrated probability.
- Association metrics can remain unstable in small datasets.
- Completed orders improve evidence but do not guarantee satisfaction.
- Content quality depends on accurate ingredients and tags.
- Co-ordering describes basket relationships, not causal preference.
- Popularity can reinforce already-popular dishes.
- The model has not been validated on a large commercial dataset.
- Stakeholder and customer testing are still required.

## 13. Controlled Experiment Isolation

Production customer recommendations use Data-Aware Adaptive Weights. Experiment
Lab methods remain fixed:

- Ingredient-only: `1.0 content / 0.0 co-order`
- Co-order-only: `0.0 content / 1.0 co-order`
- Controlled Hybrid: `0.4 content / 0.6 co-order`

Experiments use temporary/cloned data and never mutate adaptive configuration.

## 14. Stakeholder Demonstration

1. Open Admin > Recommendation Experiment Lab.
2. Use Adaptive Scoring Inspector with a rare selected dish.
3. Record context counts, collaborative confidence, and weights.
4. Repeat with a frequently ordered selected dish.
5. Show how co-order influence grows with evidence.
6. Open a result row to show score, confidence, counts, and notes.
7. Explain that confidence is evidence strength, not satisfaction probability.
8. Complete repeated real checkout pairs and rerun to demonstrate immediate
   evidence learning from persisted completed orders.
