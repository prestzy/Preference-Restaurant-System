# Stakeholder Overview

## What the System Does

The Preference-Driven Restaurant Ordering System is a web-based QR ordering prototype for a single restaurant.

Customers use their own phone to:

- browse the menu
- search dishes
- choose preferences
- receive dish recommendations
- add dishes to cart
- place a prototype order

Restaurant staff use the Admin page to:

- view dashboard metrics
- monitor live orders
- update order status
- manage dishes in memory
- test recommendation scenarios

## Why QR-Based Web Ordering

The original standalone application direction required a dedicated device. The new web direction is more realistic for small restaurants because customers can scan a QR code and use their own phone.

This lowers hardware cost while still allowing the FYP to demonstrate recommendation logic and ordering workflow.

## Recommendation Approach

The system is designed to be explainable, not a black box.

Signals:

- liked ingredients
- disliked ingredients
- preferred tags
- selected/current dishes
- historical co-order patterns

Algorithms:

- content/ingredient-based filtering
- co-order collaborative filtering
- hybrid scoring
- popularity fallback
- association-rule metrics: support, confidence, and lift
- simple time-context boosting for breakfast, lunch, dinner, and snack/dessert

Recommendation cards and the admin tester show the reason, score breakdown, and co-order metrics so a lecturer or stakeholder can understand why a dish was suggested.

## Customer Experience

The customer-facing layout uses a warm Catppuccin Latte mobile-first design:

1. Search bar.
2. Category chips.
3. Preference panel.
4. Recommended for You horizontal cards.
5. Menu grid/list.
6. Dish detail modal.
7. Cart and checkout.
8. Bottom navigation.

Dish images are local and optional. If an image is missing, the UI shows a clean placeholder instead of failing.

## Staff/Admin Experience

The Admin page supports prototype operations:

- live order table
- order status workflow
- dish management
- recommendation testing/evaluation
- historical order table

Admin changes are in memory for the running server session. CSV export provides a simple way to save the current demo state.

## Prototype Scope

This is not a production commercial ordering system. It intentionally avoids heavy infrastructure so the FYP can focus on:

- recommendation explainability
- practical QR ordering flow
- CSV-based dataset management
- simple Rust architecture
- low coupling and high cohesion

## Future Improvements

Suggested future work:

- real price column
- persistent database
- QR table/session IDs
- admin authentication
- kitchen display page
- payment integration
- stronger evaluation metrics
- automatic CSV persistence after admin edits
