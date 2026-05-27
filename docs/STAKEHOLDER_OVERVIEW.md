# Stakeholder Overview

## What the System Does

The Preference-Driven Restaurant Ordering System is a desktop prototype for helping customers discover suitable dishes from one restaurant menu.

The system considers:

- What ingredients the customer likes.
- What ingredients the customer dislikes.
- What tags or food styles the customer prefers.
- What dishes are often ordered together historically.

The output is a ranked list of recommended dishes with explanations.

Local dish thumbnails help customers and evaluators recognize menu items visually. Images are used only on menu cards and recommendation cards; they are not part of the scoring algorithm.

## Why It Is Explainable

The recommendation result is not a black box. Each recommended dish shows:

- Ingredient score.
- Co-order score.
- Final hybrid score.
- A short explanation in plain language.

This makes it suitable for an FYP presentation because evaluators can see how input preferences influence output recommendations.

## Main User Experience

The main page is **Explore & Recommend**.

Stakeholders can:

1. Browse the menu.
2. Search by dish, ingredient, tag, category, or ID.
3. Select dishes from menu cards.
4. Enter preferences.
5. Open Evaluation to view updated recommendations and score explanations.

The previous separated flow has been merged into one practical workflow so users do not need to jump between menu, preferences, and recommendation pages.

## Recommendation Approach

### Ingredient-Based Filtering

The system compares each dish against explicit user preferences.

- Liked ingredients increase the ingredient score.
- Preferred tags add a small score bonus.
- Disliked ingredients exclude the dish from recommendation ranking.

### Collaborative Filtering

The system reads historical orders and counts dish pairs that were ordered together.

If a user selects `D01`, the system looks for dishes frequently ordered with `D01`.

### Hybrid Scoring

When both preference and selected dish information are available:

```text
final_score = 0.4 * ingredient_score + 0.6 * co_order_score
```

If selected dishes are missing, the system relies more on ingredient preferences. If preferences are missing, it relies more on co-order history.

## Order Simulation

Order simulation is an admin/demo tool.

It exists so presenters can demonstrate how collaborative filtering changes when new ordering behaviour is introduced.

It is not shown as a normal customer step because real customers would place orders through a checkout flow, while this prototype focuses on recommendation behaviour.

## What Can Be Extended

Future improvements can add:

- More restaurant dishes.
- More historical order records.
- More locally stored dish images with recorded source licenses.
- More precise dietary tags.
- A checkout workflow.
- More evaluation metrics.

The current system is intentionally lightweight so the core recommendation logic remains easy to inspect and explain.
