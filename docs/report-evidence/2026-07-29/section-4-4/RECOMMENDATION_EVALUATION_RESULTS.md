# 4.4 Recommendation Evaluation Results

This section presents the results obtained from the three controlled
recommendation experiments described in Section 3.6. Unlike Section 4.3, which
established whether the system functions correctly, this section examines how
ingredient preferences, co-order evidence, and recommendation methods affected
the generated rankings.

The experiments were executed on 29 July 2026 using the same 30-dish menu and
60-order historical baseline. Temporary co-order baskets were held in memory
and were not added to `data/orders.csv`. The experiment interface also reported
that production weights and operational data were unchanged.

## Experiment Protocol

The following controls were applied so the observations can be reproduced:

- Ingredient Impact used the content-based method. IP-01 and IP-02 used Top-10
  so the selected baseline dishes could be tracked; IP-03 used Top-5 in the
  evidence screenshot.
- Co-Order Impact used the co-order-only score and tested each pair at 0, 3,
  and 5 added temporary baskets.
- Method Comparison used the five most recent eligible historical orders
  (`O060` to `O056`).
- The last dish in each selected basket was hidden. The remaining dishes became
  the order context.
- The same fixed preferences, liked `rice` and `chicken`, were applied to all
  five method-comparison cases. No disliked ingredient was applied.
- Hit@3 was recorded when the hidden dish rank was 3 or better.
- Average hidden-dish rank used the exact rank in the complete eligible ranking,
  including ranks outside the Top-3.
- Controlled method weights were ingredient-only `1.0/0.0`, co-order-only
  `0.0/1.0`, and fixed hybrid `0.4/0.6`.

The supplied draft reused Table 4.7, which had already been assigned to the
responsive results in Section 4.3. The tables below therefore continue from
Table 4.8 to keep report numbering unique.

## 4.4.1 Ingredient Preference Impact Results

The Ingredient Preference Impact experiment addressed RQ1 by comparing a
neutral ranking with a ranking generated after liked and disliked ingredients
were applied.

**Table 4.8: Ingredient Preference Impact Results**

| Scenario | Preference input | Selected dish | Baseline rank | Preference rank | Rank change | Outcome |
|---|---|---|---:|---:|---:|---|
| IP-01 | Liked: bean sprouts | Char Kway Teow (D14) | 10 | 1 | +9 | Moved upward |
| IP-02 | Disliked: banana | Pisang Goreng (D30) | 1 | Excluded | N/A | Hard-excluded |
| IP-03 | Liked: rice; disliked: banana | Ketupat (D05) | 2 | 1 | +1 | Moved upward; D30 excluded |

In IP-01, Char Kway Teow moved from rank 10 to rank 1 after `bean sprouts`
was selected as a liked ingredient. This was an improvement of nine ranking
positions. The result's ingredient score was 0.14.

In IP-02, Pisang Goreng was removed from the eligible output after `banana`
was marked as disliked. It was not merely moved downward from its baseline rank
of 1.

The combined IP-03 scenario applied `rice` as liked and `banana` as disliked.
Ketupat moved from rank 2 to rank 1 with an ingredient score of 0.50, while
Pisang Goreng remained hard-excluded. Across the three runs, the selected
dishes produced two upward movements. There were two exclusion events because
Pisang Goreng was independently excluded in IP-02 and IP-03. No disliked
ingredient appeared in the resulting eligible Top-K lists, giving zero
restriction violations.

**Figure 4.8: Ingredient Preference Impact Experiment Result**

[Open Figure 4.8](figure-4-8-ingredient-impact.png)

Figure 4.8 displays the combined scenario's selected preferences, baseline and
changed ranks, matched ingredient, hard exclusion, and result summary. These
observations answer RQ1 by showing that a liked ingredient can raise matching
dishes while a disliked ingredient acts as an eligibility restriction.

Raw evidence: [ingredient-impact-results.csv](ingredient-impact-results.csv)

## 4.4.2 Co-Order History Impact Results

The Co-Order History Impact experiment addressed RQ2 by adding exact temporary
baskets containing an anchor and candidate dish. The baseline was recalculated
independently for every run.

**Table 4.9: Co-Order History Impact Results**

| Pair | Added co-orders | Pair count | Support | Confidence | Lift | Candidate rank |
|---|---:|---:|---:|---:|---:|---:|
| Nasi Lemak (D01) -> Sambal Sotong (D07) | 0 | 1 | 0.02 | 0.12 | 1.87 | 7 |
| Nasi Lemak (D01) -> Sambal Sotong (D07) | 3 | 4 | 0.06 | 0.36 | 3.27 | 1 |
| Nasi Lemak (D01) -> Sambal Sotong (D07) | 5 | 6 | 0.09 | 0.46 | 3.33 | 1 |
| Nasi Lemak (D01) -> Chicken Satay (D09) | 0 | 3 | 0.05 | 0.38 | 4.50 | 1 |
| Nasi Lemak (D01) -> Chicken Satay (D09) | 3 | 6 | 0.10 | 0.55 | 4.30 | 1 |
| Nasi Lemak (D01) -> Chicken Satay (D09) | 5 | 8 | 0.12 | 0.62 | 4.00 | 1 |

For Pair A, the pair count increased from 1 to 6 after five temporary
co-orders. Support increased from 0.02 to 0.09, confidence increased from 0.12
to 0.46, and lift increased from 1.87 to 3.33. The candidate moved from rank 7
to rank 1 after three additions and remained at rank 1 after five additions.

Pair B showed a ceiling effect. Its pair count increased from 3 to 8, support
from 0.05 to 0.12, and confidence from 0.38 to 0.62, but Chicken Satay already
occupied rank 1 at baseline and therefore could not move higher. Its lift
changed from 4.50 to 4.00 as both the pair frequency and marginal dish
frequencies changed in the enlarged temporary dataset.

**Figure 4.9: Effect of Simulated Co-Orders on Candidate Rank**

[Open Figure 4.9](figure-4-9-coorder-rank.png)

The graph places rank 1 at the top because a lower rank value represents a
stronger recommendation position.

**Figure 4.10: Co-Order History Impact Experiment Interface**

[Open Figure 4.10](figure-4-10-coorder-impact-interface.png)

Figure 4.10 shows Pair A after five temporary additions, including the
before-and-after pair count, co-order score, support, confidence, lift, and rank.
The experiment answers RQ2 by demonstrating that additional pair evidence
changed association measurements and could change candidate position. It also
showed that stronger measurements do not necessarily change rank when the
candidate is already first.

Raw evidence: [coorder-impact-results.csv](coorder-impact-results.csv)

## 4.4.3 Recommendation Method Comparison Results

The Recommendation Method Comparison addressed RQ3 by hiding one dish from
each selected historical basket and testing whether ingredient-only,
co-order-only, and fixed-hybrid ranking recovered it.

**Table 4.10: Hidden-Dish Recovery by Recommendation Method**

| Case | Historical order | Hidden dish | Ingredient rank | Co-order rank | Hybrid rank | Ingredient Hit@3 | Co-order Hit@3 | Hybrid Hit@3 |
|---|---|---|---:|---:|---:|---:|---:|---:|
| MC-01 | O060 | Kuih Seri Muka (D06) | 4 | 1 | 1 | 0 | 1 | 1 |
| MC-02 | O059 | Gado-Gado (D29) | 18 | 2 | 3 | 0 | 1 | 1 |
| MC-03 | O058 | Pisang Goreng (D30) | 23 | 1 | 1 | 0 | 1 | 1 |
| MC-04 | O057 | Ketupat (D05) | 1 | 2 | 1 | 1 | 1 | 1 |
| MC-05 | O056 | Rojak Buah (D28) | 25 | 3 | 3 | 0 | 1 | 1 |

The ingredient-only method recovered the hidden dish within the Top-3 in one
of five cases. Co-order-only and fixed hybrid each recovered the hidden dish in
all five cases.

**Table 4.11: Overall Method Comparison**

| Recommendation method | Successful Hit@3 cases | Hit@3 rate | Average hidden-dish rank |
|---|---:|---:|---:|
| Ingredient-only | 1/5 | 20% | 14.20 |
| Co-order-only | 5/5 | 100% | 1.80 |
| Fixed hybrid 0.4/0.6 | 5/5 | 100% | 1.80 |

Co-order-only and fixed hybrid jointly produced the highest Hit@3 result,
recovering all five hidden dishes. Ingredient-only produced the lowest result,
with one successful recovery.

The methods did not produce identical ranks in every case. For Gado-Gado in
MC-02, co-order-only placed the hidden dish at rank 2 and hybrid placed it at
rank 3, while ingredient-only placed it at rank 18. Conversely, Ketupat in
MC-04 was ranked first by ingredient-only and hybrid but second by co-order-only.
These are direct ranking observations; their broader interpretation belongs in
Section 4.5.

**Figure 4.11: Hit@3 Rate by Recommendation Method**

[Open Figure 4.11](figure-4-11-method-hit-at-3.png)

**Figure 4.12: Recommendation Method Comparison Interface**

[Open Figure 4.12](figure-4-12-method-comparison-interface.png)

Figure 4.12 shows MC-02. The interface records that ingredient-only did not
recover Gado-Gado within Top-3, whereas co-order-only recovered it at rank 2 and
fixed hybrid recovered it at rank 3. All methods reported zero restriction
violations.

Raw evidence:

- [method-comparison-results.csv](method-comparison-results.csv)
- [method-comparison-summary.csv](method-comparison-summary.csv)

## 4.4.4 Summary of Recommendation Results

Ingredient Preference Impact showed that liked ingredients changed the
relative position of compatible dishes and disliked ingredients enforced hard
exclusions. Co-Order Impact showed that added pair evidence increased pair
count, support, and confidence; this moved Sambal Sotong from rank 7 to rank 1,
while Chicken Satay remained first because it was already the strongest
co-order candidate. Method Comparison produced Hit@3 rates of 20% for
ingredient-only and 100% for both co-order-only and fixed hybrid in the five
selected cases.

Together, the controlled observations demonstrate that explicit preferences
and historical basket evidence affect recommendation rankings in different
ways. The results remain specific to the selected 30-dish, 60-order dataset and
five hidden-dish cases.

The method comparison is a controlled recovery demonstration, not an
independent train/test evaluation. It hides the target from the request context
but retains the loaded historical baseline. A future evaluation should use more
baskets and remove each test basket from the training history before calculating
its recommendations.

## Evidence Reproduction

1. Start the application with `cargo run`.
2. Open `/admin/recommendations` and log in.
3. Open `Experiments`.
4. Re-run the inputs recorded in Tables 4.8-4.10.
5. Confirm that every result states that production data and weights were not
   changed.
6. Confirm that `git diff -- data/orders.csv` is empty.
7. Regenerate the two graphs with:

   ```powershell
   python docs/report-evidence/2026-07-29/section-4-4/generate_charts.py
   ```

Do not add additional graphs unless they communicate a result that cannot be
read clearly from the tables.
