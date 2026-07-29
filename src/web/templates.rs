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
    let required_category_options = view
        .dishes
        .iter()
        .map(|dish| dish.category.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|category| {
            format!(
                r#"<label class="category-check"><input type="checkbox" value="{}" data-meal-category> {}</label>"#,
                escape_attr(&category),
                escape_html(&category)
            )
        })
        .collect::<String>();
    let meal_context_options = view
        .dishes
        .iter()
        .map(|dish| {
            format!(
                r#"<label class="category-check"><input type="checkbox" value="{}" data-meal-context> {} ({})</label>"#,
                escape_attr(&dish.dish_id),
                escape_html(&dish.name),
                escape_html(&dish.dish_id)
            )
        })
        .collect::<String>();
    let content = format!(
        r#"
        <section class="search-panel unified-search-panel" aria-label="Search menu and preferences">
            <p class="eyebrow">Preston's Restaurant</p>
            <h1>Welcome, {customer_name}</h1>
            <label class="search-box">
                <span class="search-icon">{search_icon}</span>
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
                    <button class="carousel-arrow" type="button" data-carousel-scroll="recommended-row" data-direction="-1" aria-label="Scroll recommendations left">{chevron_left}</button>
                    <button class="carousel-arrow" type="button" data-carousel-scroll="recommended-row" data-direction="1" aria-label="Scroll recommendations right">{chevron_right}</button>
                </div>
            </div>
            <div class="recommended-row" id="recommended-row" data-drag-scroll aria-label="Recommended dishes. Swipe or drag horizontally to see more.">
                {recommended_cards}
            </div>
            <div class="evaluation-strip" id="recommendation-stats"></div>
        </section>

        <section class="meal-set-builder" aria-labelledby="meal-set-title">
            <div class="section-heading">
                <div><h2 id="meal-set-title">Build a Meal Set</h2><p>Create a balanced selection that fits your budget and preferences.</p></div>
            </div>
            <div class="meal-step">
                <div class="meal-step-heading"><span>1</span><div><h3>Your table</h3><p>Set the spending limit and serving size.</p></div></div>
                <div class="meal-set-controls">
                    <label>Budget (RM)<input id="meal-budget" data-meal-control type="number" min="1" step="1" value="60" inputmode="numeric"></label>
                    <label>People<input id="meal-party-size" data-meal-control type="number" min="1" max="12" value="2" inputmode="numeric"></label>
                    <label>Dishes (optional)<input id="meal-target-count" data-meal-control type="number" min="1" max="8" placeholder="Auto" inputmode="numeric"></label>
                    <label>Results<select id="meal-set-count" data-meal-control><option>1</option><option selected>3</option><option>5</option></select></label>
                </div>
            </div>
            <div class="meal-step">
                <div class="meal-step-heading"><span>2</span><div><h3>Preferences</h3><p>Likes, dislikes and tags come from Personalise Recommendations above.</p></div></div>
                <details class="meal-category-options">
                    <summary>Required categories</summary>
                    <div class="category-check-row">{required_category_options}</div>
                </details>
                <details class="meal-category-options">
                    <summary>Current dishes used as recommendation context</summary>
                    <p class="muted">These choices guide co-ordering only. Clearing them never removes items from your Cart.</p>
                    <div class="category-check-row meal-context-options">{meal_context_options}</div>
                </details>
            </div>
            <div class="meal-step">
                <div class="meal-step-heading"><span>3</span><div><h3>Style</h3><p id="diversity-description">Balanced combines familiar matches with some variety.</p></div></div>
                <div class="diversity-selector" aria-label="Recommendation variety">
                    <button type="button" data-meal-control data-diversity-mode="familiar" aria-pressed="false">Familiar</button>
                    <button class="active" type="button" data-meal-control data-diversity-mode="balanced" aria-pressed="true">Balanced</button>
                    <button type="button" data-meal-control data-diversity-mode="discover" aria-pressed="false">Discover</button>
                </div>
            </div>
            <div class="meal-action-row">
                <button class="primary-action" id="build-meal-set" data-meal-control type="button">Generate Meal Set</button>
                <button class="ghost-action" id="clear-meal-choices" data-meal-control type="button">Clear Choices</button>
                <button class="ghost-action" id="clear-meal-result" data-meal-control type="button">{clear_icon}Clear Result</button>
            </div>
            <p class="status-message" id="meal-set-status" aria-live="polite"></p>
            <div class="meal-set-results" id="meal-set-results" aria-live="polite">
                <div class="empty-state"><strong>No meal set has been generated yet.</strong><span>Choose your table settings and generate a meal set when ready.</span></div>
            </div>
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
        dish_modal = dish_detail_modal(),
        required_category_options = required_category_options,
        meal_context_options = meal_context_options,
        search_icon = icon_svg("search"),
        chevron_left = icon_svg("chevron-left"),
        chevron_right = icon_svg("chevron-right"),
        clear_icon = icon_svg("x")
    );

    page_shell(CustomerPageShell {
        title: "Home",
        active_nav: "home",
        content: &content,
        dishes_json: &view.dishes_json,
        recommendations_json: &view.recommendations_json,
        preference_options_json: &view.preference_options_json,
        search_vocabulary_json: &view.search_vocabulary_json,
        customer_session: Some(session),
    })
}

/// Renders the prototype cart page.
pub fn cart_page(view: &MenuView, session: &CustomerSession) -> String {
    let content = format!(
        r#"
        <section class="plain-page">
            <p class="eyebrow">Preston's Restaurant</p>
            <h1 class="page-title-with-icon">{cart_icon} Cart</h1>
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
                <div class="cart-summary__metrics">
                    <div class="cart-summary__row"><span>Unique dishes</span><strong class="cart-summary__value" id="cart-unique-count">0</strong></div>
                    <div class="cart-summary__row"><span>Total portions</span><strong class="cart-summary__value" id="cart-portions-count">0</strong></div>
                    <div class="cart-summary__row total-row"><span>Subtotal</span><strong class="cart-summary__value" id="cart-page-total">RM0.00</strong></div>
                </div>
                <button class="primary-action" id="checkout-button" type="button">{checkout_icon} Place Order</button>
            </div>
            <p class="muted">FYP prototype: no payment is processed.</p>
            <p class="status-message" id="checkout-status" aria-live="polite"></p>
        </section>
    "#,
        name = escape_html(&session.customer_name),
        table = escape_html(&session.table_number),
        phone = escape_html(&session.masked_phone()),
        cart_icon = icon_svg("shopping-cart"),
        checkout_icon = icon_svg("circle-check")
    );

    page_shell(CustomerPageShell {
        title: "Cart",
        active_nav: "cart",
        content: &content,
        dishes_json: &view.dishes_json,
        recommendations_json: &view.recommendations_json,
        preference_options_json: &view.preference_options_json,
        search_vocabulary_json: &view.search_vocabulary_json,
        customer_session: Some(session),
    })
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
    <link rel="stylesheet" href="/static/app.css?v=20260729-drag-scroll">
    <script src="/static/auth.js?v=20260724" defer></script>
</head>
<body>
    <main class="app-shell start-shell">
        <section class="admin-card start-card">
            <p class="eyebrow">Preston's Restaurant</p>
            <h1 class="page-title-with-icon">{user_icon} Start Your Dining Session</h1>
            <p class="muted">Enter basic details once. Checkout and order tracking will use this temporary session.</p>
            {message}
            <form class="admin-form" method="post" action="/start" data-auth-form>
                <label class="field-label">Customer name<input name="customer_name" value="{name}" placeholder="Customer name" autocomplete="name" required></label>
                <label class="field-label">Phone number<input name="customer_phone" value="{phone}" placeholder="e.g. 0123456789" autocomplete="tel" inputmode="tel" required></label>
                <label class="field-label">Table number<input name="table_number" value="{table}" placeholder="e.g. T05" required></label>
                <button class="primary-action" type="submit" data-submitting-label="Continuing...">Enter Menu {enter_icon}</button>
            </form>
            <p class="privacy-note">Your details are used only for this dining session, order fulfilment, and order-status communication.</p>
            <a class="ghost-link" href="/admin/login">Staff Admin Login</a>
        </section>
    </main>
</body>
</html>"#,
        user_icon = icon_svg("user"),
        enter_icon = icon_svg("chevron-right")
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
            <section class="order-filter-row" data-drag-scroll aria-label="Order status filters. Swipe or drag horizontally to see more.">
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

    page_shell(CustomerPageShell {
        title: "Profile",
        active_nav: "profile",
        content: &content,
        dishes_json: &view.dishes_json,
        recommendations_json: &view.recommendations_json,
        preference_options_json: &view.preference_options_json,
        search_vocabulary_json: &view.search_vocabulary_json,
        customer_session: Some(session),
    })
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
    <link rel="stylesheet" href="/static/app.css?v=20260729-drag-scroll">
    <script src="/static/auth.js?v=20260724" defer></script>
</head>
<body>
    <main class="app-shell admin-login-shell">
        <section class="admin-card login-card">
            <p class="eyebrow">Staff Access</p>
            <h1 class="page-title-with-icon">{lock_icon} Admin Login</h1>
            <p class="muted">Prototype staff area for orders, dishes, recommendation testing, and menu insights.</p>
            {message}
            <form class="admin-form" method="post" action="/admin/login" data-auth-form>
                <input name="username" value="{username}" placeholder="Username" autocomplete="username" required>
                <input name="password" type="password" placeholder="Password" autocomplete="current-password" required>
                <button class="primary-action" type="submit" data-submitting-label="Logging in...">{lock_icon} Log In</button>
            </form>
            <a class="ghost-link" href="/">{home_icon} Go to Customer Menu</a>
        </section>
    </main>
</body>
</html>"#,
        lock_icon = icon_svg("lock-keyhole"),
        home_icon = icon_svg("home")
    )
}

struct CustomerPageShell<'a> {
    title: &'a str,
    active_nav: &'a str,
    content: &'a str,
    dishes_json: &'a str,
    recommendations_json: &'a str,
    preference_options_json: &'a str,
    search_vocabulary_json: &'a str,
    customer_session: Option<&'a CustomerSession>,
}

fn page_shell(page: CustomerPageShell<'_>) -> String {
    let customer_json = page
        .customer_session
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
    <link rel="stylesheet" href="/static/app.css?v=20260729-drag-scroll">
    <script>
        // Successful registration can render this menu directly so embedded
        // mobile previews do not depend on a redirect-cookie round trip.
        if (window.location.pathname === "/start") {{
            window.history.replaceState(null, "", "/");
        }}
        window.MENU_DISHES = {};
        window.RECOMMENDATIONS = {};
        window.PREFERENCE_OPTIONS = {};
        window.SEARCH_VOCABULARY = {};
        window.CUSTOMER_SESSION = {};
    </script>
    <script src="/static/app.js?v=20260729-drag-scroll" defer></script>
</head>
<body>
    <main class="app-shell">
        {}
    </main>
    {}
</body>
</html>"#,
        escape_html(page.title),
        page.dishes_json,
        page.recommendations_json,
        page.preference_options_json,
        page.search_vocabulary_json,
        customer_json,
        page.content,
        bottom_nav(page.active_nav)
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
    <link rel="stylesheet" href="/static/app.css?v=20260729-drag-scroll">
    <script>
        window.MENU_DISHES = {};
        window.RECOMMENDATIONS = {};
        window.PREFERENCE_OPTIONS = {};
        window.SEARCH_VOCABULARY = {};
    </script>
    <script src="/static/app.js?v=20260729-drag-scroll" defer></script>
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
        ("dashboard", "Dashboard", "/admin", "layout-dashboard"),
        ("orders", "Orders", "/admin/orders", "clipboard-list"),
        ("dishes", "Dishes", "/admin/dishes", "utensils"),
        (
            "recommendations",
            "Recommendation Tester",
            "/admin/recommendations",
            "flask-conical",
        ),
    ];
    let items = links
        .iter()
        .map(|(id, label, href, icon)| {
            format!(
                r#"<a class="admin-nav-link{}" href="{}">{}{}</a>"#,
                if *id == active { " active" } else { "" },
                escape_attr(href),
                icon_svg(icon),
                escape_html(label)
            )
        })
        .collect::<String>();

    let logout = format!(
        r#"<form action="/admin/logout" method="post"><button class="admin-nav-link" type="submit">{}Logout</button></form>"#,
        icon_svg("log-out")
    );

    format!(
        r#"<nav class="admin-section-nav" data-drag-scroll aria-label="Admin sections. Swipe or drag horizontally to see more.">{items}{logout}</nav>"#
    )
}

fn preference_group(title: &str, help: &str, kind: &str, values: &[String]) -> String {
    let chips = values
        .iter()
        .map(|value| {
            format!(
                r#"<button class="mini-chip" type="button" aria-pressed="false" data-preference-kind="{kind}" data-preference-value="{}">{}</button>"#,
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
                <div class="evidence-summary">
                    <span class="evidence-badge evidence-{confidence_class}">{confidence_label}</span>
                    <span class="evidence-meter" role="meter" aria-label="Recommendation evidence confidence" aria-valuemin="0" aria-valuemax="100" aria-valuenow="{confidence_percent}"><span style="width:{confidence_percent}%"></span></span>
                </div>
                <span class="reason">{reason}</span>
                <strong>{price}</strong>
                <div class="card-actions">
                    <button class="add-button" data-add-cart="{dish_id}" type="button">{add_icon} Add</button>
                    <button class="ghost-action" data-view-dish="{dish_id}" type="button">{info_icon} Why this?</button>
                </div>
            </div>
        </article>
        "#,
        image_block(dish, "compact"),
        dish_id = escape_attr(&dish.dish_id),
        name = escape_html(&dish.name),
        category = escape_html(&dish.category),
        confidence_class = confidence_class(recommendation),
        confidence_label = confidence_label(recommendation),
        confidence_percent = (recommendation.evidence.overall_confidence * 100.0).round() as u8,
        reason = escape_html(&short_recommendation_reason(recommendation)),
        price = escape_html(&dish.price),
        add_icon = icon_svg("plus"),
        info_icon = icon_svg("circle-info")
    )
}

fn confidence_class(recommendation: &RecommendationView) -> &'static str {
    match recommendation.evidence.confidence_level {
        crate::recommender::evidence::ConfidenceLevel::Insufficient => "insufficient",
        crate::recommender::evidence::ConfidenceLevel::Low => "low",
        crate::recommender::evidence::ConfidenceLevel::Medium => "medium",
        crate::recommender::evidence::ConfidenceLevel::High => "high",
    }
}

fn confidence_label(recommendation: &RecommendationView) -> &'static str {
    match recommendation.evidence.confidence_level {
        crate::recommender::evidence::ConfidenceLevel::Insufficient => "Limited evidence",
        crate::recommender::evidence::ConfidenceLevel::Low => "Low evidence",
        crate::recommender::evidence::ConfidenceLevel::Medium => "Medium evidence",
        crate::recommender::evidence::ConfidenceLevel::High => "High evidence",
    }
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
                    <button class="add-button" data-add-cart="{dish_id}" type="button">{add_icon} Add</button>
                </div>
                <button class="text-action button-with-icon" data-view-dish="{dish_id}" type="button">{view_icon} View details</button>
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
        price = escape_html(&dish.price),
        add_icon = icon_svg("plus"),
        view_icon = icon_svg("eye")
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
            r#"<div class="dish-art placeholder {extra_class}" aria-label="No image for {}">{}</div>"#,
            escape_attr(&dish.name),
            icon_svg("utensils")
        ),
    }
}

fn dish_detail_modal() -> String {
    format!(
        r#"
    <dialog class="dish-modal" id="dish-detail-modal" aria-labelledby="dish-detail-title">
        <div class="modal-content">
            <button class="modal-close icon-button" type="button" data-close-dish-modal aria-label="Close dish details">{}</button>
            <div id="dish-detail-content"></div>
        </div>
    </dialog>
    "#,
        icon_svg("x")
    )
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
                        <button class="ghost-action" data-edit-dish="{dish_id}" type="button">{edit_icon} Edit</button>
                        <button class="ghost-action" data-toggle-dish="{dish_id}" data-available="{next_available}" type="button">{availability_icon} {toggle_label}</button>
                        <button class="danger-action" data-delete-dish="{dish_id}" type="button">{delete_icon} Delete</button>
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
                edit_icon = icon_svg("pencil"),
                availability_icon = icon_svg("circle-check"),
                delete_icon = icon_svg("trash-2"),
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
                <button class="primary-action" id="open-dish-form" type="button">{add_icon} Add Dish</button>
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
        "#,
        add_icon = icon_svg("plus")
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
    let adaptive_ingredient_options =
        plain_select_options(&admin.preference_options.ingredients, false);
    let adaptive_tag_options = plain_select_options(&admin.preference_options.tags, false);
    let adaptive_dish_options = plain_select_options(
        &admin
            .dishes
            .iter()
            .map(|dish| format!("{}|{} ({})", dish.dish_id, dish.name, dish.dish_id))
            .collect::<Vec<_>>(),
        true,
    );

    format!(
        r#"
        <section class="tester-overview" id="recommendation-tester-overview" data-tester-overview>
            <div class="tester-overview-grid">
                <article class="tool-overview-card"><span>Production</span><h2>Production Recommendation</h2><p>Inspect the live adaptive recommender, confidence, diversity and meal sets.</p><button class="primary-action" type="button" data-open-tester-category="production">Open Production</button></article>
                <article class="tool-overview-card"><span>Research</span><h2>Controlled Experiments</h2><p>Run Ingredient Impact, Co-Order Impact and Method Comparison for FYP evaluation.</p><button class="primary-action" type="button" data-open-tester-category="experiments">Open Experiments</button></article>
                <article class="tool-overview-card"><span>Reasoning</span><h2>Explainability</h2><p>Compare baseline and counterfactual recommendation results without production writes.</p><button class="primary-action" type="button" data-open-tester-category="explainability">Open Explainability</button></article>
                <article class="tool-overview-card"><span>Evidence</span><h2>Learning History</h2><p>Review how completed orders changed popularity and co-order evidence.</p><button class="primary-action" type="button" data-open-tester-category="learning">Open Learning History</button></article>
            </div>
        </section>

        <div class="tester-shell" data-tester-shell hidden>
            <aside class="tool-category-nav" aria-label="Recommendation Tester categories">
                <button type="button" data-tester-home>Overview</button>
                <button type="button" data-tester-category="production" data-default-tool="adaptive">Production</button>
                <button type="button" data-tester-category="experiments" data-default-tool="ingredient-impact">Experiments</button>
                <button type="button" data-tester-category="explainability" data-default-tool="counterfactual">Explainability</button>
                <button type="button" data-tester-category="learning" data-default-tool="timeline">Learning History</button>
            </aside>
            <div class="tester-workspace">
                <label class="tester-mobile-category">Tool category
                    <select id="tester-category-select">
                        <option value="production">Production</option>
                        <option value="experiments">Experiments</option>
                        <option value="explainability">Explainability</option>
                        <option value="learning">Learning History</option>
                    </select>
                </label>
                <div class="tool-tabs" data-drag-scroll aria-label="Tools in selected category. Swipe or drag horizontally to see more.">
                    <button type="button" data-tester-tool="adaptive" data-tool-category="production" data-tool-target="production-adaptive">Adaptive Scoring</button>
                    <button type="button" data-tester-tool="confidence" data-tool-category="production" data-tool-target="production-adaptive">Confidence Meter</button>
                    <button type="button" data-tester-tool="diversity" data-tool-category="production" data-tool-target="production-adaptive">Diversity</button>
                    <button type="button" data-tester-tool="meal-sets" data-tool-category="production" data-tool-target="production-meal-sets">Meal Sets</button>
                    <button type="button" data-tester-tool="ingredient-impact" data-tool-category="experiments" data-tool-target="controlled-experiments" data-experiment-shortcut="ingredient">Ingredient Impact</button>
                    <button type="button" data-tester-tool="co-order-impact" data-tool-category="experiments" data-tool-target="controlled-experiments" data-experiment-shortcut="coorder">Co-Order Impact</button>
                    <button type="button" data-tester-tool="method-comparison" data-tool-category="experiments" data-tool-target="controlled-experiments" data-experiment-shortcut="method">Method Comparison</button>
                    <button type="button" data-tester-tool="counterfactual" data-tool-category="explainability" data-tool-target="explainability-counterfactual">What Would Change?</button>
                    <button type="button" data-tester-tool="evidence" data-tool-category="explainability" data-tool-target="explainability-counterfactual">Evidence Breakdown</button>
                    <button type="button" data-tester-tool="simulation" data-tool-category="explainability" data-tool-target="explainability-simulation">Co-Order Simulation</button>
                    <button type="button" data-tester-tool="timeline" data-tool-category="learning" data-tool-target="learning-timeline">Timeline</button>
                </div>

        <section id="tool-production-adaptive" class="admin-card adaptive-inspector tool-workspace-panel" data-tool-panel="production-adaptive" hidden>
            <div class="section-heading">
                <div><h2>Adaptive Scoring Inspector</h2><p>Inspect the data-aware adaptive weights used by the production customer recommender.</p></div>
                <span class="method-label">Data-aware adaptive weights</span>
            </div>
            <details class="experiment-guide">
                <summary>What Data-Aware Adaptive Recommendation Does</summary>
                <p>The system adjusts recommendation weights according to available evidence. With limited co-order data, explicit preferences and popularity receive more influence. As reliable co-order evidence grows, collaborative filtering receives more influence.</p>
                <h3>How to demonstrate it</h3>
                <ol><li>Select a rarely ordered context dish and observe low collaborative confidence.</li><li>Select a frequently ordered context dish and compare the weights.</li><li>Complete additional co-orders, rerun, and observe how evidence changes.</li></ol>
                <h3>What confidence means</h3>
                <p>The confidence meter represents the strength of evidence supporting a recommendation. It is not a prediction probability and does not guarantee customer satisfaction.</p>
                <p class="muted">The current prototype thresholds are shown in the inspector result and can be changed centrally in Rust.</p>
            </details>
            <div class="adaptive-input-grid">
                <label class="field-label">Liked ingredients<select id="adaptive-liked" multiple size="5">{adaptive_ingredient_options}</select></label>
                <label class="field-label">Disliked ingredients<select id="adaptive-disliked" multiple size="5">{adaptive_ingredient_options}</select></label>
                <label class="field-label">Preferred tags<select id="adaptive-tags" multiple size="5">{adaptive_tag_options}</select></label>
                <label class="field-label">Selected/context dishes<select id="adaptive-context" multiple size="5">{adaptive_dish_options}</select></label>
                <label class="field-label compact-control">Time context<select id="adaptive-time"><option>Any</option><option>Breakfast</option><option>Lunch</option><option>Dinner</option><option>Snack</option></select></label>
            </div>
            <div class="form-actions">
                <button class="primary-action" id="run-adaptive-inspector" type="button">{run_icon}Inspect Adaptive Scoring</button>
                <button class="ghost-action" id="reset-adaptive-inspector" type="button">{reset_icon}Reset</button>
            </div>
            <div id="adaptive-inspector-results" class="experiment-result" aria-live="polite"></div>
        </section>

        <section id="tool-production-meal-sets" class="admin-card tool-workspace-panel" data-tool-panel="production-meal-sets" hidden>
            <div class="section-heading">
                <div><h2>Budget Meal Set Tester</h2><p>Run the same bounded meal-set service used by the customer application.</p></div>
                <span class="method-label">Production pipeline</span>
            </div>
            <details class="experiment-guide"><summary>How to use this tool</summary><p>Enter a budget and party size, optionally select preferences and context dishes, then generate candidate sets. This request does not write orders or preferences.</p></details>
            <div class="admin-form compact-form">
                <label class="field-label">Budget (RM)<input id="admin-meal-budget" type="number" min="1" value="60"></label>
                <label class="field-label">Party size<input id="admin-meal-party" type="number" min="1" max="12" value="2"></label>
                <label class="field-label">Target dishes<input id="admin-meal-target" type="number" min="1" max="8" placeholder="Auto"></label>
                <label class="field-label">Diversity<select id="admin-meal-diversity"><option>familiar</option><option selected>balanced</option><option>discover</option></select></label>
                <label class="field-label">Liked ingredients<select id="admin-meal-liked" multiple size="5">{adaptive_ingredient_options}</select></label>
                <label class="field-label">Disliked ingredients<select id="admin-meal-disliked" multiple size="5">{adaptive_ingredient_options}</select></label>
                <label class="field-label">Preferred tags<select id="admin-meal-tags" multiple size="5">{adaptive_tag_options}</select></label>
                <label class="field-label">Context dishes<select id="admin-meal-context" multiple size="5">{adaptive_dish_options}</select></label>
            </div>
            <div class="form-actions">
                <button class="primary-action" id="run-admin-meal-set" type="button">{run_icon}Generate Meal Set</button>
                <button class="ghost-action" id="reset-admin-meal-set" type="button">{reset_icon}Reset Inputs</button>
                <button class="ghost-action" id="clear-admin-meal-result" type="button">{clear_icon}Clear Result</button>
            </div>
            <p class="status-message" id="admin-meal-status"></p>
            <div id="admin-meal-results" class="experiment-result" aria-live="polite"></div>
        </section>

        <section id="tool-explainability-counterfactual" class="admin-card counterfactual-explorer tool-workspace-panel" data-tool-panel="explainability-counterfactual" hidden>
            <div class="section-heading">
                <div><h2>What Would Change?</h2><p>Compare the exact production pipeline with one temporary alternative scenario.</p></div>
                <span class="method-label">No production writes</span>
            </div>
            <details class="experiment-guide">
                <summary>Counterfactual Explorer</summary>
                <p>Baseline and changed rankings use the same adaptive scoring, hard exclusions, and diversity reranker. Temporary co-orders remain in memory for this comparison only.</p>
            </details>
            <h3>Baseline</h3>
            <div class="adaptive-input-grid">
                <label class="field-label">Liked ingredients<select id="cf-base-liked" multiple size="4">{adaptive_ingredient_options}</select></label>
                <label class="field-label">Disliked ingredients<select id="cf-base-disliked" multiple size="4">{adaptive_ingredient_options}</select></label>
                <label class="field-label">Preferred tags<select id="cf-base-tags" multiple size="4">{adaptive_tag_options}</select></label>
                <label class="field-label">Context dishes<select id="cf-base-context" multiple size="4">{adaptive_dish_options}</select></label>
            </div>
            <h3>Temporary change</h3>
            <div class="adaptive-input-grid">
                <label class="field-label">Add liked<select id="cf-add-liked" multiple size="4">{adaptive_ingredient_options}</select></label>
                <label class="field-label">Remove liked<select id="cf-remove-liked" multiple size="4">{adaptive_ingredient_options}</select></label>
                <label class="field-label">Add disliked<select id="cf-add-disliked" multiple size="4">{adaptive_ingredient_options}</select></label>
                <label class="field-label">Remove disliked<select id="cf-remove-disliked" multiple size="4">{adaptive_ingredient_options}</select></label>
                <label class="field-label">Add tags<select id="cf-add-tags" multiple size="4">{adaptive_tag_options}</select></label>
                <label class="field-label">Remove tags<select id="cf-remove-tags" multiple size="4">{adaptive_tag_options}</select></label>
                <label class="field-label">Add context<select id="cf-add-context" multiple size="4">{adaptive_dish_options}</select></label>
                <label class="field-label">Remove context<select id="cf-remove-context" multiple size="4">{adaptive_dish_options}</select></label>
            </div>
            <div class="admin-form compact-form">
                <label class="field-label">Temporary co-order anchor<select id="cf-anchor">{select_options}</select></label>
                <label class="field-label">Temporary co-order candidate<select id="cf-candidate">{select_options}</select></label>
                <label class="field-label">Additional baskets<input id="cf-order-count" type="number" min="0" max="100" value="0"></label>
                <label class="field-label">Changed diversity mode<select id="cf-diversity"><option value="">Keep baseline</option><option value="familiar">Familiar</option><option value="balanced">Balanced</option><option value="discover">Discover</option></select></label>
                <label class="field-label">Top-K<select id="cf-top-k"><option>3</option><option selected>5</option><option>10</option></select></label>
            </div>
            <div class="form-actions">
                <button class="primary-action" id="run-counterfactual" type="button">{run_icon}Compare Scenarios</button>
                <button class="ghost-action" id="export-counterfactual" type="button" disabled>Export Comparison CSV</button>
            </div>
            <div id="counterfactual-results" class="experiment-result" aria-live="polite"></div>
        </section>

        <section id="tool-explainability-simulation" class="admin-card simulation-panel tool-workspace-panel" data-tool-panel="explainability-simulation" hidden>
            <div class="section-heading"><div><h2>Temporary Co-Order Simulation</h2><p>Compare ranking changes against generated in-memory baskets.</p></div><span class="method-label">No production writes</span></div>
            <details class="experiment-guide"><summary>How to use this tool</summary><p>Choose deterministic simulation settings and an optional forced pair. The generated baskets are never appended to orders.csv.</p></details>
            <div class="admin-form compact-form">
                <label class="field-label">Order count<input id="simulation-order-count" type="number" min="1" max="200" value="20"></label>
                <label class="field-label">Minimum dishes<input id="simulation-min-dishes" type="number" min="1" max="8" value="2"></label>
                <label class="field-label">Maximum dishes<input id="simulation-max-dishes" type="number" min="1" max="8" value="4"></label>
                <label class="field-label">Seed<input id="simulation-seed" type="number" value="42"></label>
                <label class="field-label">Popularity skew<select id="simulation-skew"><option value="uniform">Uniform</option><option value="moderate">Moderate</option><option value="strong">Strong</option></select></label>
                <label class="field-label">Forced pair A<select id="simulation-forced-a"></select></label>
                <label class="field-label">Forced pair B<select id="simulation-forced-b"></select></label>
                <label class="field-label">Pair probability (%)<input id="simulation-pair-probability" type="number" min="0" max="100" value="35"></label>
            </div>
            <div class="form-actions"><button class="primary-action" id="run-simulation" type="button">{run_icon}Run Simulation</button><button class="ghost-action" id="reset-simulation" type="button">{clear_icon}Clear Result</button></div>
            <div class="simulation-results" id="simulation-results" aria-live="polite"></div>
        </section>

        <section id="tool-learning-timeline" class="admin-card learning-timeline-panel tool-workspace-panel" data-tool-panel="learning-timeline" hidden>
            <div class="section-heading">
                <div><h2>How the Recommender Learned</h2><p>Factual evidence changes produced by real completed historical orders.</p></div>
            </div>
            <details class="experiment-guide"><summary>How to use this tool</summary><p>Filter explanatory events for presentation. Reset Filters is non-destructive. Clear Timeline removes only JSONL explanation records; Rebuild reconstructs them from historical orders.</p></details>
            <div class="timeline-filter-grid">
                <label class="field-label">Search<input id="timeline-search" type="search" placeholder="Order, dish, or summary"></label>
                <label class="field-label">Date<input id="timeline-date" type="date"></label>
                <label class="field-label">Dish<select id="timeline-dish"><option value="">All dishes</option>{select_options}</select></label>
                <label class="field-label">Sort<select id="timeline-sort"><option value="newest">Newest first</option><option value="oldest">Oldest first</option></select></label>
                <label class="field-label">Visible events<select id="timeline-limit"><option>10</option><option selected>25</option><option>50</option><option value="all">All</option></select></label>
            </div>
            <div class="form-actions">
                <button class="ghost-action" id="reset-timeline-filters" type="button">{reset_icon}Reset Filters</button>
                <button class="danger-action" id="clear-learning-timeline" type="button">{delete_icon}Clear Timeline</button>
                <button class="primary-action" id="rebuild-learning-timeline" type="button">{rebuild_icon}Rebuild Timeline</button>
            </div>
            <p class="status-message" id="learning-timeline-status"></p>
            <div id="learning-timeline" class="learning-timeline" aria-live="polite"></div>
        </section>

        <section id="tool-controlled-experiments" class="admin-card experiment-lab tool-workspace-panel" data-tool-panel="controlled-experiments" hidden>
            <div class="section-heading"><div><h2>Recommendation Experiment Lab</h2><p>Run controlled tests without changing production orders or recommendation weights.</p></div><span class="method-label">Controlled fixed weights for comparison</span></div>
            <details class="experiment-guide">
                <summary>How to Use the Experiment Lab</summary>
                <div class="experiment-guide-grid">
                    <div><h3>1. Ingredient Impact</h3><p>Select liked or disliked ingredients, choose Top-K, and compare neutral and preference-shaped rankings.</p></div>
                    <div><h3>2. Co-Order Impact</h3><p>Choose two different dishes and add temporary co-orders to observe changes in collaborative evidence and rank.</p></div>
                    <div><h3>3. Method Comparison</h3><p>Choose a historical order, hide one dish, and compare whether each recommendation method recovers it.</p></div>
                </div>
                <p class="muted">All simulations are temporary. They do not modify data/orders.csv or production recommendation weights.</p>
            </details>
            <div class="experiment-tabs" data-drag-scroll role="tablist" aria-label="Experiment type. Swipe or drag horizontally to see more.">
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
                    <button class="primary-action" type="button" data-run-experiment="ingredient">{run_icon}Run Ingredient Experiment</button>
                    <button class="ghost-action" type="button" data-reset-experiment="ingredient">{reset_icon}Reset</button>
                    <button class="ghost-action" type="button" data-clear-experiment="ingredient">{clear_icon}Clear Result</button>
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
                    <button class="primary-action" type="button" data-run-experiment="coorder">{run_icon}Run Co-Order Experiment</button>
                    <button class="ghost-action" type="button" data-reset-experiment="coorder">{reset_icon}Reset</button>
                    <button class="ghost-action" type="button" data-clear-experiment="coorder">{clear_icon}Clear Result</button>
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
                    <button class="primary-action" type="button" data-run-experiment="method">{run_icon}Run Method Comparison</button>
                    <button class="ghost-action" type="button" data-reset-experiment="method">{reset_icon}Reset</button>
                    <button class="ghost-action" type="button" data-clear-experiment="method">{clear_icon}Clear Result</button>
                </div>
                <div class="experiment-result" id="experiment-result-method" aria-live="polite"></div>
            </section>

            <div class="reason-box"><strong>Controlled testing only</strong><p>Production customer recommendations use adaptive weights. These experiments keep ingredient-only 1.0/0.0, co-order-only 0.0/1.0, and Hybrid 0.4/0.6 fixed. Simulations use cloned orders and never write to data/orders.csv.</p></div>
        </section>

            </div>
        </div>

        <dialog class="confirmation-dialog" id="timeline-confirm-dialog" aria-labelledby="timeline-confirm-title">
            <form method="dialog">
                <h2 id="timeline-confirm-title">Confirm timeline action</h2>
                <p id="timeline-confirm-message"></p>
                <div class="form-actions">
                    <button class="ghost-action" value="cancel" type="submit">Cancel</button>
                    <button class="danger-action" id="confirm-timeline-action" value="confirm" type="button">Confirm</button>
                </div>
            </form>
        </dialog>
        "#,
        adaptive_ingredient_options = adaptive_ingredient_options,
        adaptive_tag_options = adaptive_tag_options,
        adaptive_dish_options = adaptive_dish_options,
        select_options = dish_options_for_select(dishes),
        historical_order_options = historical_order_options_for_select(&admin.historical_orders),
        top_k_ingredient = top_k_select("ingredient-top-k"),
        top_k_coorder = top_k_select("coorder-top-k"),
        top_k_method = top_k_select("method-top-k"),
        run_icon = icon_svg("play"),
        reset_icon = icon_svg("rotate-ccw"),
        clear_icon = icon_svg("x"),
        delete_icon = icon_svg("trash-2"),
        rebuild_icon = icon_svg("refresh-cw"),
    )
}

fn plain_select_options(values: &[String], pipe_label: bool) -> String {
    values
        .iter()
        .map(|value| {
            let (option_value, label) = if pipe_label {
                value
                    .split_once('|')
                    .unwrap_or((value.as_str(), value.as_str()))
            } else {
                (value.as_str(), value.as_str())
            };
            format!(
                r#"<option value="{}">{}</option>"#,
                escape_attr(option_value),
                escape_html(&display_option_label(label))
            )
        })
        .collect()
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

/// Renders the shared inline Lucide icon subset.
///
/// Keeping the path data in one helper gives customer and admin templates the
/// same 24px view box, rounded strokes, and `currentColor` theming without
/// requiring an external icon CDN at runtime.
fn icon_svg(name: &str) -> String {
    let body = match name {
        "home" => {
            r#"<path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"/><polyline points="9 22 9 12 15 12 15 22"/>"#
        }
        "search" => r#"<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>"#,
        "user" => r#"<circle cx="12" cy="8" r="5"/><path d="M20 21a8 8 0 0 0-16 0"/>"#,
        "shopping-cart" => {
            r#"<circle cx="8" cy="21" r="1"/><circle cx="19" cy="21" r="1"/><path d="M2.05 2.05h2l2.66 12.42a2 2 0 0 0 2 1.58h7.72a2 2 0 0 0 2-1.61L20.05 7H5.12"/>"#
        }
        "lock-keyhole" => {
            r#"<rect width="18" height="11" x="3" y="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/><circle cx="12" cy="16" r="1"/>"#
        }
        "chevron-left" => r#"<path d="m15 18-6-6 6-6"/>"#,
        "chevron-right" => r#"<path d="m9 18 6-6-6-6"/>"#,
        "plus" => r#"<path d="M5 12h14"/><path d="M12 5v14"/>"#,
        "minus" => r#"<path d="M5 12h14"/>"#,
        "x" => r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
        "eye" => {
            r#"<path d="M2.06 12.35a1 1 0 0 1 0-.7 10.94 10.94 0 0 1 19.88 0 1 1 0 0 1 0 .7 10.94 10.94 0 0 1-19.88 0"/><circle cx="12" cy="12" r="3"/>"#
        }
        "utensils" => {
            r#"<path d="M3 2v7c0 1.1.9 2 2 2h4a2 2 0 0 0 2-2V2"/><path d="M7 2v20"/><path d="M21 15V2a5 5 0 0 0-5 5v6c0 1.1.9 2 2 2Z"/><path d="M18 22v-7"/>"#
        }
        "circle-check" => r#"<circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/>"#,
        "circle-info" => {
            r#"<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>"#
        }
        "layout-dashboard" => {
            r#"<rect width="7" height="9" x="3" y="3" rx="1"/><rect width="7" height="5" x="14" y="3" rx="1"/><rect width="7" height="9" x="14" y="12" rx="1"/><rect width="7" height="5" x="3" y="16" rx="1"/>"#
        }
        "clipboard-list" => {
            r#"<rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/><path d="M12 11h4"/><path d="M12 16h4"/><path d="M8 11h.01"/><path d="M8 16h.01"/>"#
        }
        "flask-conical" => {
            r#"<path d="M10 2v7.31"/><path d="M14 9.3V1.99"/><path d="M8.5 2h7"/><path d="M14 9.3 20.5 20a1 1 0 0 1-.85 1.5H4.35A1 1 0 0 1 3.5 20L10 9.3"/><path d="M6.5 16h11"/>"#
        }
        "log-out" => {
            r#"<path d="M10 17l5-5-5-5"/><path d="M15 12H3"/><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/>"#
        }
        "pencil" => {
            r#"<path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/>"#
        }
        "trash-2" => {
            r#"<path d="M3 6h18"/><path d="M8 6V4h8v2"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v5"/><path d="M14 11v5"/>"#
        }
        "rotate-ccw" => r#"<path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>"#,
        "refresh-cw" => {
            r#"<path d="M21 12a9 9 0 0 0-15-6.7L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 15 6.7L21 16"/><path d="M16 16h5v5"/>"#
        }
        "play" => r#"<polygon points="6 3 20 12 6 21 6 3"/>"#,
        "history" => {
            r#"<path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/><path d="M12 7v5l4 2"/>"#
        }
        _ => r#"<circle cx="12" cy="12" r="9"/>"#,
    };
    format!(
        r#"<svg class="icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">{body}</svg>"#
    )
}

fn bottom_nav(active: &str) -> String {
    let items = [
        ("home", "Home", "/", "home"),
        ("profile", "Profile", "/profile", "user"),
        ("cart", "Cart", "/cart", "shopping-cart"),
        ("admin", "Admin Login", "/admin/login", "lock-keyhole"),
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
                r#"<a class="nav-item{}" href="{}">{}<strong>{}</strong>{}</a>"#,
                if *id == active { " active" } else { "" },
                href,
                icon_svg(icon),
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
        assert!(html.contains("id=\"recommended-row\" data-drag-scroll"));
        assert!(html.contains("id=\"clear-meal-choices\""));
        assert!(html.contains("id=\"clear-meal-result\""));
        assert!(html.contains("data-meal-context"));
        assert!(!html.contains("category-strip"));
        assert!(html.contains("Preston's Restaurant"));
        assert!(!html.contains("QR Restaurant Ordering"));
        assert!(!html.contains("data-feedback-dish"));
    }

    #[test]
    fn cart_page_exposes_stable_grid_and_summary_hooks() {
        let state = WebState::new(vec![test_dish("D01", "Nasi Lemak")], Vec::new());
        let session = CustomerSession {
            session_id: "S001".to_string(),
            customer_name: "Tester".to_string(),
            customer_phone: "0123456789".to_string(),
            table_number: "T01".to_string(),
        };

        let html = cart_page(&state.menu_view(), &session);
        let script = include_str!("../../static/app.js");
        let styles = include_str!("../../static/app.css");

        for id in [
            "cart-page-items",
            "cart-unique-count",
            "cart-portions-count",
            "cart-page-total",
            "checkout-button",
        ] {
            assert_eq!(html.matches(&format!("id=\"{id}\"")).count(), 1);
        }
        assert!(script.contains("class=\"cart-item\""));
        assert!(script.contains("calculateCartTotals"));
        assert!(script.contains("formatCurrency"));
        assert!(script.contains("data-action=\"decrease-cart-quantity\""));
        assert!(script.contains("aria-label=\"Decrease"));
        assert!(script.contains("aria-label=\"Increase"));
        assert!(script.contains("aria-label=\"Remove"));
        assert!(styles.contains("grid-template-areas: \"image details quantity total remove\""));
        assert!(styles.contains("\"image details remove\""));
        assert!(styles.contains("\"image quantity quantity\""));
        assert!(styles.contains("\"image total total\""));
    }

    #[test]
    fn customer_and_admin_pages_use_shared_accessible_svg_icons() {
        let state = WebState::new(vec![test_dish("D01", "Nasi Lemak")], Vec::new());
        let session = CustomerSession {
            session_id: "S001".to_string(),
            customer_name: "Tester".to_string(),
            customer_phone: "0123456789".to_string(),
            table_number: "T01".to_string(),
        };
        let customer_html = customer_menu_page(&state.menu_view(), &session);
        let admin_html = admin_page(&state.menu_view(), &state.admin_view());

        for html in [&customer_html, &admin_html] {
            assert!(html.contains("<svg class=\"icon\""));
            assert!(html.contains("aria-hidden=\"true\""));
            assert!(!html.contains("nav-css-icon"));
            assert!(!html.contains('🍽'));
            assert!(!html.contains('⌕'));
        }
    }

    #[test]
    fn preference_chips_expose_toggle_state_to_assistive_technology() {
        let html = preference_group(
            "Liked Ingredients",
            "Ingredients the customer wants more often.",
            "liked_ingredients",
            &["chicken".to_string()],
        );

        assert!(html.contains("aria-pressed=\"false\""));
        assert!(html.contains("data-preference-kind=\"liked_ingredients\""));
    }

    #[test]
    fn admin_login_links_back_to_the_customer_menu() {
        let html = admin_login_page(None, None);

        assert!(html.contains(r#"<a class="ghost-link" href="/">"#));
        assert!(html.contains("Go to Customer Menu"));
        assert!(html.contains("<svg class=\"icon\""));
    }

    #[test]
    fn stylesheet_uses_catppuccin_latte_tokens_without_legacy_palette() {
        let styles = include_str!("../../static/app.css");

        for token in [
            "--ctp-base: #eff1f5",
            "--ctp-mantle: #e6e9ef",
            "--ctp-text: #4c4f69",
            "--ctp-maroon: #e64553",
            "--ctp-peach: #fe640b",
            "--ctp-green: #40a02b",
            "--ctp-lavender: #7287fd",
        ] {
            assert!(styles.contains(token), "missing theme token: {token}");
        }
        for legacy_value in ["#f6efe5", "#6b3f27", "#c66b3d", "255, 116, 23"] {
            assert!(
                !styles.to_lowercase().contains(legacy_value),
                "legacy palette value remains: {legacy_value}"
            );
        }
        assert!(styles.contains("background: var(--color-primary)"));
        assert!(styles.contains(".nav-item.active::after"));
        assert!(styles.contains(r#".responsive-data-table td[data-label="Actions"]"#));
        assert!(styles.contains("gap: 8px"));
        assert!(styles.contains("outline: 3px solid rgba(114, 135, 253, 0.65)"));
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
    fn recommendation_tester_renders_unique_category_and_workspace_navigation() {
        let state = WebState::new(
            vec![
                test_dish("D01", "Nasi Lemak"),
                test_dish("D02", "Chicken Satay"),
            ],
            Vec::new(),
        );
        let html = admin_recommendations_page(&state.menu_view(), &state.admin_view());

        for category in ["production", "experiments", "explainability", "learning"] {
            assert!(html.contains(&format!("data-tester-category=\"{category}\"")));
        }
        for panel in [
            "production-adaptive",
            "production-meal-sets",
            "controlled-experiments",
            "explainability-counterfactual",
            "explainability-simulation",
            "learning-timeline",
        ] {
            assert_eq!(
                html.matches(&format!("data-tool-panel=\"{panel}\""))
                    .count(),
                1
            );
        }
        assert!(html.contains("id=\"clear-learning-timeline\""));
        assert!(html.contains("id=\"timeline-confirm-dialog\""));
        assert!(html.contains("id=\"reset-timeline-filters\""));
    }

    #[test]
    fn recommendation_tester_guide_documents_all_nine_tools() {
        let guide = include_str!("../../docs/recommendation-tester-guide.md");

        for tool in [
            "Adaptive Scoring Inspector",
            "Confidence and Evidence Meter",
            "Diversity and Discovery",
            "Budget-Aware Meal Set Tester",
            "Ingredient Impact",
            "Co-Order Impact",
            "Method Comparison",
            "What Would Change?",
            "How the Recommender Learned",
        ] {
            assert!(guide.contains(tool));
        }
        assert!(guide.contains("Real history remains unchanged"));
        assert!(guide.contains("Unsafe conclusion"));
    }
}
