# Personalized Restaurant Ordering System

## Reduced 11-Slide Presentation Script

**Slides removed from the original presentation:**

- Original Slide 7: Explainability and Evidence Confidence
- Original Slide 8: Implemented System Outcomes

Their essential information has been merged into the recommendation pipeline and system journey slides.

**Target duration:**

- Main presentation: approximately 9-11 minutes
- Live demonstration: approximately 3-4 minutes
- Expected total: approximately 13-15 minutes

Text inside square brackets is a presenter instruction and should not be spoken.

---

## Slide 1 - Personalized Restaurant Ordering System

Good morning. I am Yeap Chan Leong, student ID 22049837. My project is the **Personalized Restaurant Ordering System**, supervised by Professor Serge Demidenko.

Customers often see many dishes but receive little help in deciding what suits their preferences. My project addresses this through a lightweight web system that customers can access using their own phones.

The system recommends dishes using ingredient preferences, previous co-order patterns, and hybrid scoring. It also explains why each dish is recommended.

The goal is not to build a complex artificial intelligence model. The goal is to create a practical and understandable prototype for a small restaurant.

[Next slide.]

---

## Slide 2 - Problem and Research Gap

Small restaurants do not have the same resources or amount of customer data as large food-delivery platforms.

This creates four main problems. Historical order data may be limited. New customers have no personal order history. Customer restrictions must be respected. Finally, recommendation scores can be difficult to understand without an explanation.

For example, if a customer dislikes banana, a banana dish should be excluded even when it is popular.

Therefore, this project uses a lightweight, deterministic, explainable, and mobile-first approach. However, whether the system improves sales or customer satisfaction still needs to be confirmed through a real customer study.

[Next slide.]

---

## Slide 3 - Research Questions and Objectives

This project investigates three questions.

First, how do liked and disliked ingredients affect dish ranking and exclusion?

Second, how does stronger evidence that two dishes were ordered together affect recommendation ranking?

Third, how do ingredient-only, co-order-only, and hybrid recommendation compare when recovering a hidden dish from an order?

The objectives are therefore to build a complete mobile ordering prototype, implement an explainable hybrid recommendation engine, and evaluate its behaviour using repeatable tests.

[Next slide.]

---

## Slide 4 - Proposed System and User Journey

The system has two main users: the customer and restaurant staff.

A customer opens the menu using a phone, selects preferences, reviews recommendations, adds dishes to the cart, and places an order.

The order then appears on the administrator page. Staff can update its status from Pending to Preparing, Ready, and Completed. The customer can see the updated status as well.

Only completed orders are added to the historical data. This creates a feedback loop because completed dish combinations can strengthen future popularity and co-order recommendations.

The implemented system also includes menu search, recommendation explanations, order tracking, dish management, historical orders, and recommendation testing. I will show the main functions during the live demonstration.

[Next slide.]

---

## Slide 5 - Lightweight Web Architecture

Customers and staff access the same Rust web application through their browsers.

The back end uses Rust, Axum, and Tokio. The browser interface uses server-rendered HTML, JavaScript, and mobile-first CSS.

The code is separated into focused modules for search, recommendation, carts and orders, and data persistence. In simple terms, each module has one clear responsibility and depends as little as possible on unrelated modules.

The prototype stores its main data in CSV and JSONL files, while dish images are stored locally. This keeps the system simple and easy to inspect for an FYP prototype.

For a real deployment, CSV should be replaced with a database to support concurrent users, transactions, backups, and recovery.

[Next slide.]

---

## Slide 6 - Recommendation Processing Pipeline

The main recommendation rule is **filter first, then score**.

The system first excludes unavailable dishes, dishes already selected, and dishes containing disliked ingredients. This means popularity cannot override a customer's restriction.

Eligible dishes are then evaluated using four signals. Content scoring measures matching ingredients and tags. Co-order scoring measures which dishes commonly appear together. Popularity provides a general fallback, while time context adds a small situational signal.

The system adjusts the weights based on available information. For example, when no dish is selected, there is less co-order evidence, so the system relies more on explicit preferences.

Each result also includes a simple explanation. For example, a dish may be recommended because it matches chicken, matches the spicy tag, and is often ordered with Nasi Lemak.

The evidence-confidence label describes the strength of the supporting evidence. It is not the probability that the customer will purchase or like the dish.

[Next slide.]

---

## Slide 7 - System Testing Results

This is original Slide 9 in the full presentation.

The Rust automated test suite contains **115 tests**. All 115 passed, with zero failed and zero ignored. These tests cover CSV validation, search, preference conflicts, recommendation rules, deterministic ranking, cart behaviour, and order persistence.

I also completed **55 report-level checks**. These included component, integration, end-to-end, security, and responsive-interface testing across mobile, tablet, and desktop sizes.

One important test followed a customer order through checkout, administrator status updates, completion, CSV storage, and application restart. The completed order was saved only once and loaded again successfully.

These results confirm that the tested requirements worked in the controlled environment. They do not prove that the system is ready for large-scale commercial use.

[Next slide.]

---

## Slide 8 - RQ1 and RQ2: Observable Ranking Impact

This is original Slide 10 in the full presentation.

For Research Question 1, selecting bean sprouts as a liked ingredient moved Char Kway Teow, D14, from rank 10 to rank 1.

Selecting banana as a disliked ingredient changed Pisang Goreng, D30, from rank 1 to Excluded. It was removed completely because disliked ingredients are treated as hard restrictions.

When rice was liked and banana was disliked, Ketupat moved from rank 2 to rank 1. This shows that positive preferences and restrictions can operate together.

For Research Question 2, I temporarily added three order baskets containing Nasi Lemak and Sambal Sotong. Sambal Sotong, D07, then moved from rank 7 to rank 1.

Chicken Satay received stronger association evidence but remained at rank 1 because it was already in the highest possible position.

These controlled results show that preference and co-order evidence affected ranking as designed. Results with a larger real-world dataset remain to be confirmed.

[Next slide.]

---

## Slide 9 - RQ3: Recommendation Method Comparison

This is original Slide 11 in the full presentation.

For this comparison, I treated the recommender like a small quiz. I hid one dish from a historical order and tested whether each method could place that dish in its top three recommendations.

Five historical-order cases were tested.

Ingredient-only achieved a **20 percent Hit at 3**, meaning it recovered one hidden dish within the top three. Its average hidden-dish rank was 14.20.

Co-order-only and fixed hybrid both achieved **100 percent Hit at 3**. They recovered all five hidden dishes within the top three, with an average rank of 1.80.

The historical co-order relationships were stronger than the fixed rice-and-chicken preference in these cases.

However, this does not prove that ingredient recommendation is generally ineffective or that hybrid recommendation is always best. The evaluation only used five controlled cases, so wider generalisation requires more data and testing.

[Next slide.]

---

## Slide 10 - Contributions, Limitations and Future Work

This is original Slide 12 in the full presentation.

The project makes four main contributions. It integrates mobile ordering and recommendation in one working prototype. It treats disliked ingredients as hard restrictions. It provides explainable hybrid scoring. It also includes controlled evaluation tools that do not modify real operational data.

The main limitations are the small historical dataset, the single-restaurant setting, controlled test cases, CSV storage, and the absence of a large real-customer study.

Future work should test the interface with real customers, collect longer-term order data, compare the methods using more cases, and replace CSV storage with a database.

Commercial outcomes such as increased sales or customer satisfaction are not proven by this prototype and would require a separate real-world study.

[Next slide.]

---

## Slide 11 - Conclusion and Live Demo

This is original Slide 13 in the full presentation.

In conclusion, I developed a working Rust web prototype for a small restaurant with limited data.

Customers can browse the menu and order using their own phones. The recommendation system combines explicit preferences with historical ordering behaviour and explains the supporting reasons. Completed orders then become new evidence for future recommendations.

The tests and controlled experiments show that the prototype behaves according to its current design. They do not prove large-scale commercial performance.

I will now demonstrate four parts of the system: the customer menu, personalised recommendations, checkout and order status, and the administrator workflow.

Thank you. I will now begin the live demonstration.

---

# Live Demonstration Script

## Step 1 - Customer Menu and Search

[Open the customer page.]

This is the phone-based customer menu. The customer can browse every available dish.

I will search for "laksa." Matching suggestions appear immediately, but the complete Menu remains unchanged because search is only used to locate a dish.

[Select a suggestion.]

The page scrolls to and highlights the selected dish while keeping all other dishes visible.

## Step 2 - Preferences and Recommendations

[Open Personalise Recommendations.]

I will select chicken as a liked ingredient, beef as a disliked ingredient, and spicy as a preferred tag.

These options are generated from the actual menu, so the customer does not need to guess which terms are supported. An ingredient also cannot remain both liked and disliked.

[Select a current dish and open a recommendation reason.]

The recommendations update automatically. The explanation shows matched preferences, co-order influence, and supporting evidence. Dishes containing beef are excluded before scoring.

## Step 3 - Cart and Checkout

[Add two dishes and open the cart.]

The cart allows the customer to change quantities and view subtotals and the total price.

[Submit the order.]

Checkout creates a Pending order. The order is immediately available to the customer and the administrator.

## Step 4 - Administrator and Historical Orders

[Open the administrator page.]

Staff can update the order from Pending to Preparing, Ready, and Completed. The customer sees the same status.

When the order is marked Completed for the first time, its dish basket is added once to the historical CSV. It immediately becomes available to popularity, co-order, and hybrid recommendation calculations and remains available after restart.

The administrator can also review historical orders and run controlled recommendation tests without modifying operational data.

This concludes the demonstration. I am ready for your questions.

---

# Short Defence Answers

## Why did you not use a large machine-learning model?

The prototype targets one restaurant with limited data. A transparent method is easier to reproduce, maintain, test, and explain. A more complex model should only be considered after enough real data is collected.

## Why was ingredient-only lower in the comparison?

All five cases used the same rice-and-chicken preference, but the hidden target did not always match those ingredients. The result is specific to this setup and does not prove that ingredient-based recommendation is generally ineffective.

## Why did co-order-only and hybrid produce the same result?

The five selected cases contained strong co-order relationships, and the hybrid method gave co-order evidence a higher weight. More cases are needed to reveal differences under other conditions.

## Why use CSV instead of a database?

CSV keeps the FYP prototype simple and makes the algorithm inputs easy to inspect. A deployed system should use a database for transactions, concurrency, backups, and recovery.

## Can this system increase restaurant sales?

That is not proven. This project evaluates functionality and recommendation behaviour. Sales and customer satisfaction require a real-world customer study.
