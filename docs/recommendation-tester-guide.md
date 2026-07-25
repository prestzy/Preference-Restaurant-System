# Recommendation Tester Guide

## Audience and Purpose

This is the beginner-facing manual for **Admin > Recommendation Tester**. It
explains what to click, what each result means, and what the experiment can and
cannot prove. No Rust knowledge is required.

All examples use synthetic preferences and the repository's public sample
dishes. Never enter private customer data during a demonstration.

## Chapter 1: Reading a Ranked Result

A **score** is a number used to order eligible dishes under one set of rules. A
higher score means the dish better satisfies those rules; it is not a customer
satisfaction percentage.

A **rank** is the dish's position after sorting: rank 1 is first. **Top-K** means
the first `K` results. Top-5 therefore means ranks 1 through 5.

Some tools show:

- **base rank**: before diversity;
- **reranked rank**: after diversity;
- **entered Top-K**: absent before, present after;
- **left Top-K**: present before, absent after; and
- **rank movement**: a smaller rank number is movement upward.

Identical input and data should produce identical output because the engine uses
explicit tie-breakers.

## Chapter 2: Recommendation Methods

### Content-based

Content-based recommendation compares liked ingredients and preferred tags with
dish metadata. Think of a waiter using the ingredients you explicitly mention.
Disliked ingredients are hard exclusions.

### Co-ordering

Co-ordering counts dishes appearing in the same historical basket. Think of a
waiter noticing that customers who order Nasi Lemak also often order Chicken
Satay. It finds associations, not causes.

### Popularity and time

Popularity is the fallback when little personal or pair evidence exists. Time is
a small rule-based meal-period signal.

### Hybrid and adaptive hybrid

Hybrid scoring combines multiple component scores. The production hybrid is
**adaptive**: it gives co-ordering more influence only when selected context,
dataset coverage, and repeated pair evidence support it. Controlled Method
Comparison uses fixed definitions so research comparisons remain repeatable.

## Chapter 3: Evidence and Association Language

**Recommendation score** controls ranking. **Evidence confidence** describes how
well the available data supports the signals. It is not the probability that a
customer will like the dish.

Evidence labels:

- Insufficient: below 0.15;
- Low: 0.15 to below 0.40;
- Medium: 0.40 to below 0.70;
- High: 0.70 and above.

For a pair A and B:

- **pair count**: baskets containing both;
- **support**: pair count divided by all baskets;
- **association confidence A -> B**: pair count divided by baskets containing A;
- **lift**: observed confidence divided by B's normal frequency.

One rare pair can have high lift, so always read lift with pair count and
support.

## Chapter 4: Evaluation Vocabulary

**Hidden target** means removing one known dish from an historical order,
treating the remaining dishes as context, then checking whether a method
recovers the removed dish.

**Hit@K** is true when the hidden target appears in the first K results. It is one
case-level retrieval result, not global accuracy.

**Preference match rate** is the share of shown results matching selected
preferences. **Restriction violations** should remain zero for disliked
ingredients.

**Counterfactual** means "what would the calculation show if this temporary
input changed?" It does not predict actual customer action.

---

## Tool 1: Adaptive Scoring Inspector

**1. What it is.** The live production scorer showing request evidence,
content/co-order/popularity/time weights, and ranked candidates.

**2. Why it exists.** Sparse restaurant data should not receive the same
collaborative weight as repeated, relevant basket evidence.

**3. Research question.** Does the recommender change method importance in a
deterministic direction when useful evidence changes?

**4. Data used.** Current available dishes, all historical baskets, selected
preferences, selected context dishes, and optional time context.

**5. Inputs.** Liked/disliked ingredients, tags, context dishes, Top-K, and
diversity mode.

**6. Exact steps.**

1. Open `Admin > Recommendation Tester`.
2. Choose `Production > Adaptive Scoring`.
3. Select liked ingredient `chicken`.
4. Run with no context dish and record weights.
5. Select `Nasi Lemak (D01)` as context.
6. Run again and inspect dataset, context, and pair evidence.
7. Change Top-K only; confirm request weights do not change.

**7. Outputs.** Historical basket count, context basket count, strongest pair
count, collaborative confidence, four weights, component scores, final score,
and candidate evidence.

**8. Worked example.** With chicken liked and no context, content receives 70%,
co-order 0%, popularity 20%, and time 10%. Adding D01 enables co-ordering. Its
weight grows only to the degree supported by observed D01 pairs.

**9. Beginner interpretation.** The system changes which source of waiter advice
it trusts according to how much relevant evidence exists.

**10. Common mistakes.** Do not compare weights after also changing several
preferences. Do not treat a 40% co-order weight as 40% customer probability.

**11. Safe conclusion.** "The system changed method importance according to
available evidence."

**12. Unsafe conclusion.** "The system proved that the customer will like the
first dish."

**13. Data safety.** Inspection reads production history but does not append,
delete, or simulate baskets.

**14. Demo tip.** Put the no-context and D01-context weight cards side by side in
your notes and point to the co-order increase.

---

## Tool 2: Confidence and Evidence Meter

**1. What it is.** A candidate-level view of evidence strength separate from the
ranking score.

**2. Why it exists.** A dish can rank first for a clear ingredient match even
when historical support is sparse.

**3. Research question.** Can the interface reveal weak evidence instead of
overstating a high rank?

**4. Data used.** Matched preferences, candidate popularity, pair counts,
request weights, and time-context support.

**5. Inputs.** The same production preference/context form used by Adaptive
Scoring.

**6. Exact steps.**

1. Choose `Production > Confidence Meter`.
2. Select `chicken` and run without a context dish.
3. Find a matching dish with limited order appearances.
4. Compare its final score with its evidence percentage and label.
5. Add a context dish with repeated pair evidence and run again.

**7. Outputs.** Final score, evidence percentage, Insufficient/Low/Medium/High
band, and primary evidence source.

**8. Worked example.** Chicken Satay may rank first because chicken matches, but
show Low evidence if its historical basket coverage is small. Rank and evidence
are both correct because they answer different questions.

**9. Beginner interpretation.** Score is the recommendation order; confidence is
the strength of the supporting records.

**10. Common mistakes.** Do not call evidence "accuracy" or read 80% as an 80%
chance of liking. Do not compare bands from different datasets as calibrated
probabilities.

**11. Safe conclusion.** "This recommendation is strongly or weakly supported by
the available evidence."

**12. Unsafe conclusion.** "There is an 80% chance this customer will like it."

**13. Data safety.** The meter is read-only and contains no private customer
history.

**14. Demo tip.** Deliberately show a high-rank, low-evidence result; it
demonstrates honest uncertainty better than showing only High labels.

---

## Tool 3: Diversity and Discovery

**1. What it is.** Three reranking modes: Familiar, Balanced, and Discover.

**2. Why it exists.** Relevance-only lists can repeat similar categories and
reinforce popular-item exposure.

**3. Research question.** Can variety increase while restrictions and a minimum
relevance standard remain intact?

**4. Data used.** Base relevance, ingredient/tag/category similarity,
popularity, and current Top-K category coverage.

**5. Inputs.** Preferences, context dishes, Top-K, and diversity mode.

**6. Exact steps.**

1. Choose `Production > Diversity`.
2. Run Familiar with Top-5 and record dishes/categories.
3. Run Balanced without changing any other field.
4. Run Discover.
5. Compare base rank, reranked rank, novelty, and category count.
6. Add disliked `peanuts`; verify D01 and D09 are excluded in all modes.

**7. Outputs.** Base/reranked ranks, novelty score, maximum similarity, category
bonus, diversity notes, and category variety.

**8. Worked example.** A less-popular but relevant dessert can move upward in
Discover because it adds category variety, while a near-duplicate main dish
moves down. A candidate below the relevance floor does not replace suitable
items solely for novelty.

**9. Beginner interpretation.** Familiar stays close to the safest known list;
Discover opens space for a relevant but less repetitive choice.

**10. Common mistakes.** Changing preferences between modes invalidates the
comparison. More categories does not automatically mean a better list.

**11. Safe conclusion.** "Discover increased variety while retaining a minimum
relevance requirement."

**12. Unsafe conclusion.** "Discover mode is always better."

**13. Data safety.** Reranking is request-only and never changes order history.

**14. Demo tip.** Read out the category sequence for each Top-5; stakeholders
understand variety before they understand similarity scores.

---

## Tool 4: Budget-Aware Meal Set Tester

**1. What it is.** A bounded search for groups of dishes, rather than one dish,
under a budget and hard restrictions.

**2. Why it exists.** Group diners need a compatible set with variety and budget
coverage, not five independent first-ranked dishes.

**3. Research question.** Can the prototype construct explainable sets satisfying
configured constraints?

**4. Data used.** Current available dishes and prototype prices, production dish
utility, content coverage, categories, pair compatibility, and similarity.

**5. Inputs.** Budget, party size, required categories, selected dishes,
preferences, diversity mode, and number of sets.

**6. Exact steps.**

1. Choose `Production > Meal Sets`.
2. Enter budget `80` and party size `4`.
3. Require `main`, `side`, and `dessert`.
4. Like `chicken`; dislike `peanuts`.
5. Choose Balanced and generate.
6. Check total price, categories, restrictions, and score breakdown.
7. Lower the budget until the UI reports that no valid set exists.

**7. Outputs.** Dish group, total, budget remaining/utilisation, category and
preference coverage, compatibility, diversity, and final set score.

**8. Worked example.** An illustrative RM80 result can combine available
repository dishes from requested categories while excluding Nasi Lemak and
Chicken Satay because both contain peanuts/peanut sauce. The exact set depends on
current prototype prices and availability.

**9. Beginner interpretation.** The system evaluates possible groups and keeps
combinations balancing suitability, variety, compatibility, and budget.

**10. Common mistakes.** Do not describe party-size heuristics as serving-size
guidance. Admin price edits are in memory and sample prices are not a commercial
menu source.

**11. Safe conclusion.** "The suggested set satisfies configured budget and
restrictions under the prototype scoring rules."

**12. Unsafe conclusion.** "This is the exact amount of food needed for four
people."

**13. Data safety.** Generation reads current state and writes no order. Only
customer checkout creates a live order.

**14. Demo tip.** First show a valid set, then deliberately use an impossible
budget to demonstrate constraint handling rather than silent rule-breaking.

---

## Tool 5: Ingredient Impact

**1. What it is.** A controlled before/after comparison of neutral ranking
against liked, disliked, and tag preferences.

**2. Why it exists.** It isolates content-based effects for academic evaluation.

**3. Research question.** Do explicit ingredients move compatible dishes and
hard-exclude conflicts as designed?

**4. Data used.** Dish ingredients/tags and a fixed experimental ranking setup.
It does not alter production adaptive weights.

**5. Inputs.** Liked ingredients, disliked ingredients, preferred tags, and
Top-K.

**6. Exact steps.**

1. Choose `Experiments > Ingredient Impact`.
2. Run No Preferences and record Top-K.
3. Add liked `chicken` and run.
4. Observe matched dishes moving upward.
5. Add disliked `peanuts` and run.
6. Confirm Nasi Lemak (peanuts) and Chicken Satay (peanut sauce) are excluded.
7. Reset after the demonstration.

**7. Outputs.** Before/after ranks and scores, rank movement, matches, exclusions,
and summary metrics.

**8. Worked example.** Liking chicken raises dishes containing chicken relative
to the neutral baseline. Disliking peanuts removes D01. Whether `peanut sauce`
matches a selected term depends on the normalised menu option chosen; use the
actual selectable vocabulary rather than free text.

**9. Beginner interpretation.** Liked features affect ranking; disliked features
act as a gate.

**10. Common mistakes.** Selecting almost all ingredients makes the content
signal less discriminating. Do not change Top-K between baseline and changed
run.

**11. Safe conclusion.** "Selected ingredients changed content-based ranking and
exclusions."

**12. Unsafe conclusion.** "All customers with this preference will choose the
top result."

**13. Data safety.** This is a calculated comparison with no order or timeline
write.

**14. Demo tip.** Use exactly one liked ingredient first; the causal path in the
prototype is easier to explain.

---

## Tool 6: Co-Order Impact

**1. What it is.** A before/after experiment that adds temporary baskets
containing one anchor and one candidate.

**2. Why it exists.** It demonstrates how repeated co-order evidence affects
pair metrics and collaborative rank.

**3. Research question.** Does stronger temporary pair evidence change the
collaborative component in the expected direction?

**4. Data used.** A clone of historical orders plus deterministic simulated
baskets. Real history remains unchanged.

**5. Inputs.** Different anchor and candidate dishes, added co-order count, and
Top-K.

**6. Exact steps.**

1. Choose `Experiments > Co-Order Impact`.
2. Select Nasi Lemak (D01) as anchor.
3. Select Chicken Satay (D09) as candidate.
4. Run with `0`; record pair count, support, confidence, lift, and rank.
5. Run with `10`; compare the same values.
6. Reset and open Dashboard to confirm historical count is unchanged.

**7. Outputs.** Before/after pair count, support, association confidence, lift,
candidate score/rank, and Top-K entry state.

**8. Worked example.** Ten temporary D01+D09 baskets increase pair count and
usually strengthen D09's collaborative rank for D01 context. The exact rank
depends on competing pairs in the fixture.

**9. Beginner interpretation.** Repeating a combination gives the system more
reason to associate those two dishes.

**10. Common mistakes.** Anchor and candidate must differ. High lift with one
rare pair is weak evidence; do not quote lift alone.

**11. Safe conclusion.** "Adding repeated temporary co-orders strengthened the
calculated relationship."

**12. Unsafe conclusion.** "Ten simulated orders prove real customers will buy
the pair."

**13. Data safety.** The experiment uses cloned in-memory baskets and is tested
not to mutate orders or learning events.

**14. Demo tip.** Show zero and ten in sequence, then point to both pair count and
rank rather than only the final score.

---

## Tool 7: Method Comparison

**1. What it is.** A leave-one-item-out case study comparing ingredient-only,
co-order-only, and controlled hybrid methods.

**2. Why it exists.** The same known target can be tested under three explicit
method definitions.

**3. Research question.** Which method recovers one hidden target best for the
selected historical case?

**4. Data used.** One historical basket, menu metadata, and historical baskets.
The hidden target is removed from the context before calculation.

**5. Inputs.** Historical order, hidden dish, optional preferences, and Top-K.

**6. Exact steps.**

1. Choose `Experiments > Method Comparison`.
2. Select an order containing at least two dishes.
3. Select one of its dishes as hidden target.
4. Choose Top-3.
5. Run with no preference, then optionally repeat with a relevant liked
   ingredient as a separately labelled case.
6. Compare Hit@3 and hidden-target rank across methods.

**7. Outputs.** Method, ranked rows, component/final scores, Hit@K, hidden-target
rank, preference match rate, and restriction violations.

**8. Worked example.** For a sample basket containing D01 and D09, hide D09 and
use D01 as context. If hybrid places D09 at rank 2 while co-order places it at 3
and content misses Top-3, hybrid has the better result **for this case only**.

**9. Beginner interpretation.** The test hides a known answer and checks whether
each method can find it from the remaining clue.

**10. Common mistakes.** Never leave the hidden target in selected context.
Top-3 and Top-5 are different tests. One basket is not a dataset-wide result.

**11. Safe conclusion.** "In this selected case, the hybrid method recovered the
hidden dish at a better rank."

**12. Unsafe conclusion.** "The hybrid method is always superior."

**13. Data safety.** Historical orders are read only; the hidden item is removed
from a temporary request.

**14. Demo tip.** Choose a basket with an understandable pair so stakeholders
can reason about why content and co-order methods differ.

---

## Tool 8: What Would Change?

**1. What it is.** A counterfactual comparison between the current production
pipeline and one temporary alternative scenario.

**2. Why it exists.** Stakeholders can inspect sensitivity without changing
customer data or real evidence.

**3. Research question.** Which ranks, exclusions, and adaptive weights change
when one controlled input changes?

**4. Data used.** Current production dishes/history plus cloned preferences,
context, and optional simulated pairs.

**5. Inputs.** Baseline preferences/context, additions/removals, temporary
co-orders, and Top-K.

**6. Exact steps.**

1. Choose `Explainability > What Would Change?`.
2. Create a baseline and run it.
3. Add `seafood` or an actual seafood ingredient as disliked.
4. Compare exclusions and Top-K movement.
5. Reset.
6. Add a selected dish and temporary co-orders.
7. Compare co-order score and adaptive-weight deltas.
8. Verify Dashboard historical count remains unchanged.

**7. Outputs.** Baseline/changed Top-K, entered/left dishes, rank deltas,
exclusions, component changes, and adaptive-weight changes.

**8. Worked example.** Adding a seafood restriction removes matching fish/prawn
dishes from the changed list. Eligible alternatives enter Top-K. Temporary pair
evidence may increase co-order weight without touching the baseline history.

**9. Beginner interpretation.** It is a calculator for "what if this input were
different?", not a forecast.

**10. Common mistakes.** Changing several fields makes one cause difficult to
attribute. Reset between the restriction and simulated-pair examples.

**11. Safe conclusion.** "This temporary scenario shows how the ranking
calculation would change."

**12. Unsafe conclusion.** "This predicts exactly what will happen in the
restaurant."

**13. Data safety.** Both scenarios are temporary. No order, customer profile, or
timeline entry is written.

**14. Demo tip.** Use one visible hard exclusion first; the before/after change is
immediate and easy to defend.

---

## Tool 9: How the Recommender Learned

**1. What it is.** A timeline explaining evidence changes caused by real
completed orders.

**2. Why it exists.** Historical learning should be inspectable rather than an
unexplained change in ranking.

**3. Research question.** Can the system trace a completed basket to popularity
and pair-frequency changes?

**4. Data used.** Completed `orders.csv` baskets and derived JSONL learning
events.

**5. Inputs.** Timeline filters plus protected Clear and Rebuild actions.

**6. Exact steps.**

1. Place a synthetic customer order.
2. In `Admin > Orders`, move it through status to Completed.
3. Open `Learning > Timeline`.
4. Locate the new event and inspect popularity/pair deltas.
5. Note any displayed rank change.
6. Use filter reset; confirm records remain.
7. For a controlled demo only, Clear Timeline and show orders still exist.
8. Rebuild Timeline and confirm events return from history.

**7. Outputs.** Historical order count, completed order reference, dish
popularity deltas, pair-count/association changes, and explanatory rank changes.

**8. Worked example.** Completing D01+D09 adds one appearance to each dish and one
D01-D09 pair occurrence. Subsequent recommendations can use the new evidence
immediately and after restart.

**9. Beginner interpretation.** The system can show which real basket changed
the evidence it uses.

**10. Common mistakes.** One completed order does not prove accuracy improved.
Clearing timeline is not the same as clearing order history.

**11. Safe conclusion.** "This completed order changed the recommendation
evidence."

**12. Unsafe conclusion.** "The system became more accurate after one order."

**13. Data safety.** Only real completion appends history. Clear removes derived
explanations only; Rebuild reads real history. Do not use private customer data
in screenshots.

**14. Demo tip.** Record the historical count before completion and show it
increase by exactly one, then point to the pair delta.

---

## Worked Example Summary

| Scenario | Expected interpretation |
|---|---|
| Like chicken | Compatible dishes can move upward through content score. |
| Dislike peanuts | Dishes containing the selected normalised ingredient are hard-excluded. |
| Add temporary D01+D09 baskets | Pair count and collaborative influence strengthen without persistence. |
| Hide a known basket dish | Hit@K reveals whether a method recovered it for that case. |
| New relevant dish | Content can rank it while historical confidence remains low. |
| Discover mode | A less-popular relevant dish can rise to improve variety. |
| RM80 set | Returned combinations must satisfy prototype budget/restriction rules. |
| Complete D01+D09 order | Popularity and pair evidence increase by one basket. |
| Add seafood restriction | Matching dishes leave the counterfactual Top-K. |

## Before a Demonstration

1. Start with known synthetic fixture data.
2. Confirm admin credentials and Recommendation Tester access.
3. Record the initial historical order count.
4. Use one input change per experiment.
5. Keep Top-K constant during before/after comparisons.
6. Do not show private order-detail files.
7. Say "demonstrates behavior" rather than "proves accuracy".
8. Reset temporary controls after each tool.
