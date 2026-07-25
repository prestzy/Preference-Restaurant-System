# Stakeholder Presentation Guide

## Presentation Narrative

Use this order:

1. Restaurant problem.
2. Why popularity alone is not enough.
3. Why limited history is difficult.
4. How several simple evidence sources are combined.
5. How decisions and uncertainty are explained.
6. How controlled experiments demonstrate behavior.
7. What the prototype can and cannot prove.

## Two-Minute Elevator Script

"Most food-ordering systems either show the same popular dishes to everyone or
need a large amount of customer history. This project is designed for a smaller
restaurant that may have limited data and cannot place a dedicated tablet at
every table.

A customer scans a QR code, opens a mobile menu, selects ingredients and tags
they prefer or dislike, adds dishes to a cart, and tracks the order. The Menu
always stays complete. Search only helps the customer locate a dish.

The recommender combines four simple sources. Content matching uses ingredients
the customer explicitly selects. Co-ordering uses dish combinations observed in
historical baskets. Popularity provides a safe fallback when little else is
known. A small time rule accounts for meal context.

The important part is adaptive weighting. When order evidence is weak, the
system trusts explicit preferences and popularity more. When the same context
and dish pairs appear repeatedly, co-ordering receives more influence. Disliked
ingredients remain hard exclusions.

The interface separates recommendation score from evidence confidence. A dish
can rank highly because it matches chicken while still showing low historical
evidence. This avoids presenting uncertainty as certainty.

The admin Recommendation Tester demonstrates ingredient impact, temporary
co-order impact, hidden-target method comparison, diversity, meal sets,
counterfactual changes, and how completed orders alter evidence. Simulations do
not modify real history.

This is a deterministic, explainable FYP prototype. It demonstrates practical
recommendation behavior for limited restaurant data; it does not claim that a
score is the probability a customer will like a dish."

## Ten-Minute Stakeholder Demonstration

### 0:00-1:00 - Problem and Objective

**Click:** Customer start page, then Home.

**Say:** "A customer uses their own phone after scanning a QR code. The design
avoids dedicated table hardware and targets one small restaurant."

**Point at:** Compact registration, search near the top, full Menu, bottom
navigation.

**Do not claim:** That the prototype is already an internet-hardened commercial
deployment.

### 1:00-2:00 - Customer Ordering Flow

**Click:** A search suggestion, one dish detail, Add, Cart.

**Say:** "Search locates a card; it never removes dishes from the Menu. The cart
uses the same local menu and availability state."

**Point at:** Smooth locator highlight, all-dish count, quantity and total.

**Do not claim:** Real payment, stock, nutrition, or kitchen integration.

### 2:00-3:30 - Adaptive Recommendation

**Click:** Personalise Recommendations; like chicken; select a context dish.
Then open Admin Recommendation Tester > Adaptive Scoring.

**Say:** "The scorer uses preferences, co-orders, popularity, and time. It
increases collaborative influence only when the available pair evidence
supports it."

**Point at:** Four weights totaling 100%, evidence counts, recommendation reason.

**Do not claim:** That weights were learned from a neural model.

### 3:30-5:00 - Ingredient Impact

**Click:** Experiments > Ingredient Impact; run neutral, like chicken, then
dislike peanuts.

**Say:** "This controlled before/after test isolates the content signal. Liked
ingredients can move dishes; disliked ingredients remove conflicts."

**Point at:** Rank movement and excluded dishes.

**Do not claim:** That every chicken-preferring customer will select rank 1.

### 5:00-6:30 - Co-Order Impact

**Click:** Co-Order Impact; D01 anchor, D09 candidate; compare zero and ten
temporary co-orders.

**Say:** "The experiment adds baskets only to a temporary clone. Pair count,
support, confidence, lift, and rank can change without altering restaurant
history."

**Point at:** Before/after pair count plus candidate rank.

**Do not claim:** Simulated demand is observed customer behavior.

### 6:30-8:00 - Method Comparison

**Click:** Method Comparison; select a multi-dish order, hide one target, Top-3.

**Say:** "We hide one known dish and test whether ingredient-only, co-order-only,
or fixed hybrid ranking recovers it."

**Point at:** Hit@3 and hidden-target rank for each method.

**Do not claim:** One case proves a method is globally superior.

### 8:00-9:00 - Counterfactual and Learning Timeline

**Click:** What Would Change? add a seafood restriction. Then open Timeline.

**Say:** "Counterfactual analysis changes a temporary calculation. The timeline
is different: it records how real completed orders changed popularity and pair
evidence."

**Point at:** Entered/left Top-K, exclusions, pair deltas.

**Do not claim:** Clearing timeline removes historical orders; it does not.

### 9:00-10:00 - Findings and Limitations

**Click:** Dashboard or remain in tester overview.

**Say:** "The contribution is transparent adaptation under limited data and a
safe evaluation environment. It is deterministic and lightweight. Limits
include the small fixture, rule-based thresholds, local CSV persistence, and no
formal customer-outcome study."

**Point at:** Historical count and explicit tool separation.

**Do not claim:** Production-scale concurrency, measured satisfaction gain, or
calibrated accuracy.

## Fifteen-Minute Academic Presentation

### 1. Background - 1 minute

QR menus reduce table hardware requirements. Recommendation can improve menu
discovery, but SME restaurants have sparse orders and limited engineering
capacity.

### 2. Problem Statement - 1 minute

Popularity is impersonal; pure collaborative filtering is unreliable with sparse
context; opaque models are difficult to defend. The research problem is
explainable recommendation that degrades safely under limited evidence.

### 3. Research Questions - 1 minute

1. Can explicit preferences and basket evidence be combined adaptively?
2. Can evidence strength be communicated separately from ranking?
3. Can controlled experiments demonstrate component effects without corrupting
   real history?
4. Can diversity and meal-set support be added while preserving restrictions?

### 4. System Architecture - 1.5 minutes

Show the architecture Mermaid diagram. Explain Axum handlers, state services,
focused recommender modules, CSV/JSONL persistence, and plain browser assets.
Emphasise local-only images and no external inference service.

### 5. Recommendation Methodology - 2 minutes

Explain eligibility, content ratio/tag bonus, item-item pair frequency,
normalised popularity, association metrics, and weighted sum. State that these
are transparent heuristics.

### 6. Production Adaptive Model - 2 minutes

Show evidence targets and four profile cases. Explain why collaborative
confidence is zero without a selected context/observed pair. Show score versus
evidence confidence.

### 7. Controlled Experiments - 2.5 minutes

Demonstrate Ingredient Impact, Co-Order Impact, and Method Comparison. Explain
Top-K, hidden target, Hit@K, and fixed experimental methods. State the
non-mutation boundary.

### 8. Results Interpretation - 1.5 minutes

Present case-level observations: rank movement after one ingredient, pair
metrics after temporary baskets, and hidden-target recovery. Use "in this case"
language and report restriction violations.

### 9. Limitations - 1 minute

Small synthetic/public sample, heuristic thresholds, CSV concurrency limits,
prototype prices, no long-term profiles, no calibrated probability, and no
customer satisfaction experiment.

### 10. Future Work - 0.5 minute

Database transactions, security hardening, consent-aware profiles, formal
offline evaluation, usability study, and multi-site work only after validating
the single-restaurant model.

## Beginner Analogies

| Concept | Analogy | Technical connection |
|---|---|---|
| Content | "A waiter uses ingredients you say you like." | Ingredient/tag overlap score |
| Co-ordering | "A waiter notices D01 buyers often also order D09." | Basket pair frequency |
| Popularity | "With no other clue, start from commonly ordered dishes." | Normalised basket count |
| Adaptive hybrid | "Trust each kind of advice according to available evidence." | Request-level weights |
| Confidence | "How strong are the records behind the advice?" | Candidate evidence, not probability |
| Lift | "Does the pair happen more than expected from general popularity?" | Association ratio |
| Counterfactual | "Recalculate after changing one assumption on a whiteboard." | Two temporary pipeline runs |
| Timeline | "An audit note showing what one completed basket changed." | Derived learning event |

## Stakeholder Questions and Honest Answers

### Is this artificial intelligence?

It is an intelligent decision-support system in the broad sense, built from
explicit recommendation algorithms. It does not use generative AI.

### Is this machine learning?

It learns behavioral evidence by updating counts, but it does not train a
statistical parameter model. "Algorithmic recommendation" is the most precise
description.

### Why not use ChatGPT?

The task needs deterministic menu matching, hard restrictions, local data
privacy, low cost, and inspectable formulas. A language model would add
uncertainty and external dependency without solving those needs better.

### Why not deep learning?

The sample data is far too small, and the project values explanation and
maintainability. Deep learning would increase cost and overfitting risk.

### How does the system know what I like?

Only from ingredients/tags selected in the current session and dishes selected
as context. It does not infer a permanent identity profile.

### What happens for a new customer?

Explicit preferences work immediately. Without preferences, popularity and time
provide fallback recommendations.

### What happens for a new dish?

It can rank through metadata matches even with low historical evidence. The UI
should show that evidence limitation.

### What if there is no order history?

Content preferences drive relevant results; otherwise deterministic menu
fallback behavior remains. Co-order confidence is zero.

### Is confidence an accuracy percentage?

No. It is normalised evidence strength behind that recommendation, not the
probability of a correct prediction.

### Why can lift be high with one order?

Lift compares against expected frequency. Rare items can produce a large ratio
from little data, so pair count and support must accompany lift.

### Does simulation change real restaurant data?

No. Simulation and counterfactual tools operate on cloned in-memory data and
have non-mutation tests.

### Does every completed order improve accuracy?

No. It adds evidence. Accuracy improvement requires evaluation across many
representative outcomes.

### Why use CSV instead of a database?

CSV keeps the single-restaurant FYP transparent and easy to inspect. It is not
suitable for concurrent production scale; a database is future work.

### How is customer privacy protected?

Customer/admin sessions are separate, cookies are HttpOnly/SameSite, order
lookups are ownership-scoped, runtime contact details are Git-ignored, and the
app does not call external AI services. This is not a complete regulatory
security solution.

### Why is the hybrid method useful?

Different evidence helps in different situations. Content handles explicit
preferences and new items; co-ordering captures basket behavior; popularity
supports sparse cases.

### Can this scale to multiple restaurants?

Not as implemented. Data and state are single-restaurant. Multi-tenancy needs
database isolation, authentication roles, and operational design.

### What are the limitations?

Limited data, rule-based thresholds, prototype prices, process-local sessions
and live orders, no payment/stock system, and no formal outcome trial.

## Final Presentation Checklist

- Use synthetic names and orders.
- Verify admin credentials before the session.
- Keep Top-K fixed in comparisons.
- Say "evidence" rather than "accuracy probability".
- Say "association" rather than "customers caused".
- Demonstrate simulation isolation.
- State limitations before questions.

