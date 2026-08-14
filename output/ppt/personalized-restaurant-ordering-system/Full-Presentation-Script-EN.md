# Personalized Restaurant Ordering System — Full Presentation Script

> Suggested delivery: speak at a calm, natural pace. The slide presentation should take about 12–14 minutes, followed by a 3–4 minute live demonstration. The total should remain within 16–18 minutes. Text inside square brackets is a presenter cue and should not be spoken aloud.

## Slide 01 — Personalized Restaurant Ordering System

Good morning, supervisors. My name is Yeap Chan Leong, and Today, I will present my Final Year Project, titled **Personalized Restaurant Ordering System**. My supervisor is Professor Serge Demidenko.

Many of us have experienced the same situation in a restaurant. We open a long menu, look through it for several minutes and finally order the same hot dishes despite the dishes might not match our preference. The problem is not a lack of choices. The problem is that there are many choices, but very little guidance about which dish matches our current preferences.

Therefore, I developed a ordering system for a small restaurant.
The system recommends dishes using three main ideas: ingredient preferences, historical co-order patterns, and hybrid scoring. More importantly, it explains why each dish was recommended.

The aim is not to build a very complex artificial intelligence model. The aim is to build a system that is lightweight, usable, and easy to understand.

[Move to the next slide.]

I will begin by explaining the problem that motivated this project.

## Slide 02 — Problem and Research Gap

The main problem can be summarised in one sentence: **small restaurants may benefit from personalisation, but they usually do not have the same amount of data as large food-delivery platforms.**

I divided this problem into four areas.

First, the historical data is limited. A large platform may have millions of orders, but a single restaurant may have only tens or hundreds of order baskets.

Second, new customers create a cold-start problem. Cold start simply means that the system has never seen the customer before, so it cannot refer to that customer’s past choices.

Third, customer restrictions must be respected. If a customer explicitly dislikes banana, the system should not recommend Pisang Goreng simply because it is popular. In this project, a disliked ingredient is not treated as a small score penalty. It is treated as a hard exclusion.

Fourth, recommendations need to be transparent. If the system only displays a score such as 0.85, the customer and administrator still do not know where that number came from or whether the supporting evidence is strong.



[Move to the next slide.]

Based on these problems, I defined three research questions.

## Slide 03 — Research Questions and Objectives

The three research questions focus on three recommendation behaviours that can be observed and measured.

Research Question 1 asks how liked and disliked ingredients affect dish ranking and restriction compliance. I mainly observe whether compatible dishes move upward and whether conflicting dishes are excluded correctly.

Research Question 2 asks how increasing the evidence that two dishes were ordered together affects their association measures and recommendation ranking.

For example, if many customers order Chicken Satay together with Nasi Lemak, Chicken Satay may become a reasonable complementary recommendation after a customer selects Nasi Lemak.

Research Question 3 compares three methods: ingredient-only, co-order-only, and hybrid recommendation. In this experiment, one dish is hidden from a historical order, and the system is tested to see whether it can recover that hidden dish.

The research objectives can be grouped into three outcomes. The first is **Build**: develop a complete mobile ordering artefact. The second is **Recommend**: implement an explainable hybrid recommendation engine. The third is **Evaluate**: provide repeatable experiments that do not modify the real operational data.

[Move to the next slide.]

Next, I will show how these research functions are connected to the complete customer and staff workflow.

## Slide 04 — Proposed System and User Journey

The system has two main user roles: the customer and the restaurant staff.

The customer first scans a QR code or opens the website directly. The customer creates a temporary dining session, browses the complete menu, searches for dishes, selects preferences, and reviews recommendation reasons. Suitable dishes will be added to the cart and submitted as an order.

After checkout, the order appears immediately on the administrator page. Staff can update the status from Pending to Preparing, Ready, and finally Completed. The customer can see the same status from the customer interface.

There is also an important feedback loop. Only after an order is marked as Completed will be added to the historical orders. The completed basket can then influence future popularity and co-order calculations.

For example, if more completed orders contain both D01 and D09, the system gradually receives stronger evidence that those two dishes are commonly ordered together.

[Move to the next slide.]

To support this workflow while keeping the system lightweight, I used a modular web architecture.

## Slide 05 — Lightweight Web Architecture

From the user’s perspective, customers use a phone browser and staff use an administrator browser. Both communicate with the same Rust web application.

The browser side uses server-rendered HTML, JavaScript, and mobile-first CSS. The back end uses Rust, Axum, and Tokio.

Axum can be understood as a receptionist. It receives a web request and sends the task to the correct part of the system.

Internally, Search, Recommendation, Cart and Orders, and Persistence have separate responsibilities. The search module finds dishes. The recommendation module calculates scores and explanations. The order module manages carts and order statuses. The persistence module manages file operations.

This supports low coupling and high cohesion. In simple terms, the modules depend on each other as little as practical, and each module focuses on one responsibility.

The current data is mainly stored in CSV and JSONL files. During normal operation, the application does not call an external machine-learning service,or a large language model. This makes the local prototype easier to reproduce.

However, CSV is suitable for this FYP prototype, not necessarily for a high-concurrency commercial system. A future implementation should consider a relational database, transactions, backups, and more complete access control.

[Move to the next slide.]

I will now explain the most important part of the system: how a recommendation is produced.

## Slide 06 — Recommendation Processing Pipeline

The recommendation process contains six stages, but the most important principle is: **filter first, then score.**

The system first receives customer inputs such as liked ingredients, disliked ingredients, preferred tags, selected dishes, and simple time context.

It then applies a hard eligibility gate. Unavailable dishes, dishes that are already selected, and dishes containing any disliked ingredient are excluded before scoring begins.

This is important. If a customer dislikes beef then the beef dish cannot return to the recommendation list eventhough it is popular or frequently ordered with another dish.

For the remaining eligible dishes, the system calculates four signals.

The content score measures ingredient and tag matching. The co-order score measures whether dishes commonly appear together. The popularity score provides a general fallback signal. The time score introduces simple business context.

The system then adjusts the weights according to the available evidence. For example, if the customer has not selected any dish, the co-order evidence is weaker, so the system can rely more on explicit preferences. The weights always add up to 1.

Finally, the system applies deterministic ranking and diversity reranking. It returns the ranked dishes, component scores, a plain-language explanation, and an evidence-confidence indicator. The same input and historical data produce the same output, making the behaviour easier to test and explain.

[Move to the next slide.]

However, a rank alone is not sufficient. The user also needs to understand why a dish was recommended.

## Slide 07 — Explainability and Evidence Confidence

The key point on this slide is that **the recommendation reason is linked to specific evidence rather than a vague statement such as “the algorithm thinks you will like this.”**

For example, the system may explain: “This dish is recommended because it contains your liked ingredients, chicken and chilli; it matches your preferred spicy tag; and it is often ordered with Nasi Lemak.”

This sentence can be traced back to four sources. Chicken and chilli come from the liked ingredients. Spicy comes from the preferred tag. The connection with Nasi Lemak comes from historical co-orders. These signals contribute to the component scores and final ranking.
 

[Move to the next slide.]

These recommendation ideas have been implemented as working customer, administrator, and evaluation features.

## Slide 08 — Implemented System Outcomes

This slide summarises the completed customer, administrator, and evaluation functions. Together, they form one working restaurant-ordering prototype rather than an isolated recommendation algorithm. I will demonstrate the main functions after the presentation, so I will move directly to the testing evidence.

[Move to the next slide.]

## Slide 09 — System Testing Results

The system passed all **115 Rust automated tests** and all **55 report-level checks**. These covered the recommendation rules, search, ordering workflow, persistence, security checks, and responsive layouts.

One end-to-end test confirmed that an order could move from customer checkout to administrator processing, be saved once after completion, and be loaded again after restart.

These results confirm correct behaviour in the controlled test environment. They do not prove production readiness because large-scale load and external customer usability were not evaluated.

[Move to the next slide.]

## Slide 10 — RQ1 and RQ2: Observable Ranking Impact

The results for Research Question 1 show that explicit preferences can change both ranking and eligibility.

When bean sprouts was selected as a liked ingredient, Char Kway Teow, or D14, moved from rank 10 to rank 1. This shows that the system recognised the match between the dish and the selected ingredient.

When banana was selected as a disliked ingredient, Pisang Goreng, or D30, moved from rank 1 to Excluded. It did not simply fall to the bottom of the list. It became ineligible because a disliked ingredient is a hard restriction.

Research Question 2 examines historical co-order evidence. After adding three temporary baskets containing both Nasi Lemak and Sambal Sotong, Sambal Sotong moved from rank 7 to rank 1.


[Move to the next slide.]

I will now compare the three complete recommendation methods.

## Slide 11 — RQ3: Recommendation Method Comparison

Research Question 3 compares the methods using a hidden-dish test. One dish is removed from a historical order, and the system must recover it within the top three recommendations.

Across five cases, ingredient-only achieved **20 percent Hit@3** with an average rank of **14.20**. Co-order-only and fixed hybrid both achieved **100 percent Hit@3**, with an average rank of **1.80**.

The co-order signal was stronger because the historical baskets contained useful dish relationships, while the same rice-and-chicken preference was used in every case.

Therefore, co-order-only and hybrid performed better in this controlled experiment. However, five cases are not enough to claim that either method will always perform better.

[Move to the next slide.]

For this reason, the contributions and limitations should be considered together.

## Slide 12 — Contributions, Limitations and Future Work

The project contributes a complete mobile ordering prototype with hard ingredient restrictions, explainable hybrid recommendations, and controlled evaluation tools that do not alter operational data.

Its main limitations are the small single-restaurant dataset, controlled test conditions, CSV storage, and the lack of a large real-customer study. Evidence confidence should also not be interpreted as purchase probability.

Future work should collect longer-term order data, test the system with real customers, and compare the methods using more cases. For deployment, CSV should be replaced with a database, together with stronger security, backup, and load testing.

[Move to the next slide.]

I will now conclude the presentation the live demonstration.

## Slide 13 — Conclusion and Live Demo



---

# Live Demonstration Script — Approximately 3–4 Minutes

## Step 1 — Customer Entry and the Complete Menu

[Open the customer page.]

I will first simulate a customer entering the restaurant system. After the customer provides simple temporary session details, the home page becomes available.

The upper part of the home page contains search and recommendations. The lower part contains the complete Menu.

I will now search for “laksa”. The system immediately shows matching suggestions. However, notice that the Menu still displays every available dish. Search helps the customer locate a dish; it does not limit the customer to the search results.

[Select one search suggestion.]

When I select a suggestion, the page scrolls to that dish and highlights it temporarily. All other menu dishes remain visible.

## Step 2 — Preferences and Recommendation Reasons

[Open Personalise Recommendations.]

Next, I will select one liked ingredient, such as chicken; one disliked ingredient, such as beef; and one preferred tag, such as spicy.

These options are generated from the current menu data, so customers do not need to guess which words the system understands.

The same ingredient cannot remain both liked and disliked. This prevents conflicting preference states.

[Select one current dish.]

I will also select one dish as the current order context. The recommendations update automatically.

When I select “Why this?”, the system shows matched ingredients, tags, co-order influence, and evidence strength. A dish containing beef cannot enter the recommendation list because the disliked ingredient is applied as a hard exclusion.

## Step 3 — Cart, Checkout, and Order Status

[Add at least two dishes and open the cart.]

I will now add two dishes to the cart. The customer can change quantities and view the item subtotal and total price.

I will submit the prototype checkout.

[Submit the order and open Profile or Orders.]

The order is created with a Pending status. The customer can view the ordered dishes, total price, and latest status here.

## Step 4 — Administrator Processing and Historical Evidence

[Open the administrator page and log in.]

I will now switch to the administrator page. The newly created live order appears immediately.

The administrator can update it from Preparing to Ready and finally to Completed.

[Update the status and return briefly to the customer page.]

The customer page shows the same updated status, demonstrating that both interfaces refer to the same order.

[Mark the order as Completed.]

When the order becomes Completed for the first time, the system appends its dish basket to `orders.csv` and updates the in-memory historical orders.

If Completed is submitted again for the same order, it is not appended twice. This prevents duplicate behavioural evidence.

The completed basket can now contribute to popularity, co-order relationships, and hybrid recommendation calculations. This is a local single-server prototype, so its long-term behaviour under real restaurant traffic is still **to be confirmed**.

[Open Historical Orders or the Recommendation Tester.]

Finally, the administrator can inspect historical orders and run controlled recommendation tests. These experiments use temporary data and do not change the real operational history.

This concludes my live demonstration. Thank you. I am ready to answer your questions.

---

# Short Answers for Common Defence Questions

## Why did you not use a large machine-learning model?

This project targets a single restaurant with limited data. A complex model would require more training data, computing resources, and maintenance, while also being more difficult to explain. My goal was to establish a transparent, deterministic, and repeatable baseline first. If more data becomes available, a future study can compare whether a complex model provides enough additional benefit to justify its cost.

## Why did ingredient-only achieve only 20 percent Hit@3?

All five cases used the same liked ingredients, rice and chicken, but the hidden target dish did not always match those ingredients. Therefore, the available content signal was limited. This result applies only to the controlled setup and does not prove that ingredient-based recommendation is generally ineffective.

## Why did co-order-only and fixed hybrid produce the same result?

The five cases contained strong historical co-order evidence, and the fixed hybrid gave co-order a higher weight. As a result, both methods achieved the same Hit@3 and average rank. More cases and ablation experiments are needed to determine how they differ under other evidence conditions.

## Why did you use CSV instead of a database?

CSV is simple, transparent, and suitable for inspecting algorithm inputs and reproducing an FYP prototype. For a real deployment, I would migrate to a relational database to support transactions, concurrent access, backups, and more reliable recovery.

## Can this system increase restaurant sales?

That conclusion cannot currently be made. This project evaluates system functionality and recommendation behaviour. It does not include a real commercial A/B test. The effects on sales, customer satisfaction, and decision time are **to be confirmed**.
