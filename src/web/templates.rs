use crate::models::Order;
use crate::preferences::PreferenceOptions;
use crate::web::state::{
    AdminView, CustomerSession, DishView, LiveOrder, MenuView, OrderStatus, RecommendationView,
};

/// Renders the customer-facing QR menu home page.
pub fn customer_menu_page(view: &MenuView, session: &CustomerSession) -> String {
    let recommended_cards = view
        .recommended
        .iter()
        .map(recommended_card)
        .collect::<String>();
    let menu_cards = view.dishes.iter().map(menu_card).collect::<String>();
    let preference_panel = preference_panel(&view.preference_options, "customer");
    let content = format!(
        r#"
        <section class="search-panel unified-search-panel" aria-label="Search menu and preferences">
            <p class="eyebrow">Preston's Restaurant</p>
            <h1>Welcome, {customer_name}</h1>
            <label class="search-box">
                <span class="search-icon">⌕</span>
                <input id="search-input" type="search" aria-label="Find a dish" placeholder="Find dishes, ingredients, tags, or taste..." autocomplete="off">
            </label>
            <div class="search-suggestions" id="search-suggestions" hidden></div>
            <p class="assistant-understood" id="assistant-understood"></p>
            <div class="assistant-upsells" id="assistant-upsells"></div>
        </section>

        {preference_panel}

        <section class="recommendation-band" aria-labelledby="recommended-title">
            <div class="section-heading">
                <div>
                    <h2 id="recommended-title">Recommended for You</h2>
                    <p>Based on your preferences and ordering patterns</p>
                </div>
                <div class="carousel-controls" aria-label="Recommended dish carousel controls">
                    <button class="carousel-arrow" type="button" data-carousel-scroll="recommended-row" data-direction="-1" aria-label="Scroll recommendations left">‹</button>
                    <button class="carousel-arrow" type="button" data-carousel-scroll="recommended-row" data-direction="1" aria-label="Scroll recommendations right">›</button>
                </div>
            </div>
            <div class="recommended-row" id="recommended-row">
                {recommended_cards}
            </div>
            <div class="evaluation-strip" id="recommendation-stats"></div>
        </section>

        <section class="menu-section" aria-labelledby="menu-title">
            <div class="section-heading">
                <div>
                    <h2 id="menu-title">Menu</h2>
                    <p><span id="visible-count">{dish_count}</span> dish(es) available</p>
                </div>
            </div>
            <div class="dish-grid" id="dish-grid">
                {menu_cards}
            </div>
        </section>

        {dish_modal}
        "#,
        customer_name = escape_html(&session.customer_name),
        dish_count = view.dishes.len(),
        dish_modal = dish_detail_modal()
    );

    page_shell(
        "Home",
        "home",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
        &view.search_vocabulary_json,
        Some(session),
    )
}

/// Renders the prototype cart page.
pub fn cart_page(view: &MenuView, session: &CustomerSession) -> String {
    let content = format!(
        r#"
        <section class="plain-page">
            <p class="eyebrow">Preston's Restaurant</p>
            <h1>Cart</h1>
            <p>Review selected dishes. Checkout creates a live order for staff and your Profile page.</p>
            <div class="admin-card customer-summary-card">
                <strong>Ordering for: {name}</strong>
                <span>Table: {table}</span>
                <span>Phone: {phone}</span>
                <a class="ghost-link" href="/profile">Edit in Profile</a>
            </div>
            <div id="cart-page-items" class="cart-list"></div>
            <label class="field-label order-note-label">Order note
                <input id="customer-note" placeholder="Optional, e.g. less spicy or no cutlery">
            </label>
            <div class="cart-summary">
                <strong id="cart-page-total">RM 0</strong>
                <button class="primary-action" id="checkout-button" type="button">Place Order</button>
            </div>
            <p class="muted">FYP prototype: no payment is processed.</p>
            <p class="status-message" id="checkout-status"></p>
        </section>
    "#,
        name = escape_html(&session.customer_name),
        table = escape_html(&session.table_number),
        phone = escape_html(&session.masked_phone())
    );

    page_shell(
        "Cart",
        "cart",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
        &view.search_vocabulary_json,
        Some(session),
    )
}

pub fn customer_start_page(
    previous: Option<&crate::web::handlers::customer::CustomerRegistrationForm>,
    message: Option<&str>,
) -> String {
    let name = previous
        .map(|form| escape_attr(&form.customer_name))
        .unwrap_or_default();
    let phone = previous
        .map(|form| escape_attr(&form.customer_phone))
        .unwrap_or_default();
    let table = previous
        .map(|form| escape_attr(&form.table_number))
        .unwrap_or_default();
    let message = message
        .map(|message| {
            format!(
                r#"<p class="status-message error">{}</p>"#,
                escape_html(message)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <link rel="icon" href="data:,">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="Preston's Restaurant mobile ordering menu">
    <title>Start Order | Preston's Restaurant</title>
    <link rel="stylesheet" href="/static/app.css?v=20260724-brand-responsive">
    <script src="/static/auth.js?v=20260724" defer></script>
</head>
<body>
    <main class="app-shell start-shell">
        <section class="admin-card start-card">
            <p class="eyebrow">Preston's Restaurant</p>
            <h1>Start Your Dining Session</h1>
            <p class="muted">Enter basic details once. Checkout and order tracking will use this temporary session.</p>
            {message}
            <form class="admin-form" method="post" action="/start" data-auth-form>
                <label class="field-label">Customer name<input name="customer_name" value="{name}" placeholder="Customer name" autocomplete="name" required></label>
                <label class="field-label">Phone number<input name="customer_phone" value="{phone}" placeholder="e.g. 0123456789" autocomplete="tel" inputmode="tel" required></label>
                <label class="field-label">Table number<input name="table_number" value="{table}" placeholder="e.g. T05" required></label>
                <button class="primary-action" type="submit" data-submitting-label="Continuing...">Enter Menu</button>
            </form>
            <p class="privacy-note">Your details are used only for this dining session, order fulfilment, and order-status communication.</p>
            <a class="ghost-link" href="/admin/login">Staff Admin Login</a>
        </section>
    </main>
</body>
</html>"#
    )
}

/// Renders the customer Profile page, replacing the old separate Orders page.
pub fn profile_page(view: &MenuView, session: &CustomerSession, orders: &[LiveOrder]) -> String {
    profile_page_inner(view, session, orders, None)
}

pub fn profile_page_with_message(
    view: &MenuView,
    session: &CustomerSession,
    orders: &[LiveOrder],
    message: &str,
) -> String {
    profile_page_inner(view, session, orders, Some(message))
}

fn profile_page_inner(
    view: &MenuView,
    session: &CustomerSession,
    orders: &[LiveOrder],
    message: Option<&str>,
) -> String {
    let order_cards = if orders.is_empty() {
        r#"<div class="info-card"><strong>No current orders yet.</strong><span>Place an order from the cart to track it here.</span></div>"#.to_string()
    } else {
        orders.iter().map(order_card).collect::<String>()
    };
    let message = message
        .map(|message| {
            format!(
                r#"<p class="status-message error">{}</p>"#,
                escape_html(message)
            )
        })
        .unwrap_or_default();
    let content = format!(
        r#"
        <section class="plain-page">
            <p class="eyebrow">Preston's Restaurant</p>
            <h1>Profile</h1>
            <p>View your dining-session details and track your own order status.</p>
            <section class="admin-card customer-profile-card">
                <div class="section-heading"><h2>Customer Profile</h2><p>Temporary details for this QR dining session.</p></div>
                {message}
                <p><strong>Name:</strong> {name}</p>
                <p><strong>Phone:</strong> {phone}</p>
                <p><strong>Table:</strong> {table}</p>
                <details>
                    <summary>Edit details</summary>
                    <form class="admin-form compact-form" method="post" action="/profile">
                        <label class="field-label">Customer name<input name="customer_name" value="{name_attr}" placeholder="Customer name" required></label>
                        <label class="field-label">Phone number<input name="customer_phone" value="{phone_attr}" placeholder="Phone number" inputmode="tel" required></label>
                        <label class="field-label">Table number<input name="table_number" value="{table_attr}" placeholder="Table number" required></label>
                        <button class="primary-action" type="submit">Save Profile</button>
                    </form>
                </details>
                <form method="post" action="/profile/end">
                    <button class="ghost-action" type="submit">End Session</button>
                </form>
            </section>
            <section class="order-filter-row" aria-label="Order status filters">
                <button class="chip active" type="button" data-order-filter="all">All</button>
                <button class="chip" type="button" data-order-filter="pending">Pending</button>
                <button class="chip" type="button" data-order-filter="preparing">Preparing</button>
                <button class="chip" type="button" data-order-filter="ready">Ready</button>
                <button class="chip" type="button" data-order-filter="completed">Completed</button>
                <button class="chip" type="button" data-order-filter="cancelled">Cancelled</button>
            </section>
            <p class="muted" id="order-sync-status">Checking for order updates...</p>
            <div class="order-card-list" id="orders-list">{order_cards}</div>
        </section>
        "#,
        name = escape_html(&session.customer_name),
        phone = escape_html(&session.masked_phone()),
        table = escape_html(&session.table_number),
        name_attr = escape_attr(&session.customer_name),
        phone_attr = escape_attr(&session.customer_phone),
        table_attr = escape_attr(&session.table_number),
    );

    page_shell(
        "Profile",
        "profile",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
        &view.search_vocabulary_json,
        Some(session),
    )
}

fn order_card(order: &LiveOrder) -> String {
    format!(
        r#"
        <article class="order-card" data-order-status-card="{status_lower}">
            <div class="order-card-main">
                <div>
                    <p class="eyebrow">{order_id}</p>
                    <h2>{status_badge} {total}</h2>
                </div>
                <span class="status-badge {status_lower}">{status}</span>
            </div>
            <p><strong>Dishes:</strong> {dish_names}</p>
            <p><strong>Dish IDs:</strong> {dish_ids}</p>
            <p><strong>Time:</strong> {timestamp}</p>
        </article>
        "#,
        order_id = escape_html(&order.order_id),
        status = order.status.label(),
        status_lower = order.status.label().to_lowercase(),
        status_badge = escape_html(order.status.label()),
        total = escape_html(&order.total_price),
        dish_names = escape_html(&order.dish_names.join(", ")),
        dish_ids = escape_html(&order.ordered_dishes.join(", ")),
        timestamp = escape_html(&order.timestamp)
    )
}

/// Renders the staff/admin dashboard and management page.
pub fn admin_page(view: &MenuView, admin: &AdminView) -> String {
    let dashboard = admin_dashboard(admin);

    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>Admin</h1>
            <p>Dashboard overview for the QR ordering and recommendation prototype.</p>
            {}
            {dashboard}
        </section>
        "#,
        admin_section_nav("dashboard")
    );

    admin_page_shell(
        "Admin",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
        &view.search_vocabulary_json,
    )
}

pub fn admin_orders_page(view: &MenuView, admin: &AdminView) -> String {
    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>Admin Orders</h1>
            <p>Manage live checkout orders and review saved historical order baskets.</p>
            {}
            {}
            {}
            {}
        </section>
        "#,
        admin_section_nav("orders"),
        live_orders_table(&admin.live_orders),
        completed_orders_table(&admin.completed_session_orders),
        historical_orders_table(&admin.historical_orders)
    );

    admin_page_shell(
        "Admin Orders",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
        &view.search_vocabulary_json,
    )
}

pub fn admin_dishes_page(view: &MenuView, admin: &AdminView) -> String {
    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>Dish Management</h1>
            <p>Add, edit, hide, and review dishes used by the customer menu.</p>
            {}
            {}
        </section>
        "#,
        admin_section_nav("dishes"),
        dish_management_panel(&admin.dishes)
    );

    admin_page_shell(
        "Admin Dishes",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
        &view.search_vocabulary_json,
    )
}

pub fn admin_recommendations_page(view: &MenuView, admin: &AdminView) -> String {
    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>Recommendation Tester</h1>
            <p>Academic scoring view for content, co-ordering, popularity, time boost, and association metrics.</p>
            {}
            {}
        </section>
        "#,
        admin_section_nav("recommendations"),
        recommendation_tester(admin)
    );

    admin_page_shell(
        "Admin Recommendations",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
        &view.search_vocabulary_json,
    )
}

pub fn admin_data_page(view: &MenuView) -> String {
    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>Data Tools</h1>
            <p>CSV tools have been removed from the visible admin workflow to keep staff management simple for the FYP demo.</p>
            {}
            <section class="admin-card">
                <div class="section-heading"><h2>CSV Tools Removed</h2><p>Dish and order data still load from data/dishes.csv and data/orders.csv at startup. Staff-facing management is handled from Dish Management and Orders.</p></div>
            </section>
        </section>
        "#,
        admin_section_nav("data")
    );

    admin_page_shell(
        "Admin Data",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
        &view.search_vocabulary_json,
    )
}

pub fn admin_login_page(username: Option<&str>, message: Option<&str>) -> String {
    let username = username.map(escape_attr).unwrap_or_default();
    let message = message
        .map(|message| {
            format!(
                r#"<p class="status-message error" role="alert">{}</p>"#,
                escape_html(message)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <link rel="icon" href="data:,">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Admin Login</title>
    <link rel="stylesheet" href="/static/app.css?v=20260724-brand-responsive">
    <script src="/static/auth.js?v=20260724" defer></script>
</head>
<body>
    <main class="app-shell admin-login-shell">
        <section class="admin-card login-card">
            <p class="eyebrow">Staff Access</p>
            <h1>Admin Login</h1>
            <p class="muted">Prototype staff area for orders, dishes, recommendation testing, and menu insights.</p>
            {message}
            <form class="admin-form" method="post" action="/admin/login" data-auth-form>
                <input name="username" value="{username}" placeholder="Username" autocomplete="username" required>
                <input name="password" type="password" placeholder="Password" autocomplete="current-password" required>
                <button class="primary-action" type="submit" data-submitting-label="Logging in...">Log In</button>
            </form>
        </section>
    </main>
</body>
</html>"#
    )
}

fn page_shell(
    title: &str,
    active: &str,
    content: &str,
    dishes_json: &str,
    recommendations_json: &str,
    preference_options_json: &str,
    search_vocabulary_json: &str,
    customer_session: Option<&CustomerSession>,
) -> String {
    let customer_json = customer_session
        .and_then(|session| serde_json::to_string(session).ok())
        .unwrap_or_else(|| "null".to_string());
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <link rel="icon" href="data:,">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="Preston's Restaurant menu, recommendations, cart, and order tracking">
    <title>{} | Preston's Restaurant</title>
    <link rel="stylesheet" href="/static/app.css?v=20260724-brand-responsive">
    <script>
        window.MENU_DISHES = {};
        window.RECOMMENDATIONS = {};
        window.PREFERENCE_OPTIONS = {};
        window.SEARCH_VOCABULARY = {};
        window.CUSTOMER_SESSION = {};
    </script>
    <script src="/static/app.js?v=20260724-brand-responsive" defer></script>
</head>
<body>
    <main class="app-shell">
        {content}
    </main>
    {}
</body>
</html>"#,
        escape_html(title),
        dishes_json,
        recommendations_json,
        preference_options_json,
        search_vocabulary_json,
        customer_json,
        bottom_nav(active)
    )
}

fn admin_page_shell(
    title: &str,
    content: &str,
    dishes_json: &str,
    recommendations_json: &str,
    preference_options_json: &str,
    search_vocabulary_json: &str,
) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <link rel="icon" href="data:,">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <link rel="stylesheet" href="/static/app.css?v=20260724-brand-responsive">
    <script>
        window.MENU_DISHES = {};
        window.RECOMMENDATIONS = {};
        window.PREFERENCE_OPTIONS = {};
        window.SEARCH_VOCABULARY = {};
    </script>
    <script src="/static/app.js?v=20260724-brand-responsive" defer></script>
</head>
<body class="admin-body">
    <main class="app-shell admin-shell">
        {content}
    </main>
</body>
</html>"#,
        escape_html(title),
        dishes_json,
        recommendations_json,
        preference_options_json,
        search_vocabulary_json,
    )
}

fn preference_panel(options: &PreferenceOptions, scope: &str) -> String {
    format!(
        r#"
        <section class="preference-card" data-preference-scope="{scope}">
            <details>
                <summary>Personalise Recommendations</summary>
                <p>Select preferences from actual menu ingredients and tags.</p>
                {}
                {}
                {}
                <button class="ghost-action" type="button" data-clear-preferences>Clear preferences</button>
            </details>
        </section>
        "#,
        preference_group(
            "Liked Ingredients",
            "Ingredients the customer wants more often.",
            "liked_ingredients",
            &options.ingredients
        ),
        preference_group(
            "Disliked Ingredients",
            "Dishes containing these are excluded.",
            "disliked_ingredients",
            &options.ingredients
        ),
        preference_group(
            "Preferred Tags",
            "Tags add a smaller content-score bonus.",
            "preferred_tags",
            &options.tags
        )
    )
}

fn admin_section_nav(active: &str) -> String {
    let links = [
        ("dashboard", "Dashboard", "/admin"),
        ("orders", "Orders", "/admin/orders"),
        ("dishes", "Dishes", "/admin/dishes"),
        (
            "recommendations",
            "Recommendation Tester",
            "/admin/recommendations",
        ),
        ("logout", "Logout", "/admin/logout"),
    ];
    let items = links
        .iter()
        .map(|(id, label, href)| {
            format!(
                r#"<a class="admin-nav-link{}" href="{}">{}</a>"#,
                if *id == active { " active" } else { "" },
                escape_attr(href),
                escape_html(label)
            )
        })
        .collect::<String>();

    format!(r#"<nav class="admin-section-nav">{items}</nav>"#)
}

fn preference_group(title: &str, help: &str, kind: &str, values: &[String]) -> String {
    let chips = values
        .iter()
        .map(|value| {
            format!(
                r#"<button class="mini-chip" type="button" data-preference-kind="{kind}" data-preference-value="{}">{}</button>"#,
                escape_attr(value),
                escape_html(value)
            )
        })
        .collect::<String>();

    format!(
        r#"
        <div class="preference-group">
            <h3>{}</h3>
            <p>{}</p>
            <div class="mini-chip-row">{chips}</div>
        </div>
        "#,
        escape_html(title),
        escape_html(help)
    )
}

fn recommended_card(recommendation: &RecommendationView) -> String {
    let dish = &recommendation.dish;
    format!(
        r#"
        <article class="recommendation-card" data-dish-id="{dish_id}">
            {}
            <div class="card-body">
                <h3>{name}</h3>
                <p>{category}</p>
                <span class="reason">{reason}</span>
                <strong>{price}</strong>
                <div class="card-actions">
                    <button class="add-button" data-add-cart="{dish_id}" type="button">Add</button>
                    <button class="ghost-action" data-view-dish="{dish_id}" type="button">Why this?</button>
                </div>
            </div>
        </article>
        "#,
        image_block(dish, "compact"),
        dish_id = escape_attr(&dish.dish_id),
        name = escape_html(&dish.name),
        category = escape_html(&dish.category),
        reason = escape_html(&short_recommendation_reason(recommendation)),
        price = escape_html(&dish.price)
    )
}

fn short_recommendation_reason(recommendation: &RecommendationView) -> String {
    let mut parts = Vec::new();
    let mut preference_matches = recommendation.matched_liked_ingredients.clone();
    preference_matches.extend(recommendation.matched_preferred_tags.clone());

    if !preference_matches.is_empty() {
        parts.push(format!(
            "Matches {}",
            preference_matches
                .into_iter()
                .take(3)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let Some(related) = recommendation.related_selected_dishes.first() {
        parts.push(format!("Often ordered with {related}"));
    }
    if recommendation.popularity_score > 0.0 && parts.is_empty() {
        parts.push("Popular from order history".to_string());
    }
    if recommendation.business_rule_score > 0.0 && parts.len() < 2 {
        parts.push("Fits the menu context".to_string());
    }

    if parts.is_empty() {
        "Based on ingredients and order patterns".to_string()
    } else {
        parts.join(" · ")
    }
}

fn menu_card(dish: &DishView) -> String {
    let tags = dish
        .tags
        .iter()
        .take(3)
        .map(|tag| format!(r#"<span class="tag">{}</span>"#, escape_html(tag)))
        .collect::<String>();
    let ingredients = dish.ingredients.join(", ");
    let search_text = format!(
        "{} {} {} {} {}",
        dish.dish_id,
        dish.name,
        dish.category,
        dish.tags.join(" "),
        dish.ingredients.join(" ")
    )
    .to_lowercase();

    format!(
        r#"
        <article id="dish-{dish_id}" class="dish-card" data-dish-id="{dish_id}" data-category="{category_lower}" data-search="{search_text}">
            {}
            <div class="dish-content">
                <div class="dish-title-row">
                    <div>
                        <h3>{name}</h3>
                        <p>{category}</p>
                    </div>
                    {badge}
                </div>
                <div class="tag-row">{tags}</div>
                <p class="ingredients">{ingredients}</p>
                <div class="dish-footer">
                    <strong>{price}</strong>
                    <button class="add-button" data-add-cart="{dish_id}" type="button">Add</button>
                </div>
                <button class="text-action" data-view-dish="{dish_id}" type="button">View details</button>
            </div>
        </article>
        "#,
        image_block(dish, ""),
        dish_id = escape_attr(&dish.dish_id),
        category_lower = escape_attr(&dish.category.to_lowercase()),
        search_text = escape_attr(&search_text),
        name = escape_html(&dish.name),
        category = escape_html(&dish.category),
        badge = if dish.recommended {
            r#"<span class="badge">Recommended</span>"#
        } else {
            ""
        },
        tags = tags,
        ingredients = escape_html(&ingredients),
        price = escape_html(&dish.price)
    )
}

fn image_block(dish: &DishView, extra_class: &str) -> String {
    match &dish.image_url {
        Some(url) => format!(
            r#"<div class="dish-art {extra_class}"><img src="{}" alt="{}"></div>"#,
            escape_attr(url),
            escape_attr(&dish.name)
        ),
        None => format!(
            r#"<div class="dish-art placeholder {extra_class}" aria-label="No image for {}">🍽</div>"#,
            escape_attr(&dish.name)
        ),
    }
}

fn dish_detail_modal() -> &'static str {
    r#"
    <dialog class="dish-modal" id="dish-detail-modal">
        <div class="modal-content">
            <button class="modal-close" type="button" data-close-dish-modal>×</button>
            <div id="dish-detail-content"></div>
        </div>
    </dialog>
    "#
}

fn admin_dashboard(admin: &AdminView) -> String {
    let frequent = frequency_list(&admin.frequent_dishes);
    let pairs = frequency_list(&admin.co_order_pairs);
    format!(
        r#"
        <section class="dashboard-stat-grid">
            <div class="dashboard-stat-card"><div class="dashboard-stat-label">Total dishes</div><div class="dashboard-stat-value">{}</div></div>
            <div class="dashboard-stat-card"><div class="dashboard-stat-label">Available dishes</div><div class="dashboard-stat-value">{}</div></div>
            <div class="dashboard-stat-card"><div class="dashboard-stat-label">Unavailable dishes</div><div class="dashboard-stat-value">{}</div></div>
            <div class="dashboard-stat-card"><div class="dashboard-stat-label">Historical orders</div><div class="dashboard-stat-value">{}</div></div>
            <div class="dashboard-stat-card"><div class="dashboard-stat-label">Live orders</div><div class="dashboard-stat-value">{}</div></div>
            <div class="dashboard-stat-card"><div class="dashboard-stat-label">Completed session orders</div><div class="dashboard-stat-value">{}</div></div>
        </section>
        <section class="admin-two-column">
            <div class="admin-card"><h2>Most Frequent Dishes</h2>{frequent}</div>
            <div class="admin-card"><h2>Common Co-order Pairs</h2>{pairs}</div>
        </section>
        {insights}
        <section class="admin-card">
            <div class="section-heading">
                <div><h2>Quick Actions</h2><p>Open the main staff workflows.</p></div>
            </div>
            <div class="form-actions">
                <a class="primary-link" href="/admin/orders">Manage Orders</a>
                <a class="ghost-link" href="/admin/dishes">Manage Dishes</a>
                <a class="ghost-link" href="/admin/recommendations">Open Recommendation Experiment Lab</a>
            </div>
        </section>
        "#,
        admin.total_dishes,
        admin.available_dishes,
        admin.unavailable_dishes,
        admin.historical_order_count,
        admin.live_order_count,
        admin.completed_session_order_count,
        insights = admin_insight_panel()
    )
}

fn admin_insight_panel() -> String {
    r#"
        <section class="admin-card assistant-card">
            <div class="section-heading">
                <div>
                    <h2>Smart Menu Insights</h2>
                    <p>Rule-based summary calculated from historical order baskets.</p>
                </div>
                <button class="ghost-action" id="refresh-admin-insights" type="button">Refresh</button>
            </div>
            <p class="assistant-understood" id="admin-insight-summary">Loading insights...</p>
            <div class="insight-grid" id="admin-insight-grid"></div>
        </section>
    "#
    .to_string()
}

fn frequency_list(values: &[crate::web::state::FrequencyView]) -> String {
    if values.is_empty() {
        return r#"<p class="muted">No order data yet.</p>"#.to_string();
    }

    let items = values
        .iter()
        .map(|value| {
            format!(
                r#"<li><span>{}</span><strong>{}</strong></li>"#,
                escape_html(&value.label),
                value.count
            )
        })
        .collect::<String>();
    format!(r#"<ul class="frequency-list">{items}</ul>"#)
}

fn live_orders_table(live_orders: &[LiveOrder]) -> String {
    let rows = if live_orders.is_empty() {
        r#"<tr><td colspan="11">No active live customer orders yet.</td></tr>"#.to_string()
    } else {
        live_orders
            .iter()
            .rev()
            .map(|order| {
                format!(
                    r#"
                    <tr data-live-order-row="{order_id}">
                        <td data-label="Order ID">{order_id}</td>
                        <td data-label="Session">{session}</td>
                        <td data-label="Customer">{customer}</td>
                        <td data-label="Phone">{phone}</td>
                        <td data-label="Table">{table}</td>
                        <td data-label="Dish IDs">{dishes}</td>
                        <td data-label="Dish Names">{names}</td>
                        <td data-label="Note">{note}</td>
                        <td data-label="Time">{timestamp}</td>
                        <td data-label="Total">{total}</td>
                        <td data-label="Status">
                            <select data-order-status="{order_id}">
                                {status_options}
                            </select>
                        </td>
                    </tr>
                    "#,
                    order_id = escape_attr(&order.order_id),
                    session = escape_html(&order.session_user_id),
                    customer = escape_html(&order.customer_name),
                    phone = escape_html(&order.customer_phone),
                    table = escape_html(order.table_number.as_deref().unwrap_or("-")),
                    dishes = escape_html(&order.ordered_dishes.join(", ")),
                    names = escape_html(&order.dish_names.join(", ")),
                    note = escape_html(order.note.as_deref().unwrap_or("-")),
                    timestamp = escape_html(&order.timestamp),
                    total = escape_html(&order.total_price),
                    status_options = status_options(order.status)
                )
            })
            .collect::<String>()
    };

    format!(
        r#"
        <section class="admin-card">
            <div class="section-heading"><h2>Live Orders</h2><p>Pending, preparing, and ready checkout orders from this server session.</p></div>
            <p class="status-message" id="admin-order-status"></p>
            <div class="table-wrap">
                <table class="responsive-data-table">
                    <thead><tr><th>Order ID</th><th>Session</th><th>Customer</th><th>Phone</th><th>Table</th><th>Dish IDs</th><th>Dish Names</th><th>Note</th><th>Time</th><th>Total</th><th>Status</th></tr></thead>
                    <tbody id="admin-live-orders-body">{rows}</tbody>
                </table>
            </div>
        </section>
        "#
    )
}

fn completed_orders_table(orders: &[LiveOrder]) -> String {
    let rows = if orders.is_empty() {
        r#"<tr><td colspan="6">No checkout order has been completed in this server session yet.</td></tr>"#.to_string()
    } else {
        orders
            .iter()
            .rev()
            .map(|order| {
                format!(
                    r#"<tr><td data-label="Order ID">{}</td><td data-label="Session">{}</td><td data-label="Dish IDs">{}</td><td data-label="Dish Names">{}</td><td data-label="Timestamp">{}</td><td data-label="Total">{}</td></tr>"#,
                    escape_html(&order.order_id),
                    escape_html(&order.session_user_id),
                    escape_html(&order.ordered_dishes.join(", ")),
                    escape_html(&order.dish_names.join(", ")),
                    escape_html(&order.timestamp),
                    escape_html(&order.total_price)
                )
            })
            .collect::<String>()
    };

    format!(
        r#"
        <section class="admin-card">
            <div class="section-heading"><h2>Completed Orders This Session</h2><p>Checkout orders marked Completed during this server session. They are also saved into data/orders.csv for future recommendation calculations.</p></div>
            <div class="form-actions">
                <a class="ghost-link" href="/admin/export/completed-session-orders.csv">Export Completed Session Orders</a>
            </div>
            <div class="table-wrap">
                <table class="responsive-data-table">
                    <thead><tr><th>Order ID</th><th>Session</th><th>Dish IDs</th><th>Dish Names</th><th>Timestamp</th><th>Total</th></tr></thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
        </section>
        "#
    )
}

fn historical_orders_table(orders: &[Order]) -> String {
    let rows = orders
        .iter()
        .map(|order| {
            format!(
                r#"<tr><td data-label="Order ID">{}</td><td data-label="Session">{}</td><td data-label="Dishes">{}</td><td data-label="Timestamp">{}</td></tr>"#,
                escape_html(&order.order_id),
                escape_html(&order.session_user_id),
                escape_html(&order.ordered_dishes.join(", ")),
                escape_html(&order.timestamp)
            )
        })
        .collect::<String>();

    format!(
        r#"
        <section class="admin-card">
            <div class="section-heading"><h2>Historical Orders</h2><p>Order logs loaded from data/orders.csv, including completed checkout orders saved by the admin flow.</p></div>
            <div class="table-wrap">
                <table class="responsive-data-table">
                    <thead><tr><th>Order ID</th><th>Session</th><th>Dishes</th><th>Timestamp</th></tr></thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
        </section>
        "#
    )
}

fn status_options(selected: OrderStatus) -> String {
    [
        OrderStatus::Pending,
        OrderStatus::Preparing,
        OrderStatus::Ready,
        OrderStatus::Completed,
        OrderStatus::Cancelled,
    ]
    .iter()
    .map(|status| {
        format!(
            r#"<option value="{}"{}>{}</option>"#,
            status.label(),
            if *status == selected { " selected" } else { "" },
            status.label()
        )
    })
    .collect()
}

fn dish_management_panel(dishes: &[DishView]) -> String {
    let rows = dishes
        .iter()
        .map(|dish| {
            format!(
                r#"
                <tr data-admin-dish-row="{dish_id}" data-dish-name="{name_attr}" data-dish-category="{category_attr}" data-dish-ingredients="{ingredients_attr}" data-dish-tags="{tags_attr}" data-dish-price="{price_attr}" data-dish-image-path="{image_path_attr}" data-dish-available="{available_attr}">
                    <td data-label="Image">{image}</td>
                    <td data-label="Dish"><strong>{name}</strong><span>{id_label}</span></td>
                    <td data-label="Category">{category}</td>
                    <td data-label="Ingredients">{ingredients}</td>
                    <td data-label="Status">{availability}</td>
                    <td data-label="Actions">
                        <button class="ghost-action" data-edit-dish="{dish_id}" type="button">Edit</button>
                        <button class="ghost-action" data-toggle-dish="{dish_id}" data-available="{next_available}" type="button">{toggle_label}</button>
                        <button class="danger-action" data-delete-dish="{dish_id}" type="button">Delete</button>
                    </td>
                </tr>
                "#,
                image = image_block(dish, "thumb"),
                name = escape_html(&dish.name),
                id_label = escape_html(&dish.dish_id),
                category = escape_html(&dish.category),
                ingredients = escape_html(&dish.ingredients.join(", ")),
                name_attr = escape_attr(&dish.name),
                category_attr = escape_attr(&dish.category),
                ingredients_attr = escape_attr(&dish.ingredients.join(", ")),
                tags_attr = escape_attr(&dish.tags.join(", ")),
                price_attr = dish.price_amount,
                image_path_attr = escape_attr(dish.image_path.as_deref().unwrap_or_default()),
                availability = if dish.available { "Available" } else { "Unavailable" },
                available_attr = if dish.available { "true" } else { "false" },
                dish_id = escape_attr(&dish.dish_id),
                next_available = if dish.available { "false" } else { "true" },
                toggle_label = if dish.available {
                    "Mark unavailable"
                } else {
                    "Mark available"
                }
            )
        })
        .collect::<String>();

    format!(
        r#"
        <section class="admin-card">
            <div class="section-heading"><h2>Dish Management</h2><p>Search, add, edit, hide, or delete menu records used by the customer app.</p></div>
            <div class="admin-form compact-form">
                <input id="admin-dish-search" placeholder="Search dishes...">
                <select id="admin-availability-filter">
                    <option value="all">All availability</option>
                    <option value="available">Available</option>
                    <option value="unavailable">Unavailable</option>
                </select>
                <button class="primary-action" id="open-dish-form" type="button">+ Add Dish</button>
            </div>
            <div class="modal-backdrop" id="dish-form-modal" hidden>
            <form class="admin-form dish-modal-card" id="dish-form">
                <div class="section-heading"><h2 id="dish-form-title">Add Dish</h2><button class="ghost-action" id="cancel-dish-form" type="button">Cancel</button></div>
                <input name="dish_id" placeholder="Dish ID (optional)">
                <input name="name" placeholder="Dish name" required>
                <input name="price" type="number" min="1" step="1" placeholder="Price in RM" required>
                <input name="category" placeholder="Category" required>
                <input name="ingredients" placeholder="Ingredients: chicken, rice" required>
                <input name="tags" placeholder="Tags: spicy, signature">
                <input name="image_path" placeholder="assets/dishes/D31.jpg">
                <label><input type="checkbox" name="available" checked> Available in customer menu</label>
                <button class="primary-action" type="submit">Save Dish</button>
            </form>
            </div>
            <p class="status-message" id="dish-management-status"></p>
            <div class="table-wrap">
                <table class="responsive-data-table">
                    <thead><tr><th>Image</th><th>Dish</th><th>Category</th><th>Ingredients</th><th>Status</th><th>Actions</th></tr></thead>
                    <tbody id="admin-dish-table-body">{rows}</tbody>
                </table>
            </div>
            <p class="muted" id="admin-dish-empty" hidden>No matching dishes</p>
        </section>
        "#
    )
}

fn recommendation_tester(admin: &AdminView) -> String {
    let dishes = &admin.dishes;
    let liked_options =
        experiment_option_list(&admin.preference_options.ingredients, "ingredient-liked");
    let disliked_options =
        experiment_option_list(&admin.preference_options.ingredients, "ingredient-disliked");
    let method_liked_options =
        experiment_option_list(&admin.preference_options.ingredients, "method-liked");
    let method_disliked_options =
        experiment_option_list(&admin.preference_options.ingredients, "method-disliked");

    format!(
        r#"
        <section class="admin-card experiment-lab">
            <div class="section-heading"><h2>Recommendation Experiment Lab</h2><p>Run controlled tests without changing production orders or recommendation weights.</p></div>
            <details class="experiment-guide">
                <summary>How to Use the Experiment Lab</summary>
                <div class="experiment-guide-grid">
                    <div><h3>1. Ingredient Impact</h3><p>Select liked or disliked ingredients, choose Top-K, and compare neutral and preference-shaped rankings.</p></div>
                    <div><h3>2. Co-Order Impact</h3><p>Choose two different dishes and add temporary co-orders to observe changes in collaborative evidence and rank.</p></div>
                    <div><h3>3. Method Comparison</h3><p>Choose a historical order, hide one dish, and compare whether each recommendation method recovers it.</p></div>
                </div>
                <p class="muted">All simulations are temporary. They do not modify data/orders.csv or production recommendation weights.</p>
            </details>
            <div class="experiment-tabs" role="tablist">
                <button id="experiment-tab-ingredient" class="chip active" type="button" role="tab" data-experiment-tab="ingredient" aria-controls="experiment-panel-ingredient" aria-selected="true">Ingredient Impact</button>
                <button id="experiment-tab-coorder" class="chip" type="button" role="tab" data-experiment-tab="coorder" aria-controls="experiment-panel-coorder" aria-selected="false">Co-Order Impact</button>
                <button id="experiment-tab-method" class="chip" type="button" role="tab" data-experiment-tab="method" aria-controls="experiment-panel-method" aria-selected="false">Method Comparison</button>
            </div>

            <section id="experiment-panel-ingredient" class="experiment-panel" role="tabpanel" aria-labelledby="experiment-tab-ingredient" data-experiment-panel="ingredient">
                <h3>Step 1: Choose preferences</h3>
                <p class="muted">Compare a neutral baseline with a ranking shaped by liked and disliked ingredients.</p>
                <div class="experiment-help"><strong>What this experiment demonstrates</strong><p>Liked ingredients can raise compatible dishes, while disliked ingredients exclude conflicting dishes.</p><strong>How to run it</strong><p>Select preferences, choose Top-K, then press Run Ingredient Experiment. Clear Result keeps selections; Reset removes them.</p></div>
                <div class="form-actions">
                    <button class="ghost-action" type="button" data-ingredient-preset="none">No preferences</button>
                    <button class="ghost-action" type="button" data-ingredient-preset="example">Example preferences</button>
                    <button class="ghost-action" type="button" data-ingredient-preset="all">All ingredients</button>
                </div>
                <div class="experiment-option-columns">
                    <div><h4>Liked ingredients</h4><input class="experiment-option-search" type="search" placeholder="Find a liked ingredient" aria-label="Search liked ingredients" data-experiment-option-search="ingredient-liked"><div class="ingredient-option-list" data-experiment-option-list="ingredient-liked">{liked_options}</div></div>
                    <div><h4>Disliked ingredients</h4><input class="experiment-option-search" type="search" placeholder="Find a disliked ingredient" aria-label="Search disliked ingredients" data-experiment-option-search="ingredient-disliked"><div class="ingredient-option-list" data-experiment-option-list="ingredient-disliked">{disliked_options}</div></div>
                </div>
                <label class="field-label compact-control">Top-K {top_k_ingredient}</label>
                <div class="form-actions">
                    <button class="primary-action" type="button" data-run-experiment="ingredient">Run Ingredient Experiment</button>
                    <button class="ghost-action" type="button" data-reset-experiment="ingredient">Reset</button>
                    <button class="ghost-action" type="button" data-clear-experiment="ingredient">Clear Result</button>
                </div>
                <div class="experiment-result" id="experiment-result-ingredient" aria-live="polite"></div>
            </section>

            <section id="experiment-panel-coorder" class="experiment-panel" role="tabpanel" aria-labelledby="experiment-tab-coorder" data-experiment-panel="coorder" hidden>
                <h3>Co-Order Impact</h3>
                <p class="muted">Add temporary pair baskets and compare association evidence before and after.</p>
                <div class="experiment-help"><strong>What this experiment demonstrates</strong><p>Repeated dish combinations strengthen collaborative evidence such as support, confidence, and lift.</p><strong>How to run it</strong><p>Choose different anchor and candidate dishes, enter temporary co-orders, choose Top-K, and press Run Co-Order Experiment.</p></div>
                <div class="admin-form compact-form">
                    <label class="field-label">Anchor dish<select id="coorder-anchor-dish">{select_options}</select></label>
                    <label class="field-label">Candidate dish<select id="coorder-candidate-dish">{select_options}</select></label>
                    <label class="field-label">Additional simulated co-orders<input id="coorder-additional" type="number" min="0" max="200" value="10"></label>
                    <label class="field-label">Top-K {top_k_coorder}</label>
                </div>
                <div class="form-actions">
                    <button class="primary-action" type="button" data-run-experiment="coorder">Run Co-Order Experiment</button>
                    <button class="ghost-action" type="button" data-reset-experiment="coorder">Reset</button>
                    <button class="ghost-action" type="button" data-clear-experiment="coorder">Clear Result</button>
                </div>
                <div class="experiment-result" id="experiment-result-coorder" aria-live="polite"></div>
            </section>

            <section id="experiment-panel-method" class="experiment-panel" role="tabpanel" aria-labelledby="experiment-tab-method" data-experiment-panel="method" hidden>
                <h3>Method Comparison</h3>
                <p class="muted">Hide one dish from a historical basket and test whether each method recovers it.</p>
                <div class="experiment-help"><strong>What this experiment demonstrates</strong><p>Ingredient-only, co-order-only, and controlled hybrid methods can recover the same hidden dish differently.</p><strong>How to run it</strong><p>Select an order and hidden target, optionally choose preferences, choose Top-K, and press Run Method Comparison.</p></div>
                <div class="admin-form compact-form">
                    <label class="field-label">Historical order<select id="method-historical-order">{historical_order_options}</select></label>
                    <label class="field-label">Hidden target dish<select id="method-hidden-dish" disabled><option value="">Select an order first</option></select></label>
                    <label class="field-label">Top-K {top_k_method}</label>
                </div>
                <div class="experiment-option-columns">
                    <div><h4>Liked ingredients</h4><input class="experiment-option-search" type="search" placeholder="Find a liked ingredient" aria-label="Search method liked ingredients" data-experiment-option-search="method-liked"><div class="ingredient-option-list" data-experiment-option-list="method-liked">{method_liked_options}</div></div>
                    <div><h4>Disliked ingredients</h4><input class="experiment-option-search" type="search" placeholder="Find a disliked ingredient" aria-label="Search method disliked ingredients" data-experiment-option-search="method-disliked"><div class="ingredient-option-list" data-experiment-option-list="method-disliked">{method_disliked_options}</div></div>
                </div>
                <div class="form-actions">
                    <button class="primary-action" type="button" data-run-experiment="method">Run Method Comparison</button>
                    <button class="ghost-action" type="button" data-reset-experiment="method">Reset</button>
                    <button class="ghost-action" type="button" data-clear-experiment="method">Clear Result</button>
                </div>
                <div class="experiment-result" id="experiment-result-method" aria-live="polite"></div>
            </section>

            <div class="reason-box"><strong>Controlled testing only</strong><p>Customer scoring remains unchanged. Co-order simulations use cloned orders in memory and never write to data/orders.csv.</p></div>
        </section>
        "#,
        select_options = dish_options_for_select(dishes),
        historical_order_options = historical_order_options_for_select(&admin.historical_orders),
        top_k_ingredient = top_k_select("ingredient-top-k"),
        top_k_coorder = top_k_select("coorder-top-k"),
        top_k_method = top_k_select("method-top-k"),
    )
}

fn experiment_option_list(values: &[String], kind: &str) -> String {
    let mut values = values.to_vec();
    values.sort_by_key(|value| value.to_lowercase());
    values.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    values
        .into_iter()
        .map(|value| {
            format!(
                r#"<label class="ingredient-option" data-option-label="{}"><input type="checkbox" name="{kind}" data-experiment-option="{kind}" value="{}"><span>{}</span></label>"#,
                escape_attr(&value.to_lowercase()),
                escape_attr(&value),
                escape_html(&display_option_label(&value))
            )
        })
        .collect()
}

fn display_option_label(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn top_k_select(id: &str) -> String {
    format!(
        r#"<select id="{}"><option value="3">Top 3</option><option value="5" selected>Top 5</option><option value="10">Top 10</option></select>"#,
        escape_attr(id)
    )
}

fn dish_options_for_select(dishes: &[DishView]) -> String {
    let mut options = r#"<option value="">Select dish</option>"#.to_string();
    options.push_str(
        &dishes
            .iter()
            .map(|dish| {
                format!(
                    r#"<option value="{}">{} ({})</option>"#,
                    escape_attr(&dish.dish_id),
                    escape_html(&dish.name),
                    escape_html(&dish.dish_id)
                )
            })
            .collect::<String>(),
    );
    options
}

fn historical_order_options_for_select(orders: &[Order]) -> String {
    let mut options = r#"<option value="">Select historical order</option>"#.to_string();
    options.push_str(
        &orders
            .iter()
            .filter(|order| order.ordered_dishes.len() >= 2)
            .map(|order| {
                format!(
                    r#"<option value="{}" data-dish-ids="{}">{} · {}</option>"#,
                    escape_attr(&order.order_id),
                    escape_attr(&order.ordered_dishes.join(",")),
                    escape_html(&order.order_id),
                    escape_html(&order.ordered_dishes.join(", "))
                )
            })
            .collect::<String>(),
    );
    options
}

#[allow(dead_code)]
fn csv_data_tools_panel() -> String {
    r#"
        <section class="admin-card">
            <div class="section-heading">
                <div>
                    <h2>CSV Import / Reload / Export</h2>
                    <p>Use compatible dishes.csv and orders.csv files. Preview validates required columns before import.</p>
                </div>
            </div>
            <p class="status-message" id="csv-import-status"></p>

            <div class="admin-two-column">
                <div class="data-tool-card">
                    <h3>Dishes CSV</h3>
                    <p class="muted">Required columns: dish_id, name, ingredients, category, tags.</p>
                    <input id="dish-csv-file" type="file" accept=".csv,text/csv">
                    <textarea id="dish-csv-import" rows="7" placeholder="Upload a CSV file or paste dishes.csv content here"></textarea>
                    <div class="csv-mode-row">
                        <label><input type="radio" name="dish-import-mode" value="replace" checked> Replace current dishes</label>
                        <label><input type="radio" name="dish-import-mode" value="merge"> Merge by dish ID</label>
                    </div>
                    <div class="form-actions">
                        <button class="primary-action" id="import-dishes-button" type="button">Import Dishes</button>
                        <button class="ghost-action" id="reload-dishes-button" type="button">Reload data/dishes.csv</button>
                        <a class="ghost-link" href="/admin/export/dishes.csv">Export Dishes</a>
                    </div>
                    <div class="csv-preview" id="dish-csv-preview"></div>
                </div>

                <div class="data-tool-card">
                    <h3>Orders CSV</h3>
                    <p class="muted">Required columns: order_id, session_user_id, ordered_dishes, timestamp.</p>
                    <input id="order-csv-file" type="file" accept=".csv,text/csv">
                    <textarea id="order-csv-import" rows="7" placeholder="Upload a CSV file or paste orders.csv content here"></textarea>
                    <div class="form-actions">
                        <button class="primary-action" id="import-orders-button" type="button">Import Orders</button>
                        <button class="ghost-action" id="reload-orders-button" type="button">Reload data/orders.csv</button>
                        <a class="ghost-link" href="/admin/export/orders.csv">Export Historical Orders</a>
                        <a class="ghost-link" href="/admin/export/completed-session-orders.csv">Export Completed Session Orders</a>
                    </div>
                    <div class="csv-preview" id="order-csv-preview"></div>
                </div>
            </div>
        </section>
    "#
    .to_string()
}

fn bottom_nav(active: &str) -> String {
    let items = [
        ("home", "Home", "/", "home-icon"),
        ("profile", "Profile", "/profile", "profile-icon"),
        ("cart", "Cart", "/cart", "cart-icon"),
        ("admin", "Admin Login", "/admin/login", "lock-icon"),
    ];

    let links = items
        .iter()
        .map(|(id, label, href, icon)| {
            let cart_count = if *id == "cart" {
                r#"<span class="cart-count" data-cart-count>0</span>"#
            } else {
                ""
            };
            format!(
                r#"<a class="nav-item{}" href="{}"><span class="nav-css-icon {}"></span><strong>{}</strong>{}</a>"#,
                if *id == active { " active" } else { "" },
                href,
                icon,
                label,
                cart_count
            )
        })
        .collect::<String>();

    format!(r#"<nav class="bottom-nav">{links}</nav>"#)
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_attr(value: &str) -> String {
    escape_html(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Dish;
    use crate::web::state::WebState;

    fn test_dish(id: &str, name: &str) -> Dish {
        Dish {
            dish_id: id.to_string(),
            name: name.to_string(),
            ingredients: vec!["rice".to_string()],
            category: "main".to_string(),
            tags: vec!["local".to_string()],
            image_path: None,
            image_source_url: None,
        }
    }

    #[test]
    fn customer_page_renders_permanent_menu_cards_and_count() {
        let state = WebState::new(
            vec![
                test_dish("D01", "Nasi Kandar"),
                test_dish("D02", "Nasi Kerabu"),
                test_dish("D03", "Chicken Satay"),
            ],
            Vec::new(),
        );
        let session = CustomerSession {
            session_id: "S001".to_string(),
            customer_name: "Tester".to_string(),
            customer_phone: "0123456789".to_string(),
            table_number: "T01".to_string(),
        };

        let html = customer_menu_page(&state.menu_view(), &session);

        assert_eq!(html.matches("class=\"dish-card\"").count(), 3);
        assert!(html.contains("<span id=\"visible-count\">3</span> dish(es) available"));
        assert!(html.contains("id=\"dish-D01\""));
        assert!(!html.contains("category-strip"));
        assert!(html.contains("Preston's Restaurant"));
        assert!(!html.contains("QR Restaurant Ordering"));
        assert!(!html.contains("data-feedback-dish"));
    }

    #[test]
    fn dashboard_contains_insights_without_insights_navigation() {
        let state = WebState::new(vec![test_dish("D01", "Nasi Lemak")], Vec::new());
        let html = admin_page(&state.menu_view(), &state.admin_view());

        assert!(html.contains("Smart Menu Insights"));
        assert!(!html.contains(r#">Insights</a>"#));
        assert!(html.contains("Open Recommendation Experiment Lab"));
        assert_eq!(html.matches("class=\"dashboard-stat-card\"").count(), 6);
        assert_eq!(html.matches("class=\"dashboard-stat-value\"").count(), 6);
    }

    #[test]
    fn experiment_lab_renders_three_distinct_accessible_panels() {
        let mut nasi_lemak = test_dish("D01", "Nasi Lemak");
        nasi_lemak.ingredients.push("coconut milk".to_string());
        let state = WebState::new(
            vec![nasi_lemak, test_dish("D02", "Chicken Satay")],
            Vec::new(),
        );
        let html = admin_recommendations_page(&state.menu_view(), &state.admin_view());

        for name in ["ingredient", "coorder", "method"] {
            assert!(html.contains(&format!("data-experiment-tab=\"{name}\"")));
            assert!(html.contains(&format!("data-experiment-panel=\"{name}\"")));
            assert!(html.contains(&format!("data-run-experiment=\"{name}\"")));
            assert!(html.contains(&format!("data-clear-experiment=\"{name}\"")));
        }
        assert!(!html.contains("admin-context-search"));
        assert!(html.contains("How to Use the Experiment Lab"));
        assert!(html.contains("What this experiment demonstrates"));
        assert!(html.contains("How to run it"));
        assert!(html.contains(
            r#"<label class="ingredient-option" data-option-label="rice"><input type="checkbox"#
        ));
        assert!(html.contains("data-experiment-option-search=\"ingredient-liked\""));
        assert!(html.contains("<span>Coconut Milk</span>"));
        assert!(!html.contains("data-feedback-dish"));
    }

    #[test]
    fn standalone_experiment_manual_documents_all_three_workflows() {
        let manual = include_str!("../../docs/recommendation-experiment-lab-manual.md");

        assert!(manual.contains("Experiment 1: Ingredient Impact"));
        assert!(manual.contains("Experiment 2: Co-Order Impact"));
        assert!(manual.contains("Experiment 3: Method Comparison"));
        assert!(manual.contains("Run Ingredient Experiment"));
        assert!(manual.contains("Run Co-Order Experiment"));
        assert!(manual.contains("Run Method Comparison"));
        assert!(manual.contains("does not modify the real `data/orders.csv`"));
    }
}
