const CART_KEY = "fyp_web_cart_v1";
const LAST_ORDER_KEY = "fyp_last_order_id_v1";

function showToast(message) {
  let region = document.getElementById("app-toast-region");
  if (!region) {
    region = document.createElement("div");
    region.id = "app-toast-region";
    region.className = "toast-region";
    region.setAttribute("aria-live", "polite");
    document.body.appendChild(region);
  }
  const toast = document.createElement("div");
  toast.className = "toast";
  toast.textContent = message;
  region.appendChild(toast);
  window.setTimeout(() => {
    toast.classList.add("leaving");
    window.setTimeout(() => toast.remove(), 220);
  }, 2600);
}

function readCart() {
  try {
    return JSON.parse(localStorage.getItem(CART_KEY)) || {};
  } catch {
    return {};
  }
}

function writeCart(cart) {
  localStorage.setItem(CART_KEY, JSON.stringify(cart));
  updateCartCount();
  refreshCustomerRecommendations();
}

function readCustomerIdentity() {
  return window.CUSTOMER_SESSION || {};
}

function cartCount(cart = readCart()) {
  return Object.values(cart).reduce((sum, quantity) => sum + Number(quantity || 0), 0);
}

function updateCartCount() {
  document.querySelectorAll("[data-cart-count]").forEach((element) => {
    element.textContent = cartCount().toString();
  });
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

// All browser API calls use one response parser so network failures, non-2xx
// responses, and malformed JSON reach the feature's existing error UI instead
// of failing silently at different call sites.
async function requestJson(url, options = {}) {
  const response = await fetch(url, options);
  let payload;
  try {
    payload = await response.json();
  } catch {
    throw new Error("The server returned an unreadable response.");
  }
  if (!response.ok) {
    throw new Error(payload?.message || `Request failed with status ${response.status}.`);
  }
  return payload;
}

// Dynamic UI uses the same Lucide-compatible inline SVG subset as the Rust
// templates. Icons inherit `currentColor`, stay sharp at every viewport, and
// remain decorative while the surrounding control supplies its accessible
// label.
function iconSvg(name) {
  const paths = {
    plus: '<path d="M5 12h14"/><path d="M12 5v14"/>',
    minus: '<path d="M5 12h14"/>',
    "trash-2":
      '<path d="M3 6h18"/><path d="M8 6V4h8v2"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v5"/><path d="M14 11v5"/>',
    utensils:
      '<path d="M3 2v7c0 1.1.9 2 2 2h4a2 2 0 0 0 2-2V2"/><path d="M7 2v20"/><path d="M21 15V2a5 5 0 0 0-5 5v6c0 1.1.9 2 2 2Z"/><path d="M18 22v-7"/>',
    "circle-info":
      '<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>',
    "shopping-cart":
      '<circle cx="8" cy="21" r="1"/><circle cx="19" cy="21" r="1"/><path d="M2.05 2.05h2l2.66 12.42a2 2 0 0 0 2 1.58h7.72a2 2 0 0 0 2-1.61L20.05 7H5.12"/>',
  };
  return `<svg class="icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">${
    paths[name] || '<circle cx="12" cy="12" r="9"/>'
  }</svg>`;
}

// Formats every browser-rendered Malaysian Ringgit value consistently.
function formatCurrency(amount) {
  const value = Number(amount || 0);
  return `RM${(Number.isFinite(value) ? value : 0).toFixed(2)}`;
}

function formatList(values) {
  return values && values.length ? values.join(", ") : "-";
}

function evidenceLabel(level) {
  return {
    insufficient: "Limited evidence",
    low: "Low evidence",
    medium: "Medium evidence",
    high: "High evidence",
  }[level] || "Limited evidence";
}

function evidenceSourceLabel(source) {
  return {
    content_preference: "Content preference",
    co_ordering: "Co-ordering",
    popularity: "Popularity",
    time_context: "Time/context",
    mixed: "Mixed evidence",
    none: "Limited fallback evidence",
  }[source] || "Limited fallback evidence";
}

function adaptivePercentages(weights = {}) {
  const values = [
    Number(weights.content || 0),
    Number(weights.co_order || 0),
    Number(weights.popularity || 0),
    Number(weights.time_context || 0),
  ];
  const rounded = values.map((value) => Math.round(value * 100));
  const largest = values.indexOf(Math.max(...values));
  rounded[largest < 0 ? 0 : largest] += 100 - rounded.reduce((sum, value) => sum + value, 0);
  return rounded;
}

function evidenceSummaryHtml(recommendation) {
  const evidence = recommendation?.evidence || {};
  const level = evidence.confidence_level || "insufficient";
  const percent = Math.max(0, Math.min(100, Math.round(Number(evidence.overall_confidence || 0) * 100)));
  return `
    <div class="evidence-summary">
      <span class="evidence-badge evidence-${escapeHtml(level)}">${escapeHtml(evidenceLabel(level))}</span>
      <span class="evidence-meter" role="meter" aria-label="Recommendation evidence confidence" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${percent}">
        <span style="width:${percent}%"></span>
      </span>
    </div>
  `;
}

function dishById(dishId) {
  return (window.MENU_DISHES || []).find((dish) => dish.dish_id === dishId);
}

function recommendationByDishId(dishId) {
  return (window.RECOMMENDATIONS || []).find((item) => item.dish.dish_id === dishId);
}

function shortRecommendationReason(recommendation) {
  const parts = [];
  const liked = [...(recommendation.matched_liked_ingredients || [])];
  const tags = [...(recommendation.matched_preferred_tags || [])];
  if (liked.length || tags.length) {
    parts.push(`Matches ${formatList([...liked, ...tags].slice(0, 3))}`);
  }
  if ((recommendation.related_selected_dishes || []).length) {
    parts.push(`Often ordered with ${recommendation.related_selected_dishes[0]}`);
  }
  if (recommendation.popularity_score > 0 && !parts.length) {
    parts.push("Popular from order history");
  }
  if (recommendation.business_rule_score > 0 && parts.length < 2) {
    parts.push("Fits the menu context");
  }
  return parts.length ? parts.join(" · ") : "Based on ingredients and order patterns";
}

function parsePriceAmount(dish) {
  return Number(dish?.price_amount || 0);
}

function calculateCartTotals(cart = readCart()) {
  return Object.entries(cart).reduce(
    (totals, [dishId, rawQuantity]) => {
      const dish = dishById(dishId);
      const quantity = Math.max(0, Number(rawQuantity || 0));
      if (!dish || quantity === 0) return totals;
      totals.uniqueDishes += 1;
      totals.totalPortions += quantity;
      totals.subtotal += parsePriceAmount(dish) * quantity;
      return totals;
    },
    { uniqueDishes: 0, totalPortions: 0, subtotal: 0 }
  );
}

window.CartCalculations = { formatCurrency, calculateCartTotals };

function imageHtml(dish, extraClass = "") {
  if (dish?.image_url) {
    return `<div class="dish-art ${extraClass}"><img src="${escapeHtml(dish.image_url)}" alt="${escapeHtml(dish.name)}"></div>`;
  }
  return `<div class="dish-art placeholder ${extraClass}" aria-label="No image">${iconSvg(
    "utensils"
  )}</div>`;
}

function addToCart(dishId) {
  const cart = readCart();
  cart[dishId] = (cart[dishId] || 0) + 1;
  writeCart(cart);
}

function changeCartQuantity(dishId, delta) {
  const cart = readCart();
  const nextQuantity = Number(cart[dishId] || 0) + delta;
  if (nextQuantity <= 0) {
    delete cart[dishId];
  } else {
    cart[dishId] = nextQuantity;
  }
  writeCart(cart);
  renderCartPage();
}

function removeFromCart(dishId) {
  const cart = readCart();
  delete cart[dishId];
  writeCart(cart);
  renderCartPage();
}

function parseSearchTerms(query) {
  return String(query || "")
    .toLowerCase()
    .split(/[,;\n|]+|\s{2,}/)
    .map(normalizeSearchTerm)
    .filter(Boolean);
}

function normalizeSearchTerm(term) {
  const cleaned = String(term || "").trim().toLowerCase().replace(/\s+/g, " ");
  if (cleaned === "noodles") return "noodle";
  if (cleaned === "fruits") return "fruit";
  if (cleaned === "chillies" || cleaned === "chilli") return "chili";
  if (cleaned.endsWith("s") && cleaned.length > 4) return cleaned.slice(0, -1);
  return cleaned;
}

function setupDishLocator() {
  // Search is a locator only. It calls the Rust search API to populate the
  // suggestion dropdown, then suggestion clicks scroll to the existing static
  // Menu card. This function intentionally never changes Menu card visibility,
  // Menu card order, or the server-rendered Menu count.
  const searchInput = document.getElementById("search-input");
  const suggestions = document.getElementById("search-suggestions");
  let latestSearch = { query: "", results: [] };
  let searchSequence = 0;

  if (!searchInput || !suggestions) {
    return;
  }

  const updateSuggestions = async () => {
    const query = searchInput.value || "";
    const hasQuery = parseSearchTerms(query).length > 0;
    const sequence = ++searchSequence;

    if (!hasQuery) {
      latestSearch = { query: "", results: [] };
      renderSearchSuggestions(latestSearch, suggestions, false);
      return;
    }

    try {
      const payload = await requestJson(`/api/search?q=${encodeURIComponent(query)}&mode=all`);
      if (sequence !== searchSequence || !payload.ok || !payload.data) {
        return;
      }
      latestSearch = payload.data;
    } catch (error) {
      console.error("Menu search failed", error);
      return;
    }
    renderSearchSuggestions(latestSearch, suggestions, hasQuery);
  };

  const debouncedUpdateSuggestions = debounce(updateSuggestions, 160);
  searchInput?.addEventListener("input", debouncedUpdateSuggestions);

  renderSearchSuggestions(latestSearch, suggestions, false);
}

function debounce(callback, delay) {
  let timer = null;
  return (...args) => {
    window.clearTimeout(timer);
    timer = window.setTimeout(() => callback(...args), delay);
  };
}

function setupCarouselControls() {
  document.querySelectorAll("[data-carousel-scroll]").forEach((button) => {
    const row = document.getElementById(button.dataset.carouselScroll);
    if (!row || button.dataset.carouselBound === "true") {
      return;
    }
    button.dataset.carouselBound = "true";

    const updateDisabledState = () => {
      const controls = document.querySelectorAll(
        `[data-carousel-scroll="${CSS.escape(button.dataset.carouselScroll)}"]`
      );
      const maxScroll = row.scrollWidth - row.clientWidth - 2;
      controls.forEach((control) => {
        const direction = Number(control.dataset.direction || 1);
        control.disabled =
          maxScroll <= 0 ||
          (direction < 0 && row.scrollLeft <= 2) ||
          (direction > 0 && row.scrollLeft >= maxScroll);
      });
    };

    button.addEventListener("click", () => {
      const direction = Number(button.dataset.direction || 1);
      row.scrollBy({
        left: direction * Math.max(220, Math.floor(row.clientWidth * 0.8)),
        behavior: "smooth",
      });
      window.setTimeout(updateDisabledState, 280);
    });
    row.addEventListener("scroll", updateDisabledState, { passive: true });
    window.addEventListener("resize", updateDisabledState);
    updateDisabledState();
  });
}

/**
 * Adds grab-and-drag scrolling for desktop mice and IDE phone previews.
 *
 * Physical phones already scroll overflow containers through native touch
 * gestures. Preview tools commonly translate the same gesture into mouse
 * pointer events, so this controller mirrors touch dragging without replacing
 * native swipe behavior. A short movement threshold preserves ordinary clicks
 * on dish buttons; only a real drag suppresses the following click.
 */
function setupDragScrolling() {
  document
    .querySelectorAll(
      ".recommended-row, .order-filter-row, .admin-section-nav, .tool-tabs, .experiment-tabs, [data-drag-scroll]"
    )
    .forEach((scroller) => {
      if (scroller.dataset.dragScrollBound === "true") return;
      scroller.dataset.dragScrollBound = "true";

      let pointerId = null;
      let startX = 0;
      let startY = 0;
      let startScrollLeft = 0;
      let dragging = false;
      let suppressClick = false;
      const dragThreshold = 6;

      scroller.addEventListener("pointerdown", (event) => {
        // Touch and pen input retain the browser's momentum scrolling. Mouse
        // input includes Lirobi's simulated press-hold-pull interaction.
        if (event.pointerType !== "mouse" || event.button !== 0) return;
        pointerId = event.pointerId;
        startX = event.clientX;
        startY = event.clientY;
        startScrollLeft = scroller.scrollLeft;
        dragging = false;
      });

      scroller.addEventListener("pointermove", (event) => {
        if (pointerId !== event.pointerId) return;
        const distanceX = event.clientX - startX;
        const distanceY = event.clientY - startY;
        if (
          !dragging &&
          Math.max(Math.abs(distanceX), Math.abs(distanceY)) < dragThreshold
        ) {
          return;
        }

        if (!dragging) {
          // Give predominantly vertical gestures to the page scroller. This
          // lets users start an up/down pull directly over a carousel card.
          if (Math.abs(distanceY) > Math.abs(distanceX)) {
            pointerId = null;
            return;
          }
          dragging = true;
          scroller.classList.add("is-dragging");
          scroller.setPointerCapture?.(event.pointerId);
        }
        event.preventDefault();
        scroller.scrollLeft = startScrollLeft - distanceX;
      });

      const finishDrag = (event) => {
        if (pointerId !== event.pointerId) return;
        if (dragging) {
          suppressClick = true;
          window.setTimeout(() => {
            suppressClick = false;
          }, 0);
        }
        if (scroller.hasPointerCapture?.(event.pointerId)) {
          scroller.releasePointerCapture(event.pointerId);
        }
        scroller.classList.remove("is-dragging");
        pointerId = null;
        dragging = false;
      };

      scroller.addEventListener("pointerup", finishDrag);
      scroller.addEventListener("pointercancel", finishDrag);
      scroller.addEventListener("lostpointercapture", (event) => {
        if (pointerId === event.pointerId) {
          scroller.classList.remove("is-dragging");
          pointerId = null;
          dragging = false;
        }
      });
      scroller.addEventListener("dragstart", (event) => event.preventDefault());
      scroller.addEventListener(
        "click",
        (event) => {
          if (!suppressClick) return;
          event.preventDefault();
          event.stopPropagation();
        },
        true
      );
    });
}

/**
 * Lets a mouse emulate a phone's press-hold-pull page gesture.
 *
 * Native touch scrolling remains under browser control. The mouse-only path is
 * intended for device-preview tools such as Lirobi. Interactive controls are
 * excluded so pressing buttons, links and form fields keeps normal behavior.
 */
function setupPageDragScrolling() {
  let pointerId = null;
  let startX = 0;
  let startY = 0;
  let startScrollY = 0;
  let dragging = false;
  let suppressClick = false;
  let startedInHorizontalScroller = false;
  const dragThreshold = 6;
  const interactiveSelector =
    "button, a, input, select, textarea, summary, label, [role='button']";

  document.addEventListener("pointerdown", (event) => {
    if (
      event.pointerType !== "mouse" ||
      event.button !== 0 ||
      event.target.closest(interactiveSelector)
    ) {
      return;
    }

    pointerId = event.pointerId;
    startX = event.clientX;
    startY = event.clientY;
    startScrollY = window.scrollY;
    dragging = false;
    startedInHorizontalScroller = Boolean(
      event.target.closest(
        ".recommended-row, .order-filter-row, .admin-section-nav, .tool-tabs, .experiment-tabs, [data-drag-scroll]"
      )
    );
  });

  document.addEventListener("pointermove", (event) => {
    if (pointerId !== event.pointerId) return;
    const distanceX = event.clientX - startX;
    const distanceY = event.clientY - startY;

    if (
      !dragging &&
      Math.max(Math.abs(distanceX), Math.abs(distanceY)) < dragThreshold
    ) {
      return;
    }

    if (!dragging) {
      // A horizontal gesture that starts in a carousel belongs to that
      // carousel. All predominantly vertical gestures move the page.
      if (startedInHorizontalScroller && Math.abs(distanceX) >= Math.abs(distanceY)) {
        pointerId = null;
        return;
      }
      if (Math.abs(distanceY) <= Math.abs(distanceX)) {
        pointerId = null;
        return;
      }
      dragging = true;
      document.body.classList.add("is-page-dragging");
    }

    event.preventDefault();
    window.scrollTo({ top: startScrollY - distanceY, behavior: "auto" });
  });

  const finishPageDrag = (event) => {
    if (pointerId !== event.pointerId) return;
    if (dragging) {
      suppressClick = true;
      window.setTimeout(() => {
        suppressClick = false;
      }, 0);
    }
    document.body.classList.remove("is-page-dragging");
    pointerId = null;
    dragging = false;
  };

  document.addEventListener("pointerup", finishPageDrag);
  document.addEventListener("pointercancel", finishPageDrag);
  document.addEventListener("dragstart", (event) => {
    if (pointerId !== null) event.preventDefault();
  });
  document.addEventListener(
    "click",
    (event) => {
      if (!suppressClick) return;
      event.preventDefault();
      event.stopPropagation();
    },
    true
  );
}

function setupOrderFilters() {
  document.querySelectorAll("[data-order-filter]").forEach((button) => {
    if (button.dataset.orderFilterBound === "true") return;
    button.dataset.orderFilterBound = "true";
    button.addEventListener("click", () => {
      document
        .querySelectorAll("[data-order-filter]")
        .forEach((item) => item.classList.remove("active"));
      button.classList.add("active");
      const filter = button.dataset.orderFilter;
      document.querySelectorAll("[data-order-status-card]").forEach((card) => {
        card.hidden = filter !== "all" && card.dataset.orderStatusCard !== filter;
      });
    });
  });
}

function renderSearchSuggestions(searchResult, container, hasQuery) {
  if (!container) {
    return;
  }

  if (!hasQuery) {
    container.hidden = true;
    container.innerHTML = "";
    return;
  }

  const matches = (searchResult.results || []).slice(0, 6);

  container.hidden = false;
  if (!matches.length) {
    container.innerHTML = `<div class="suggestion-empty">No matching dishes found</div>`;
    return;
  }

  container.innerHTML = matches
    .map((item) => {
      const dish = item.dish;
      return `
        <button class="suggestion-item" type="button" data-suggestion-dish="${escapeHtml(dish.dish_id)}">
          ${imageHtml(dish, "suggestion-thumb")}
          <span>
            <strong>${escapeHtml(dish.name)}</strong>
            <small>${escapeHtml(dish.category)} · ${escapeHtml(dish.price)} · ${escapeHtml((item.match_reasons || []).slice(0, 2).join("; ") || "menu match")}</small>
          </span>
        </button>
      `;
    })
    .join("");

  container.querySelectorAll("[data-suggestion-dish]").forEach((button) => {
    button.addEventListener("click", () => {
      focusDishFromSuggestion(button.dataset.suggestionDish);
      container.hidden = true;
    });
  });
}

function focusDishFromSuggestion(dishId) {
  const card =
    document.getElementById(`dish-${dishId}`) ||
    document.querySelector(`.dish-card[data-dish-id="${CSS.escape(dishId)}"]`);
  if (card) {
    card.scrollIntoView({ behavior: "smooth", block: "center" });
    card.classList.add("dish-locator-highlight");
    window.setTimeout(() => card.classList.remove("dish-locator-highlight"), 2000);
    return;
  }

  showDishDetail(dishId);
}

function setupCartButtons() {
  document.querySelectorAll("[data-add-cart]").forEach((button) => {
    if (button.dataset.bound === "true") {
      return;
    }
    button.dataset.bound = "true";
    button.addEventListener("click", () => {
      addToCart(button.dataset.addCart);
      const original = button.innerHTML;
      button.innerHTML = `${iconSvg("shopping-cart")} Added`;
      window.setTimeout(() => {
        button.innerHTML = original || `${iconSvg("plus")} Add`;
      }, 800);
    });
  });
}

function collectPreferences(scope) {
  const root = document.querySelector(`[data-preference-scope="${scope}"]`);
  const preferences = {
    liked_ingredients: [],
    disliked_ingredients: [],
    preferred_tags: [],
    selected_dish_ids: [],
  };

  root?.querySelectorAll(".mini-chip.active").forEach((chip) => {
    const kind = chip.dataset.preferenceKind;
    if (preferences[kind]) {
      preferences[kind].push(chip.dataset.preferenceValue);
    }
  });

  if (scope === "customer") {
    preferences.selected_dish_ids = Object.keys(readCart());
    preferences.diversity_mode =
      document.querySelector("[data-diversity-mode].active")?.dataset.diversityMode || "balanced";
  } else {
    preferences.selected_dish_ids = Array.from(
      document.querySelectorAll("[data-admin-context-dish]:checked")
    ).map((option) => option.value);
    preferences.time_context = document.getElementById("admin-time-context")?.value || "Any";
    preferences.ranking_method =
      document.getElementById("admin-ranking-method")?.value || "hybrid";
  }

  return preferences;
}

function setupDiversitySelector() {
  document.querySelectorAll("[data-diversity-mode]").forEach((button) => {
    button.addEventListener("click", () => {
      document.querySelectorAll("[data-diversity-mode]").forEach((item) => {
        item.classList.toggle("active", item === button);
        item.setAttribute("aria-pressed", String(item === button));
      });
      if (mealSetUiState.configuration) {
        mealSetUiState.configuration.diversityMode = button.dataset.diversityMode || "balanced";
        renderMealSetConfiguration();
      }
      refreshCustomerRecommendations();
    });
  });
}

function setupPreferencePanels() {
  document.querySelectorAll("[data-preference-scope]").forEach((panel) => {
    const scope = panel.dataset.preferenceScope;

    panel.querySelectorAll("[data-preference-kind]").forEach((chip) => {
      if (chip.dataset.preferenceBound === "true") return;
      chip.dataset.preferenceBound = "true";
      chip.addEventListener("click", () => {
        const kind = chip.dataset.preferenceKind;
        const value = chip.dataset.preferenceValue;
        chip.classList.toggle("active");
        chip.setAttribute("aria-pressed", String(chip.classList.contains("active")));

        // An ingredient cannot be both liked and disliked. The UI resolves the
        // conflict immediately so the recommender receives a clean preference
        // object without contradictory signals.
        if (kind === "liked_ingredients" && chip.classList.contains("active")) {
          const conflictingChip = panel
            .querySelector(
              `[data-preference-kind="disliked_ingredients"][data-preference-value="${CSS.escape(value)}"]`
            );
          conflictingChip?.classList.remove("active");
          conflictingChip?.setAttribute("aria-pressed", "false");
        }
        if (kind === "disliked_ingredients" && chip.classList.contains("active")) {
          const conflictingChip = panel
            .querySelector(
              `[data-preference-kind="liked_ingredients"][data-preference-value="${CSS.escape(value)}"]`
            );
          conflictingChip?.classList.remove("active");
          conflictingChip?.setAttribute("aria-pressed", "false");
        }

        if (scope === "customer") {
          syncMealSetPreferencesFromDom();
          refreshCustomerRecommendations();
        }
      });
    });

    panel.querySelector("[data-clear-preferences]")?.addEventListener("click", () => {
      panel.querySelectorAll(".mini-chip.active").forEach((chip) => {
        chip.classList.remove("active");
        chip.setAttribute("aria-pressed", "false");
      });
      if (scope === "customer") {
        syncMealSetPreferencesFromDom();
        refreshCustomerRecommendations();
      }
    });
  });
}

async function requestRecommendations(preferences) {
  return requestJson("/api/recommendations", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(preferences),
  });
}

async function refreshCustomerRecommendations() {
  const row = document.getElementById("recommended-row");
  if (!row) {
    return;
  }

  try {
    const result = await requestRecommendations(collectPreferences("customer"));
    window.RECOMMENDATIONS = result.recommendations || [];
    row.innerHTML = window.RECOMMENDATIONS.length
      ? window.RECOMMENDATIONS.map(renderRecommendationCard).join("")
      : `<div class="info-card slim"><strong>No recommendations yet</strong><span>Select preferences or add dishes to cart.</span></div>`;
    renderRecommendationStats(result.stats);
    applyRecommendationBadges();
  setupCartButtons();
  setupDetailButtons();
    window.dispatchEvent(new Event("resize"));
  } catch {
    row.innerHTML = `<div class="info-card slim"><strong>Recommendation refresh failed</strong><span>Please try again.</span></div>`;
  }
}

function renderRecommendationCard(recommendation) {
  const dish = recommendation.dish;
  return `
    <article class="recommendation-card" data-dish-id="${escapeHtml(dish.dish_id)}">
      ${imageHtml(dish, "compact")}
      <div class="card-body">
        <h3>${escapeHtml(dish.name)}</h3>
        <p>${escapeHtml(dish.category)}</p>
        ${evidenceSummaryHtml(recommendation)}
        <span class="reason">${escapeHtml(shortRecommendationReason(recommendation))}</span>
        <strong>${escapeHtml(dish.price)}</strong>
        <div class="card-actions">
          <button class="add-button" data-add-cart="${escapeHtml(dish.dish_id)}" type="button">${iconSvg("plus")} Add</button>
          <button class="ghost-action" data-view-dish="${escapeHtml(dish.dish_id)}" type="button">${iconSvg("circle-info")} Why this?</button>
        </div>
      </div>
    </article>
  `;
}

function renderRecommendationStats(stats) {
  const target = document.getElementById("recommendation-stats");
  if (!target || !stats) {
    return;
  }

  const matched = Number(stats.matched_preferences || 0);
  const shown = Number(stats.recommended_shown || (window.RECOMMENDATIONS || []).length);
  const fallback =
    matched === 0 && shown > 0
      ? `<span>No exact preference match found. Showing popular alternatives.</span>`
      : "";
  target.innerHTML = `
    <span>Eligible dishes: <strong>${stats.eligible_dishes ?? stats.filtered_dishes}</strong></span>
    <span>Matched preferences: <strong>${matched}</strong></span>
    <span>Excluded disliked: <strong>${stats.excluded_due_to_disliked}</strong></span>
    <span>Skipped cart dishes: <strong>${stats.skipped_selected_dishes}</strong></span>
    <span>Recommended shown: <strong>${shown}</strong></span>
    ${fallback}
  `;
}

function applyRecommendationBadges() {
  const recommendedIds = new Set((window.RECOMMENDATIONS || []).map((item) => item.dish.dish_id));
  document.querySelectorAll(".dish-card").forEach((card) => {
    const badge = card.querySelector(".badge");
    const shouldShow = recommendedIds.has(card.dataset.dishId);
    if (badge) {
      badge.hidden = !shouldShow;
    }
  });
}

function setupDetailButtons() {
  document.querySelectorAll("[data-view-dish]").forEach((button) => {
    if (button.dataset.detailBound === "true") {
      return;
    }
    button.dataset.detailBound = "true";
    button.addEventListener("click", () => showDishDetail(button.dataset.viewDish));
  });

  const dialog = document.getElementById("dish-detail-modal");
  if (dialog && dialog.dataset.modalBound !== "true") {
    dialog.dataset.modalBound = "true";
    dialog
      .querySelector("[data-close-dish-modal]")
      ?.addEventListener("click", () => closeDishDetail());
    dialog.addEventListener("close", () => {
      dishDetailReturnFocus?.focus();
      dishDetailReturnFocus = null;
    });
  }
}

let dishDetailReturnFocus = null;

function showDishDetail(dishId) {
  const dish = dishById(dishId) || recommendationByDishId(dishId)?.dish;
  const dialog = document.getElementById("dish-detail-modal");
  const content = document.getElementById("dish-detail-content");
  if (!dish || !dialog || !content) {
    return;
  }

  const recommendation = recommendationByDishId(dishId);
  const evidence = recommendation?.evidence;
  const weights = adaptivePercentages(recommendation?.adaptive_weights);
  const evidenceNotes = (evidence?.evidence_notes || [])
    .map((note) => `<li>${escapeHtml(note)}</li>`)
    .join("");
  content.innerHTML = `
    <div class="detail-layout">
      ${imageHtml(dish, "large")}
      <div>
        <p class="eyebrow">${escapeHtml(dish.dish_id)} · ${escapeHtml(dish.category)}</p>
        <h2 id="dish-detail-title">${escapeHtml(dish.name)}</h2>
        <p><strong>Ingredients:</strong> ${escapeHtml((dish.ingredients || []).join(", "))}</p>
        <p><strong>Tags:</strong> ${escapeHtml(formatList(dish.tags || []))}</p>
        <p><strong>Price:</strong> ${escapeHtml(dish.price)}</p>
        ${
          recommendation
            ? `<div class="reason-box evidence-detail">
                <strong>Why this dish was recommended</strong>
                <p>${escapeHtml(recommendation.explanation)}</p>
                <h3>Recommendation score</h3>
                <p><strong>Adaptive hybrid score:</strong> ${recommendation.hybrid_score.toFixed(2)}</p>
                <p><strong>Base rank / reranked rank:</strong> ${recommendation.base_rank || "-"} / ${recommendation.reranked_rank || "-"}</p>
                <p><strong>Diversity-adjusted score:</strong> ${Number(recommendation.reranked_score || 0).toFixed(2)}</p>
                <ul class="evidence-note-list">${(recommendation.diversity_notes || []).map((note) => `<li>${escapeHtml(note)}</li>`).join("")}</ul>
                <h3>Evidence confidence</h3>
                ${evidenceSummaryHtml(recommendation)}
                <p><strong>${escapeHtml(evidenceLabel(evidence?.confidence_level))} — ${Math.round(Number(evidence?.overall_confidence || 0) * 100)}%</strong></p>
                <p class="confidence-disclaimer">This is the strength of the available recommendation evidence, not the probability that you will like the dish.</p>
                <h3>Adaptive weights used</h3>
                <div class="weight-breakdown">
                  ${weightBar("Content preference", weights[0])}
                  ${weightBar("Co-ordering", weights[1])}
                  ${weightBar("Popularity", weights[2])}
                  ${weightBar("Time/context", weights[3])}
                </div>
                <h3>Evidence breakdown</h3>
                <ul class="evidence-note-list">${evidenceNotes || "<li>Evidence is currently limited.</li>"}</ul>
                <p><strong>Primary source:</strong> ${escapeHtml(evidenceSourceLabel(evidence?.primary_evidence_source))}</p>
                <details>
                  <summary>Technical score breakdown</summary>
                  <p>Content ${recommendation.content_score.toFixed(2)} · Co-order ${recommendation.co_order_score.toFixed(2)} · Popularity ${recommendation.popularity_score.toFixed(2)} · Time ${recommendation.business_rule_score.toFixed(2)} · Hybrid ${recommendation.hybrid_score.toFixed(2)}</p>
                  <p>Pair count ${recommendation.association_pair_count} · Context orders ${evidence?.selected_context_order_count || 0} · Candidate popularity ${evidence?.candidate_popularity_count || 0}</p>
                  <p>Support ${recommendation.association_support.toFixed(2)} · Association confidence ${recommendation.association_confidence.toFixed(2)} · Lift ${recommendation.association_lift.toFixed(2)}</p>
                </details>
              </div>`
            : ""
        }
        <button class="primary-action" data-add-cart="${escapeHtml(dish.dish_id)}" type="button">${iconSvg("shopping-cart")} Add to Cart</button>
      </div>
    </div>
  `;
  setupCartButtons();
  dishDetailReturnFocus =
    document.activeElement instanceof HTMLElement ? document.activeElement : null;

  if (typeof dialog.showModal === "function") {
    dialog.showModal();
  } else {
    dialog.setAttribute("open", "open");
  }
}

const MEAL_SET_DEFAULTS = Object.freeze({
  budget: "60",
  partySize: 2,
  targetDishCount: null,
  topSetCount: 3,
  diversityMode: "balanced",
});

// Configuration, request progress, generated data, and errors intentionally
// remain independent. Generating a result can therefore never freeze or erase
// the customer's current choices.
const mealSetUiState = {
  configuration: null,
  loading: false,
  result: null,
  error: null,
};

function defaultMealSetConfiguration() {
  return {
    budget: MEAL_SET_DEFAULTS.budget,
    partySize: MEAL_SET_DEFAULTS.partySize,
    targetDishCount: MEAL_SET_DEFAULTS.targetDishCount,
    topSetCount: MEAL_SET_DEFAULTS.topSetCount,
    requiredCategories: new Set(),
    likedIngredients: new Set(),
    dislikedIngredients: new Set(),
    preferredTags: new Set(),
    selectedDishIds: new Set(),
    diversityMode: MEAL_SET_DEFAULTS.diversityMode,
  };
}

function syncMealSetPreferencesFromDom() {
  if (!mealSetUiState.configuration) return;
  const preferences = collectPreferences("customer");
  mealSetUiState.configuration.likedIngredients = new Set(preferences.liked_ingredients || []);
  mealSetUiState.configuration.dislikedIngredients = new Set(
    preferences.disliked_ingredients || []
  );
  mealSetUiState.configuration.preferredTags = new Set(preferences.preferred_tags || []);
}

function syncMealSetConfigurationFromDom() {
  const configuration = mealSetUiState.configuration;
  if (!configuration) return;
  configuration.budget = document.getElementById("meal-budget")?.value || "";
  configuration.partySize = Number(document.getElementById("meal-party-size")?.value || 0);
  const targetCount = Number(document.getElementById("meal-target-count")?.value || 0);
  configuration.targetDishCount = targetCount > 0 ? targetCount : null;
  configuration.topSetCount = Number(document.getElementById("meal-set-count")?.value || 3);
  configuration.requiredCategories = new Set(
    Array.from(document.querySelectorAll("[data-meal-category]:checked")).map(
      (input) => input.value
    )
  );
  configuration.selectedDishIds = new Set(
    Array.from(document.querySelectorAll("[data-meal-context]:checked")).map(
      (input) => input.value
    )
  );
  configuration.diversityMode =
    document.querySelector("[data-diversity-mode].active")?.dataset.diversityMode || "balanced";
  syncMealSetPreferencesFromDom();
}

function renderMealSetConfiguration() {
  const configuration = mealSetUiState.configuration;
  if (!configuration) return;
  const setValue = (id, value) => {
    const field = document.getElementById(id);
    if (field) field.value = value ?? "";
  };
  setValue("meal-budget", configuration.budget);
  setValue("meal-party-size", configuration.partySize);
  setValue("meal-target-count", configuration.targetDishCount);
  setValue("meal-set-count", configuration.topSetCount);

  document.querySelectorAll("[data-meal-category]").forEach((input) => {
    input.checked = configuration.requiredCategories.has(input.value);
  });
  document.querySelectorAll("[data-meal-context]").forEach((input) => {
    input.checked = configuration.selectedDishIds.has(input.value);
  });
  document.querySelectorAll("[data-diversity-mode]").forEach((button) => {
    const selected = button.dataset.diversityMode === configuration.diversityMode;
    button.classList.toggle("active", selected);
    button.setAttribute("aria-pressed", String(selected));
  });

  const descriptions = {
    familiar: "Familiar prioritises strong preference and popularity evidence.",
    balanced: "Balanced combines familiar matches with some variety.",
    discover: "Discover introduces more novel dishes while respecting exclusions.",
  };
  const description = document.getElementById("diversity-description");
  if (description) {
    description.textContent =
      descriptions[configuration.diversityMode] || descriptions.balanced;
  }
}

function setMealSetLoading(loading) {
  mealSetUiState.loading = loading;
  document
    .querySelectorAll(
      "[data-meal-control], [data-meal-category], [data-meal-context]"
    )
    .forEach((control) => {
      control.disabled = loading;
    });
  const button = document.getElementById("build-meal-set");
  if (button) button.textContent = loading ? "Generating..." : "Generate Meal Set";
}

function renderMealSetResult() {
  const target = document.getElementById("meal-set-results");
  if (!target) return;
  if (!mealSetUiState.result) {
    target.innerHTML = `
      <div class="empty-state">
        <strong>No meal set has been generated yet.</strong>
        <span>Choose your table settings and generate a meal set when ready.</span>
      </div>`;
    return;
  }

  const configuration = mealSetUiState.configuration;
  const mode =
    configuration.diversityMode.charAt(0).toUpperCase() +
    configuration.diversityMode.slice(1);
  target.innerHTML = `
    <div class="meal-result-heading" tabindex="-1">
      <h3>Meal set results</h3>
      <p>${formatCurrency(configuration.budget)} · ${
        configuration.partySize
      } people · ${mode} style</p>
    </div>
    ${(mealSetUiState.result || []).map(renderMealSet).join("")}`;
  target.querySelectorAll("[data-add-meal-set]").forEach((addButton) => {
    addButton.addEventListener("click", () => {
      const ids = JSON.parse(addButton.dataset.addMealSet || "[]");
      const cart = readCart();
      ids.forEach((id) => {
        cart[id] = Math.max(1, Number(cart[id] || 0));
      });
      writeCart(cart);
      addButton.textContent = "Added to Cart";
      showToast("Meal set added to Cart.");
    });
  });
}

// Clear Choices is the single reset path for every meal-set input. Cart,
// profile, order history, dish availability, and server recommendation data
// are deliberately not changed.
function resetMealSetForm() {
  mealSetUiState.configuration = defaultMealSetConfiguration();
  mealSetUiState.loading = false;
  mealSetUiState.result = null;
  mealSetUiState.error = null;
  document
    .querySelectorAll('[data-preference-scope="customer"] .mini-chip.active')
    .forEach((chip) => chip.classList.remove("active"));
  renderMealSetConfiguration();
  renderMealSetResult();
  setMealSetLoading(false);
  const status = document.getElementById("meal-set-status");
  if (status) status.textContent = "";
  refreshCustomerRecommendations();
}

function clearMealSetResult() {
  mealSetUiState.result = null;
  mealSetUiState.error = null;
  renderMealSetResult();
  const status = document.getElementById("meal-set-status");
  if (status) status.textContent = "";
}

function mealSetPayload() {
  const configuration = mealSetUiState.configuration;
  return {
    budget_cents: Math.round(Number(configuration.budget || 0) * 100),
    party_size: configuration.partySize,
    target_dish_count: configuration.targetDishCount,
    top_set_count: configuration.topSetCount,
    liked_ingredients: Array.from(configuration.likedIngredients),
    disliked_ingredients: Array.from(configuration.dislikedIngredients),
    preferred_tags: Array.from(configuration.preferredTags),
    selected_dish_ids: Array.from(configuration.selectedDishIds),
    required_categories: Array.from(configuration.requiredCategories),
    diversity_mode: configuration.diversityMode,
  };
}

function setupMealSetBuilder() {
  const button = document.getElementById("build-meal-set");
  const clearChoices = document.getElementById("clear-meal-choices");
  const clearResult = document.getElementById("clear-meal-result");
  const status = document.getElementById("meal-set-status");
  if (!button || !status) return;

  mealSetUiState.configuration = defaultMealSetConfiguration();
  mealSetUiState.configuration.selectedDishIds = new Set(Object.keys(readCart()));
  syncMealSetPreferencesFromDom();
  renderMealSetConfiguration();

  ["meal-budget", "meal-party-size", "meal-target-count", "meal-set-count"].forEach((id) => {
    document.getElementById(id)?.addEventListener("input", syncMealSetConfigurationFromDom);
  });
  document.querySelectorAll("[data-meal-category], [data-meal-context]").forEach((input) => {
    input.addEventListener("change", syncMealSetConfigurationFromDom);
  });

  clearChoices?.addEventListener("click", () => {
    resetMealSetForm();
    showToast("Meal-set choices cleared.");
  });
  clearResult?.addEventListener("click", () => {
    clearMealSetResult();
    showToast("Meal-set result cleared. Your choices were kept.");
  });

  button.addEventListener("click", async () => {
    syncMealSetConfigurationFromDom();
    const configuration = mealSetUiState.configuration;
    if (Number(configuration.budget) <= 0 || configuration.partySize <= 0) {
      mealSetUiState.error = "Enter a budget and party size greater than zero.";
      status.textContent = mealSetUiState.error;
      return;
    }

    setMealSetLoading(true);
    mealSetUiState.error = null;
    status.textContent = "Generating meal sets...";
    try {
      const result = await requestJson("/api/recommendations/meal-set", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(mealSetPayload()),
      });
      if (!result.ok) throw new Error(result.message || "Unable to build a meal set.");
      mealSetUiState.result = result.data || [];
      status.textContent = result.message;
      renderMealSetResult();
      document.querySelector(".meal-result-heading")?.focus({ preventScroll: true });
      showToast("Meal set generated.");
    } catch (error) {
      mealSetUiState.error = error.message || "Unable to build a meal set.";
      status.textContent = mealSetUiState.error;
    } finally {
      // Every success and failure path restores the controls, preventing the
      // one-use locked state seen in the previous implementation.
      setMealSetLoading(false);
    }
  });
}

function renderMealSet(set, index) {
  const dishes = (set.dishes || [])
    .map(
      (dish) => `<li><strong>${escapeHtml(dish.name)}</strong> <span>${escapeHtml(dish.dish_id)} · ${escapeHtml(dish.price)}</span></li>`
    )
    .join("");
  const notes = (set.explanation_notes || [])
    .map((note) => `<li>${escapeHtml(note)}</li>`)
    .join("");
  const ids = JSON.stringify((set.dishes || []).map((dish) => dish.dish_id));
  return `
    <article class="meal-set-card">
      <div class="section-heading"><div><h3>Set ${index + 1}</h3><p>${escapeHtml((set.represented_categories || []).join(" · "))}</p></div><strong>${formatCurrency(Number(set.total_price_cents || 0) / 100)}</strong></div>
      <ul class="meal-dish-list">${dishes}</ul>
      <div class="meal-score-grid">
        <span>Set score <strong>${Number(set.final_set_score || 0).toFixed(2)}</strong></span>
        <span>Preference coverage <strong>${Math.round(Number(set.preference_coverage || 0) * 100)}%</strong></span>
        <span>Category coverage <strong>${Math.round(Number(set.category_coverage || 0) * 100)}%</strong></span>
        <span>Pair compatibility <strong>${Math.round(Number(set.pair_compatibility || 0) * 100)}%</strong></span>
        <span>Set diversity <strong>${Math.round(Number(set.set_diversity || 0) * 100)}%</strong></span>
        <span>Budget remaining <strong>${formatCurrency(Number(set.remaining_budget_cents || 0) / 100)}</strong></span>
      </div>
      <details><summary>Why this set?</summary><ul>${notes}</ul></details>
      <button class="primary-action" type="button" data-add-meal-set='${escapeHtml(ids)}'>Add Entire Set</button>
    </article>
  `;
}

function weightBar(label, percent) {
  return `
    <div class="weight-row">
      <span>${escapeHtml(label)}</span>
      <span class="weight-track" aria-hidden="true"><span style="width:${percent}%"></span></span>
      <strong>${percent}%</strong>
    </div>
  `;
}

function closeDishDetail() {
  const dialog = document.getElementById("dish-detail-modal");
  if (!dialog) {
    return;
  }
  if (typeof dialog.close === "function") {
    dialog.close();
  } else {
    dialog.removeAttribute("open");
  }
}

function renderCartPage() {
  const container = document.getElementById("cart-page-items");
  const totalElement = document.getElementById("cart-page-total");
  const uniqueElement = document.getElementById("cart-unique-count");
  const portionsElement = document.getElementById("cart-portions-count");
  if (!container || !totalElement) {
    return;
  }

  const cart = readCart();
  const entries = Object.entries(cart);
  const totals = calculateCartTotals(cart);
  if (uniqueElement) uniqueElement.textContent = String(totals.uniqueDishes);
  if (portionsElement) portionsElement.textContent = String(totals.totalPortions);
  totalElement.textContent = formatCurrency(totals.subtotal);

  if (!entries.length) {
    container.innerHTML =
      '<div class="info-card"><strong>Your cart is empty</strong><span>Add dishes from Home.</span></div>';
    return;
  }

  container.innerHTML = entries
    .map(([dishId, rawQuantity]) => {
      const dish = dishById(dishId);
      if (!dish) {
        return "";
      }
      const quantity = Math.max(1, Number(rawQuantity || 1));
      const unitPrice = parsePriceAmount(dish);
      const lineTotal = unitPrice * quantity;
      return `
        <article class="cart-item" data-cart-dish-id="${escapeHtml(dishId)}">
          ${imageHtml(dish, "cart-item__image")}
          <div class="cart-item__details">
            <h3 class="cart-item__name">${escapeHtml(dish.name)}</h3>
            <p class="cart-item__category">${escapeHtml(dish.category)}</p>
            <p class="cart-item__unit-price">${formatCurrency(unitPrice)} each</p>
          </div>
          <div class="cart-item__quantity">
            <span class="cart-field-label">Quantity</span>
            <div class="quantity-stepper">
              <button type="button" data-action="decrease-cart-quantity" data-cart-decrease="${escapeHtml(
                dishId
              )}" aria-label="Decrease ${escapeHtml(
                dish.name
              )} quantity" title="Decrease quantity">${iconSvg("minus")}</button>
              <output class="quantity-stepper__value" aria-live="polite" aria-label="${escapeHtml(
                dish.name
              )} quantity">${quantity}</output>
              <button type="button" data-action="increase-cart-quantity" data-cart-increase="${escapeHtml(
                dishId
              )}" aria-label="Increase ${escapeHtml(
                dish.name
              )} quantity" title="Increase quantity">${iconSvg("plus")}</button>
            </div>
          </div>
          <div class="cart-item__total">
            <span class="cart-field-label">Total</span>
            <strong class="cart-item__total-price">${formatCurrency(lineTotal)}</strong>
          </div>
          <button class="cart-item__remove icon-button danger-icon" type="button" data-action="remove-cart-item" data-remove-cart="${escapeHtml(
            dishId
          )}" aria-label="Remove ${escapeHtml(
            dish.name
          )} from cart" title="Remove from cart">${iconSvg("trash-2")}</button>
        </article>
      `;
    })
    .join("");

  document.querySelectorAll("[data-cart-decrease]").forEach((button) => {
    button.addEventListener("click", () => changeCartQuantity(button.dataset.cartDecrease, -1));
  });
  document.querySelectorAll("[data-cart-increase]").forEach((button) => {
    button.addEventListener("click", () => changeCartQuantity(button.dataset.cartIncrease, 1));
  });
  document.querySelectorAll("[data-remove-cart]").forEach((button) => {
    button.addEventListener("click", () => removeFromCart(button.dataset.removeCart));
  });
}

function setupCheckout() {
  const button = document.getElementById("checkout-button");
  const status = document.getElementById("checkout-status");
  if (!button || !status) {
    return;
  }

  button.addEventListener("click", async () => {
    const dishIds = Object.entries(readCart()).flatMap(([dishId, quantity]) =>
      Array.from({ length: Number(quantity || 0) }, () => dishId)
    );
    if (!dishIds.length) {
      status.textContent = "Add at least one dish before checkout.";
      return;
    }
    const identity = readCustomerIdentity();
    if (!identity.session_id) {
      status.textContent = "Your customer session expired. Please register again.";
      return;
    }
    const note = document.getElementById("customer-note")?.value.trim() || "";

    button.disabled = true;
    status.textContent = "Placing order...";

    try {
      const result = await requestJson("/api/orders", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ dish_ids: dishIds, note }),
      });
      status.textContent = result.message || "Checkout failed. Please try again.";

      if (result.ok) {
        if (result.order_id) {
          localStorage.setItem(LAST_ORDER_KEY, result.order_id);
        }
        writeCart({});
        renderCartPage();
        window.setTimeout(() => {
          window.location.href = "/profile";
        }, 700);
      }
    } catch (error) {
      console.error("Checkout request failed", error);
      status.textContent = "We could not reach the order server. Please try again.";
    } finally {
      button.disabled = false;
    }
  });
}

function setupSmartMenuAssistant() {
  const prompt = document.getElementById("search-input");
  const button = document.getElementById("assistant-run");
  const understood = document.getElementById("assistant-understood");
  const upsells = document.getElementById("assistant-upsells");
  const row = document.getElementById("recommended-row");
  if (!prompt || !button || !understood || !row) {
    // The Home search bar is now a dish locator only. When the optional
    // assistant button is not rendered, typing should only show search
    // suggestions and must not update recommendations or the static Menu grid.
    return;
  }

  const runAssistant = async () => {
    const text = prompt.value.trim();
    if (!text) {
      understood.textContent = "Type a dish, ingredient, or preference first, for example: spicy chicken but no beef.";
      return;
    }

    button.disabled = true;
    understood.textContent = "Understanding your request...";
    try {
      const result = await requestJson("/api/assistant/recommendations", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          prompt: text,
          selected_dish_ids: Object.keys(readCart()),
        }),
      });
      understood.textContent = result.understood || "Assistant could not match menu terms.";
      window.RECOMMENDATIONS = result.recommendations || [];
      row.innerHTML = window.RECOMMENDATIONS.length
        ? window.RECOMMENDATIONS.map(renderRecommendationCard).join("")
        : `<div class="info-card slim"><strong>No assistant recommendations</strong><span>Try menu words such as chicken, rice, spicy, dessert, or no beef.</span></div>`;
      if (upsells) {
        upsells.innerHTML = (result.upsells || []).length
          ? (result.upsells || [])
              .map((item) => `<span class="assistant-pill">${escapeHtml(item)}</span>`)
              .join("")
          : "";
      }
      renderRecommendationStats(result.stats);
      applyRecommendationBadges();
    setupCartButtons();
    setupDetailButtons();
      window.dispatchEvent(new Event("resize"));
    } catch {
      understood.textContent = "Assistant request failed. Please try again.";
    } finally {
      button.disabled = false;
    }
  };

  button.addEventListener("click", runAssistant);
  prompt.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      runAssistant();
    }
  });
}

function setupAdminTools() {
  setupAdminOrderStatus();
  setupAdminOrderPolling();
  setupDishManagement();
  setupAdaptiveInspector();
  setupCounterfactualExplorer();
  setupLearningTimeline();
  setupAdminRecommendationTester();
  setupAdminInsights();
  setupSimulationTester();
  setupAdminMealSetTester();
  setupRecommendationTesterNavigation();
}

function setupRecommendationTesterNavigation() {
  const overview = document.querySelector("[data-tester-overview]");
  const shell = document.querySelector("[data-tester-shell]");
  const panels = Array.from(document.querySelectorAll("[data-tool-panel]"));
  const categories = Array.from(document.querySelectorAll("[data-tester-category]"));
  const tools = Array.from(document.querySelectorAll("[data-tester-tool]"));
  const categorySelect = document.getElementById("tester-category-select");
  if (!overview || !shell || !panels.length) return;

  const defaultTools = {
    production: "adaptive",
    experiments: "ingredient-impact",
    explainability: "counterfactual",
    learning: "timeline",
  };

  const showOverview = (updateHash = true) => {
    overview.hidden = false;
    shell.hidden = true;
    panels.forEach((panel) => {
      panel.hidden = true;
    });
    if (updateHash) window.location.hash = "overview";
  };

  const activate = (category, tool, updateHash = true) => {
    const selectedTool =
      tools.find(
        (button) =>
          button.dataset.toolCategory === category && button.dataset.testerTool === tool
      ) ||
      tools.find(
        (button) =>
          button.dataset.toolCategory === category &&
          button.dataset.testerTool === defaultTools[category]
      );
    if (!selectedTool) {
      showOverview(updateHash);
      return;
    }

    overview.hidden = true;
    shell.hidden = false;
    categories.forEach((button) => {
      const active = button.dataset.testerCategory === category;
      button.classList.toggle("active", active);
      button.setAttribute("aria-current", active ? "page" : "false");
    });
    tools.forEach((button) => {
      button.hidden = button.dataset.toolCategory !== category;
      const active = button === selectedTool;
      button.classList.toggle("active", active);
      button.setAttribute("aria-selected", String(active));
    });
    panels.forEach((panel) => {
      panel.hidden = panel.dataset.toolPanel !== selectedTool.dataset.toolTarget;
    });
    if (categorySelect) categorySelect.value = category;

    // Controlled experiment shortcuts retain the existing tested tab logic;
    // the category layer only decides which primary workspace is visible.
    const experiment = selectedTool.dataset.experimentShortcut;
    if (experiment) {
      document.querySelector(`[data-experiment-tab="${experiment}"]`)?.click();
    }
    if (updateHash) {
      window.location.hash = `${category}/${selectedTool.dataset.testerTool}`;
    }
  };

  categories.forEach((button) => {
    button.addEventListener("click", () =>
      activate(
        button.dataset.testerCategory,
        button.dataset.defaultTool || defaultTools[button.dataset.testerCategory]
      )
    );
  });
  tools.forEach((button) => {
    button.addEventListener("click", () =>
      activate(button.dataset.toolCategory, button.dataset.testerTool)
    );
  });
  document.querySelectorAll("[data-open-tester-category]").forEach((button) => {
    button.addEventListener("click", () => {
      const category = button.dataset.openTesterCategory;
      activate(category, defaultTools[category]);
    });
  });
  document.querySelector("[data-tester-home]")?.addEventListener("click", () => showOverview());
  categorySelect?.addEventListener("change", () =>
    activate(categorySelect.value, defaultTools[categorySelect.value])
  );

  const restoreFromHash = () => {
    const raw = window.location.hash.replace(/^#/, "");
    if (!raw || raw === "overview") {
      showOverview(false);
      return;
    }
    const [category, tool] = raw.split("/");
    activate(category, tool || defaultTools[category], false);
  };
  window.addEventListener("hashchange", restoreFromHash);
  restoreFromHash();
}

function setupAdminMealSetTester() {
  const run = document.getElementById("run-admin-meal-set");
  const reset = document.getElementById("reset-admin-meal-set");
  const clear = document.getElementById("clear-admin-meal-result");
  const target = document.getElementById("admin-meal-results");
  const status = document.getElementById("admin-meal-status");
  if (!run || !target || !status) return;
  const selected = (id) =>
    Array.from(document.getElementById(id)?.selectedOptions || []).map(
      (option) => option.value
    );
  const clearSelections = () => {
    ["admin-meal-liked", "admin-meal-disliked", "admin-meal-tags", "admin-meal-context"].forEach(
      (id) => {
        Array.from(document.getElementById(id)?.options || []).forEach((option) => {
          option.selected = false;
        });
      }
    );
  };

  run.addEventListener("click", async () => {
    const budget = Number(document.getElementById("admin-meal-budget")?.value || 0);
    const party = Number(document.getElementById("admin-meal-party")?.value || 0);
    const targetCount = Number(document.getElementById("admin-meal-target")?.value || 0);
    if (budget <= 0 || party <= 0) {
      status.textContent = "Enter a budget and party size greater than zero.";
      return;
    }
    run.disabled = true;
    run.textContent = "Generating...";
    status.textContent = "Running the production meal-set service...";
    try {
      const result = await requestJson("/api/recommendations/meal-set", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          budget_cents: Math.round(budget * 100),
          party_size: party,
          target_dish_count: targetCount > 0 ? targetCount : null,
          top_set_count: 3,
          liked_ingredients: selected("admin-meal-liked"),
          disliked_ingredients: selected("admin-meal-disliked"),
          preferred_tags: selected("admin-meal-tags"),
          selected_dish_ids: selected("admin-meal-context"),
          required_categories: [],
          diversity_mode:
            document.getElementById("admin-meal-diversity")?.value || "balanced",
        }),
      });
      if (!result.ok) throw new Error(result.message);
      target.innerHTML = (result.data || []).map(renderAdminMealSet).join("");
      status.textContent = result.message;
    } catch (error) {
      status.textContent = error.message || "Unable to generate meal sets.";
    } finally {
      run.disabled = false;
      run.textContent = "Generate Meal Set";
    }
  });
  clear?.addEventListener("click", () => {
    target.innerHTML = "";
    status.textContent = "";
  });
  reset?.addEventListener("click", () => {
    document.getElementById("admin-meal-budget").value = 60;
    document.getElementById("admin-meal-party").value = 2;
    document.getElementById("admin-meal-target").value = "";
    document.getElementById("admin-meal-diversity").value = "balanced";
    clearSelections();
    target.innerHTML = "";
    status.textContent = "";
  });
}

function renderAdminMealSet(set, index) {
  const dishes = (set.dishes || [])
    .map(
      (dish) =>
        `<li><strong>${escapeHtml(dish.name)}</strong><span>${escapeHtml(
          dish.dish_id
        )} · ${escapeHtml(dish.price)}</span></li>`
    )
    .join("");
  return `
    <article class="meal-set-card">
      <div class="section-heading"><div><h3>Set ${index + 1}</h3><p>${escapeHtml(
        (set.represented_categories || []).join(" · ")
      )}</p></div><strong>${formatCurrency(
        Number(set.total_price_cents || 0) / 100
      )}</strong></div>
      <ul class="meal-dish-list">${dishes}</ul>
      <p><strong>Preference coverage:</strong> ${Math.round(
        Number(set.preference_coverage || 0) * 100
      )}% · <strong>Budget remaining:</strong> ${formatCurrency(
        Number(set.remaining_budget_cents || 0) / 100
      )}</p>
    </article>`;
}

function setupAdaptiveInspector() {
  const runButton = document.getElementById("run-adaptive-inspector");
  const resetButton = document.getElementById("reset-adaptive-inspector");
  const target = document.getElementById("adaptive-inspector-results");
  if (!runButton || !target) return;

  const values = (id) =>
    Array.from(document.getElementById(id)?.selectedOptions || []).map((option) => option.value);

  runButton.addEventListener("click", async () => {
    runButton.disabled = true;
    target.innerHTML = `<div class="info-card slim"><strong>Calculating adaptive evidence...</strong></div>`;
    try {
      const result = await requestRecommendations({
        liked_ingredients: values("adaptive-liked"),
        disliked_ingredients: values("adaptive-disliked"),
        preferred_tags: values("adaptive-tags"),
        selected_dish_ids: values("adaptive-context"),
        time_context: document.getElementById("adaptive-time")?.value || "Any",
        ranking_method: "hybrid",
      });
      renderAdaptiveInspector(result, target);
    } catch {
      target.innerHTML = `<div class="info-card slim error"><strong>Unable to calculate adaptive scoring.</strong></div>`;
    } finally {
      runButton.disabled = false;
    }
  });

  resetButton?.addEventListener("click", () => {
    ["adaptive-liked", "adaptive-disliked", "adaptive-tags", "adaptive-context"].forEach((id) => {
      Array.from(document.getElementById(id)?.options || []).forEach((option) => {
        option.selected = false;
      });
    });
    const time = document.getElementById("adaptive-time");
    if (time) time.value = "Any";
    target.innerHTML = "";
  });
}

function renderAdaptiveInspector(result, target) {
  const profile = result.evidence_profile || {};
  const config = result.scoring_config || {};
  const weights = adaptivePercentages(result.adaptive_weights);
  const collaborative = Number(profile.collaborative_confidence || 0);
  const dataNote =
    collaborative < 0.4
      ? "Collaborative evidence is limited, so the system currently gives more weight to explicit ingredient preferences or popularity."
      : "Enough co-order evidence is available for collaborative filtering to receive greater weight.";
  const rows = (result.recommendations || [])
    .map((item, index) => {
      const evidence = item.evidence || {};
      const notes = (evidence.evidence_notes || [])
        .map((note) => `<li>${escapeHtml(note)}</li>`)
        .join("");
      const itemWeights = adaptivePercentages(item.adaptive_weights);
      return `
        <tr>
          <td data-label="Base rank">${item.base_rank || index + 1}</td>
          <td data-label="Reranked">${item.reranked_rank || index + 1}</td>
          <td data-label="Dish"><strong>${escapeHtml(item.dish.name)}</strong><span>${escapeHtml(item.dish.dish_id)}</span></td>
          <td data-label="Base / reranked">${Number(item.base_score).toFixed(2)} / ${Number(item.reranked_score).toFixed(2)}</td>
          <td data-label="Diversity">Novelty ${Number(item.novelty_score).toFixed(2)} · Similarity ${Number(item.max_similarity).toFixed(2)} · Category ${Number(item.category_bonus).toFixed(2)}</td>
          <td data-label="Confidence"><span class="evidence-badge evidence-${escapeHtml(evidence.confidence_level || "insufficient")}">${escapeHtml(evidenceLabel(evidence.confidence_level))}</span><span>${Math.round(Number(evidence.overall_confidence || 0) * 100)}%</span></td>
          <td data-label="Primary evidence">${escapeHtml(evidenceSourceLabel(evidence.primary_evidence_source))}</td>
          <td data-label="Adaptive weights">${itemWeights[0]}/${itemWeights[1]}/${itemWeights[2]}/${itemWeights[3]}%</td>
          <td data-label="Evidence details">
            <details>
              <summary>View evidence</summary>
              <p>Scores: content ${Number(item.content_score).toFixed(2)}, co-order ${Number(item.co_order_score).toFixed(2)}, popularity ${Number(item.popularity_score).toFixed(2)}, time ${Number(item.business_rule_score).toFixed(2)}</p>
              <p>Contributions: content ${Number(evidence.contributions?.content || 0).toFixed(2)}, co-order ${Number(evidence.contributions?.co_order || 0).toFixed(2)}, popularity ${Number(evidence.contributions?.popularity || 0).toFixed(2)}, time ${Number(evidence.contributions?.time_context || 0).toFixed(2)}</p>
              <p>Pair count ${evidence.candidate_pair_count || 0}; candidate appearances ${evidence.candidate_popularity_count || 0}; support ${Number(item.association_support).toFixed(2)}; association confidence ${Number(item.association_confidence).toFixed(2)}; lift ${Number(item.association_lift).toFixed(2)}.</p>
              <ul>${notes}</ul>
            </details>
          </td>
        </tr>
      `;
    })
    .join("");

  target.innerHTML = `
    <div class="info-card slim"><strong>${escapeHtml(dataNote)}</strong><span>This evidence confidence is not a probability of customer satisfaction.</span></div>
    <p class="muted">Saturation thresholds: ${config.total_order_target || 50} total orders, ${config.context_order_target || 10} context orders, ${config.pair_count_target || 5} pair co-orders, ${config.popularity_count_target || 10} candidate appearances.</p>
    <div class="adaptive-summary-grid">
      ${adaptiveMetric("Historical orders", profile.total_order_count || 0)}
      ${adaptiveMetric("Context orders", profile.selected_context_order_count || 0)}
      ${adaptiveMetric("Strongest pair", profile.strongest_context_pair_count || 0)}
      ${adaptiveMetric("Dataset strength", `${Math.round(Number(profile.dataset_strength || 0) * 100)}%`)}
      ${adaptiveMetric("Context strength", `${Math.round(Number(profile.context_strength || 0) * 100)}%`)}
      ${adaptiveMetric("Pair strength", `${Math.round(Number(profile.pair_strength || 0) * 100)}%`)}
      ${adaptiveMetric("Collaborative confidence", `${Math.round(collaborative * 100)}%`)}
    </div>
    <div class="admin-card inset-card"><h3>Adaptive weights</h3><div class="weight-breakdown">
      ${weightBar("Content preference", weights[0])}
      ${weightBar("Co-ordering", weights[1])}
      ${weightBar("Popularity", weights[2])}
      ${weightBar("Time/context", weights[3])}
    </div></div>
    <div class="table-wrap"><table class="responsive-data-table adaptive-result-table"><thead><tr><th>Base rank</th><th>Reranked</th><th>Dish</th><th>Base / reranked score</th><th>Diversity evidence</th><th>Confidence</th><th>Primary evidence</th><th>Weights C/CO/P/T</th><th>Details</th></tr></thead><tbody>${rows || "<tr><td colspan='9'>No eligible recommendation results.</td></tr>"}</tbody></table></div>
  `;
}

let lastCounterfactualResult = null;

function setupCounterfactualExplorer() {
  const run = document.getElementById("run-counterfactual");
  const target = document.getElementById("counterfactual-results");
  const exportButton = document.getElementById("export-counterfactual");
  if (!run || !target) return;
  const values = (id) =>
    Array.from(document.getElementById(id)?.selectedOptions || []).map((option) => option.value);

  run.addEventListener("click", async () => {
    const count = Number(document.getElementById("cf-order-count")?.value || 0);
    const diversity = document.getElementById("cf-diversity")?.value || null;
    const simulated = count > 0
      ? [{
          anchor_dish_id: document.getElementById("cf-anchor")?.value || "",
          candidate_dish_id: document.getElementById("cf-candidate")?.value || "",
          additional_order_count: count,
        }]
      : [];
    const payload = {
      baseline: {
        liked_ingredients: values("cf-base-liked"),
        disliked_ingredients: values("cf-base-disliked"),
        preferred_tags: values("cf-base-tags"),
        selected_dish_ids: values("cf-base-context"),
        ranking_method: "hybrid",
        diversity_mode: "balanced",
      },
      changes: {
        add_liked_ingredients: values("cf-add-liked"),
        remove_liked_ingredients: values("cf-remove-liked"),
        add_disliked_ingredients: values("cf-add-disliked"),
        remove_disliked_ingredients: values("cf-remove-disliked"),
        add_preferred_tags: values("cf-add-tags"),
        remove_preferred_tags: values("cf-remove-tags"),
        add_context_dish_ids: values("cf-add-context"),
        remove_context_dish_ids: values("cf-remove-context"),
        simulated_coorders: simulated,
        diversity_mode: diversity,
      },
      top_k: Number(document.getElementById("cf-top-k")?.value || 5),
    };
    run.disabled = true;
    target.innerHTML = `<div class="info-card slim"><strong>Comparing production scenarios...</strong></div>`;
    try {
      const result = await requestJson("/api/admin/recommendations/counterfactual", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!result.ok) throw new Error(result.message);
      lastCounterfactualResult = result.data;
      if (exportButton) exportButton.disabled = false;
      renderCounterfactual(result.data, target);
    } catch (error) {
      target.innerHTML = `<div class="info-card slim error"><strong>${escapeHtml(error.message || "Comparison failed.")}</strong></div>`;
    } finally {
      run.disabled = false;
    }
  });

  exportButton?.addEventListener("click", () => {
    if (!lastCounterfactualResult) return;
    const header = "dish_id,dish_name,classification,baseline_rank,changed_rank,baseline_score,changed_score,baseline_confidence,changed_confidence";
    const rows = (lastCounterfactualResult.rank_changes || []).map((item) =>
      [
        item.dish_id,
        item.dish_name,
        item.classification,
        item.baseline_rank ?? "",
        item.changed_rank ?? "",
        item.baseline_score ?? "",
        item.changed_score ?? "",
        item.baseline_confidence ?? "",
        item.changed_confidence ?? "",
      ].map(csvCell).join(",")
    );
    downloadText(
      `# Temporary counterfactual comparison; production data was not changed.\n${header}\n${rows.join("\n")}`,
      "counterfactual-comparison.csv",
      "text/csv"
    );
  });
}

function renderCounterfactual(data, target) {
  const delta = data.adaptive_weight_change || {};
  const rows = (data.rank_changes || [])
    .filter((item) => item.classification !== "unchanged")
    .slice(0, 30)
    .map((item) => `<tr>
      <td data-label="Dish"><strong>${escapeHtml(item.dish_name)}</strong><span>${escapeHtml(item.dish_id)}</span></td>
      <td data-label="Change">${escapeHtml(item.classification)}</td>
      <td data-label="Baseline rank">${item.baseline_rank ?? "-"}</td>
      <td data-label="Changed rank">${item.changed_rank ?? "-"}</td>
      <td data-label="Score">${item.baseline_score == null ? "-" : Number(item.baseline_score).toFixed(2)} → ${item.changed_score == null ? "-" : Number(item.changed_score).toFixed(2)}</td>
      <td data-label="Confidence">${item.baseline_confidence == null ? "-" : Math.round(item.baseline_confidence * 100) + "%"} → ${item.changed_confidence == null ? "-" : Math.round(item.changed_confidence * 100) + "%"}</td>
    </tr>`).join("");
  const explanation = (data.explanation || []).map((line) => `<li>${escapeHtml(line)}</li>`).join("");
  target.innerHTML = `
    <div class="info-card slim"><strong>Temporary comparison only</strong><span>No orders, timeline events, or production preferences were saved.</span></div>
    <ul>${explanation}</ul>
    <div class="adaptive-summary-grid">
      ${adaptiveMetric("Content weight change", signedPercent(delta.content_delta))}
      ${adaptiveMetric("Co-order weight change", signedPercent(delta.co_order_delta))}
      ${adaptiveMetric("Popularity change", signedPercent(delta.popularity_delta))}
      ${adaptiveMetric("Time/context change", signedPercent(delta.time_context_delta))}
      ${adaptiveMetric("Entered Top-K", (data.entered_top_k || []).length)}
      ${adaptiveMetric("Left Top-K", (data.left_top_k || []).length)}
    </div>
    <div class="table-wrap"><table class="responsive-data-table"><thead><tr><th>Dish</th><th>Change</th><th>Baseline rank</th><th>Changed rank</th><th>Score</th><th>Confidence</th></tr></thead><tbody>${rows || "<tr><td colspan='6'>No rank changed in this scenario.</td></tr>"}</tbody></table></div>
  `;
}

function signedPercent(value) {
  const percent = Math.round(Number(value || 0) * 100);
  return `${percent > 0 ? "+" : ""}${percent}%`;
}

function csvCell(value) {
  return `"${String(value ?? "").replaceAll('"', '""')}"`;
}

function downloadText(content, filename, type) {
  const url = URL.createObjectURL(new Blob([content], { type }));
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}

function setupLearningTimeline() {
  const target = document.getElementById("learning-timeline");
  const status = document.getElementById("learning-timeline-status");
  const rebuild = document.getElementById("rebuild-learning-timeline");
  const clear = document.getElementById("clear-learning-timeline");
  const reset = document.getElementById("reset-timeline-filters");
  const dialog = document.getElementById("timeline-confirm-dialog");
  const confirmButton = document.getElementById("confirm-timeline-action");
  const confirmTitle = document.getElementById("timeline-confirm-title");
  const confirmMessage = document.getElementById("timeline-confirm-message");
  if (!target || !status) return;
  let events = [];
  let pendingAction = null;

  const filterElements = {
    search: document.getElementById("timeline-search"),
    date: document.getElementById("timeline-date"),
    dish: document.getElementById("timeline-dish"),
    sort: document.getElementById("timeline-sort"),
    limit: document.getElementById("timeline-limit"),
  };

  const render = () => {
    const query = String(filterElements.search?.value || "").trim().toLowerCase();
    const date = filterElements.date?.value || "";
    const dishId = filterElements.dish?.value || "";
    const sort = filterElements.sort?.value || "newest";
    const limitValue = filterElements.limit?.value || "25";
    let visible = events.filter((event) => {
      const searchable = [
        event.historical_order_id,
        event.completed_at,
        event.summary,
        ...(event.dish_ids || []),
        ...(event.popularity_changes || []).flatMap((item) => [
          item.dish_id,
          item.dish_name,
        ]),
      ]
        .join(" ")
        .toLowerCase();
      return (
        (!query || searchable.includes(query)) &&
        (!date || String(event.completed_at || "").startsWith(date)) &&
        (!dishId || (event.dish_ids || []).includes(dishId))
      );
    });
    if (sort === "oldest") visible = visible.reverse();
    if (limitValue !== "all") visible = visible.slice(0, Number(limitValue || 25));

    if (!events.length) {
      target.innerHTML = `<div class="empty-state"><strong>No learning timeline entries are currently stored.</strong><span>Historical orders and recommendation evidence are still available. Use Rebuild Timeline to reconstruct the explanation history.</span></div>`;
      return;
    }
    target.innerHTML = visible.length
      ? visible.map(renderLearningEvent).join("")
      : `<div class="empty-state"><strong>No events match these filters.</strong><span>Reset Filters to show the stored learning history.</span></div>`;
  };

  const load = async () => {
    status.textContent = "Loading timeline...";
    try {
      const result = await requestJson("/api/admin/recommendations/learning-timeline");
      if (!result.ok) throw new Error(result.message);
      status.textContent = result.data.warning || `${result.data.event_count} learning event(s).`;
      events = result.data.events || [];
      render();
    } catch (error) {
      status.textContent = error.message || "Unable to load timeline.";
    }
  };

  const runTimelineAction = async (action) => {
    const isClear = action === "clear";
    status.textContent = isClear ? "Clearing timeline..." : "Rebuilding timeline...";
    [clear, rebuild, reset].forEach((button) => {
      if (button) button.disabled = true;
    });
    try {
      const result = await requestJson(
        isClear
          ? "/api/admin/recommendations/learning-timeline"
          : "/api/admin/recommendations/learning-timeline/rebuild",
        { method: isClear ? "DELETE" : "POST" }
      );
      if (!result.ok) throw new Error(result.message);
      if (isClear) {
        events = [];
        status.textContent = `${result.data.removed_event_count} learning event(s) removed. Historical orders were unchanged.`;
        render();
        showToast("Learning timeline cleared.");
      } else {
        events = result.data.events || [];
        status.textContent = `${result.data.event_count} learning event(s) rebuilt from historical orders.`;
        render();
        showToast("Learning timeline rebuilt.");
      }
    } catch (error) {
      // Existing rendered events are retained when a destructive operation
      // fails, so the interface never claims data was removed when it was not.
      status.textContent = error.message || `Unable to ${action} the timeline.`;
    } finally {
      [clear, rebuild, reset].forEach((button) => {
        if (button) button.disabled = false;
      });
    }
  };

  const askForConfirmation = (action) => {
    pendingAction = action;
    const isClear = action === "clear";
    if (confirmTitle) {
      confirmTitle.textContent = isClear ? "Clear learning timeline?" : "Rebuild learning timeline?";
    }
    if (confirmMessage) {
      confirmMessage.textContent = isClear
        ? "Clear all recommendation learning timeline entries? This removes only the explanatory timeline records. Historical orders and recommendation evidence will remain unchanged. You can rebuild the timeline later from historical orders."
        : "Rebuild the recommendation learning timeline from historical orders? Existing timeline entries will be replaced. Historical orders will not be changed.";
    }
    if (confirmButton) {
      confirmButton.textContent = isClear ? "Clear Timeline" : "Rebuild Timeline";
      confirmButton.classList.toggle("danger-action", isClear);
      confirmButton.classList.toggle("primary-action", !isClear);
    }
    if (typeof dialog?.showModal === "function") dialog.showModal();
  };

  clear?.addEventListener("click", () => askForConfirmation("clear"));
  rebuild?.addEventListener("click", () => askForConfirmation("rebuild"));
  confirmButton?.addEventListener("click", () => {
    const action = pendingAction;
    pendingAction = null;
    dialog?.close();
    if (action) runTimelineAction(action);
  });
  reset?.addEventListener("click", () => {
    if (filterElements.search) filterElements.search.value = "";
    if (filterElements.date) filterElements.date.value = "";
    if (filterElements.dish) filterElements.dish.value = "";
    if (filterElements.sort) filterElements.sort.value = "newest";
    if (filterElements.limit) filterElements.limit.value = "25";
    render();
    target.querySelectorAll("details[open]").forEach((details) => {
      details.open = false;
    });
    status.textContent = `${events.length} learning event(s). Filters reset; no events were deleted.`;
  });
  Object.values(filterElements).forEach((element) => {
    element?.addEventListener("input", render);
    element?.addEventListener("change", render);
  });
  load();
}

function renderLearningEvent(event) {
  const popularity = (event.popularity_changes || [])
    .map((item) => `<li>${escapeHtml(item.dish_name)} (${escapeHtml(item.dish_id)}): ${item.before_count} → ${item.after_count}</li>`)
    .join("");
  const pairs = (event.pair_changes || [])
    .slice(0, 5)
    .map((item) => `<li><strong>${escapeHtml(item.dish_a_name)} + ${escapeHtml(item.dish_b_name)}</strong>: pair ${item.before_count} → ${item.after_count}, confidence ${Number(item.confidence_a_to_b_before).toFixed(2)} → ${Number(item.confidence_a_to_b_after).toFixed(2)}, lift ${Number(item.lift_before).toFixed(2)} → ${Number(item.lift_after).toFixed(2)}</li>`)
    .join("");
  const ranks = (event.rank_changes || [])
    .slice(0, 5)
    .map((item) => `<li>${escapeHtml(item.candidate_dish_id)} for ${escapeHtml(item.anchor_dish_id)}: ${item.before_rank ?? "-"} → ${item.after_rank ?? "-"}</li>`)
    .join("");
  return `
    <article class="learning-event">
      <div class="section-heading"><div><h3>Order ${escapeHtml(event.historical_order_id)} completed</h3><p>${escapeHtml(event.completed_at)} · ${escapeHtml((event.dish_ids || []).join(", "))}</p></div><strong>${event.total_orders_before} → ${event.total_orders_after} orders</strong></div>
      <p>${escapeHtml(event.summary)}</p>
      <details><summary>Evidence changes</summary><h4>Popularity</h4><ul>${popularity || "<li>-</li>"}</ul><h4>Co-order pairs</h4><ul>${pairs || "<li>-</li>"}</ul><h4>Ranks</h4><ul>${ranks || "<li>-</li>"}</ul></details>
    </article>
  `;
}

function adaptiveMetric(label, value) {
  return `<div class="metric-card"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`;
}

function setupAdminOrderStatus() {
  const statusMessage = document.getElementById("admin-order-status");
  document.querySelectorAll("[data-order-status]").forEach((select) => {
    select.addEventListener("change", async () => {
      const orderId = select.dataset.orderStatus;
      select.disabled = true;
      try {
        const result = await requestJson(
          `/api/admin/orders/${encodeURIComponent(orderId)}/status`,
          {
            method: "PATCH",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ status: select.value }),
          }
        );
        if (statusMessage) {
          statusMessage.textContent = result.message || "";
        }
        if (result.ok && (select.value === "Completed" || select.value === "Cancelled")) {
          // Completed orders are written before this success path returns.
          // Reloading refreshes active and historical tables together.
          window.setTimeout(() => window.location.reload(), 900);
        }
      } catch (error) {
        if (statusMessage) {
          statusMessage.textContent = error.message || "Unable to update order status.";
        }
      } finally {
        select.disabled = false;
      }
    });
  });
}

function setupAdminOrderPolling() {
  const body = document.getElementById("admin-live-orders-body");
  const statusMessage = document.getElementById("admin-order-status");
  if (!body || body.dataset.pollingBound === "true") return;
  body.dataset.pollingBound = "true";
  let lastVersion = null;
  const load = async () => {
    try {
      const result = await requestJson("/api/admin/orders");
      if (!result.ok || !result.data) {
        if (statusMessage) statusMessage.textContent = result.message || "Unable to refresh orders.";
        return;
      }
      if (lastVersion === result.data.version) return;
      const hadVersion = lastVersion !== null;
      lastVersion = result.data.version;
      const active = (result.data.orders || []).filter((order) => !["Completed", "Cancelled"].includes(String(order.status)));
      body.innerHTML = active.length
        ? active.map(renderAdminOrderRow).join("")
        : `<tr><td colspan="11">No active live customer orders yet.</td></tr>`;
      setupAdminOrderStatus();
      if (hadVersion && statusMessage) statusMessage.textContent = `Order list updated: ${result.data.updated_at}`;
    } catch (error) {
      if (statusMessage) {
        statusMessage.textContent = error.message || "Unable to refresh orders.";
      }
    }
  };
  load();
  window.setInterval(() => {
    if (!document.hidden) load();
  }, 4000);
}

function renderAdminOrderRow(order) {
  const statuses = ["Pending", "Preparing", "Ready", "Completed", "Cancelled"];
  return `
    <tr class="admin-live-order-row">
      <td data-label="Order ID">${escapeHtml(order.order_id)}</td>
      <td data-label="Session">${escapeHtml(order.session_user_id)}</td>
      <td data-label="Customer">${escapeHtml(order.customer_name || "-")}</td>
      <td data-label="Phone">${escapeHtml(order.customer_phone || "-")}</td>
      <td data-label="Table">${escapeHtml(order.table_number || "-")}</td>
      <td data-label="Dish IDs">${escapeHtml((order.ordered_dishes || []).join(", "))}</td>
      <td data-label="Dish Names">${escapeHtml((order.dish_names || []).join(", "))}</td>
      <td data-label="Note">${escapeHtml(order.note || "-")}</td>
      <td data-label="Time">${escapeHtml(order.timestamp || "-")}</td>
      <td data-label="Total">${escapeHtml(order.total_price || "-")}</td>
      <td data-label="Status"><select data-order-status="${escapeHtml(order.order_id)}">${statuses.map((status) => `<option value="${status}" ${status === order.status ? "selected" : ""}>${status}</option>`).join("")}</select></td>
    </tr>
  `;
}

function setupDishManagement() {
  const form = document.getElementById("dish-form");
  const status = document.getElementById("dish-management-status");
  const modal = document.getElementById("dish-form-modal");
  const title = document.getElementById("dish-form-title");
  const search = document.getElementById("admin-dish-search");
  const availability = document.getElementById("admin-availability-filter");
  const tableBody = document.getElementById("admin-dish-table-body");
  const emptyState = document.getElementById("admin-dish-empty");
  if (!form || !modal || !tableBody) return;

  const setStatus = (message, isError = false) => {
    if (!status) return;
    status.textContent = message || "";
    status.classList.toggle("error", isError);
  };

  const openForm = (mode, dish = null) => {
    form.reset();
    form.dataset.mode = mode;
    if (title) title.textContent = mode === "edit" ? "Edit Dish" : "Add Dish";
    if (form.elements.dish_id) {
      // Dish IDs connect menu rows with historical order logs. Edit mode keeps
      // the ID visible but read-only so staff can change dish details without
      // accidentally breaking existing co-order evidence.
      form.elements.dish_id.readOnly = mode === "edit";
    }
    const submitButton = form.querySelector('button[type="submit"]');
    if (submitButton) {
      submitButton.textContent = mode === "edit" ? "Save Changes" : "Add Dish";
    }
    form.elements.available.checked = true;
    if (dish) {
      form.elements.dish_id.value = dish.dish_id || "";
      form.elements.name.value = dish.name || "";
      form.elements.price.value = dish.price_amount || "";
      form.elements.category.value = dish.category || "";
      form.elements.ingredients.value = (dish.ingredients || []).join(", ");
      form.elements.tags.value = (dish.tags || []).join(", ");
      form.elements.image_path.value = dish.image_path || "";
      form.elements.available.checked = dish.available !== false;
    }
    modal.hidden = false;
  };

  document.getElementById("open-dish-form")?.addEventListener("click", () => openForm("add"));
  document.getElementById("cancel-dish-form")?.addEventListener("click", () => {
    modal.hidden = true;
  });

  const filterRows = () => {
    const term = (search?.value || "").trim().toLowerCase();
    const mode = availability?.value || "all";
    let visible = 0;
    document.querySelectorAll("[data-admin-dish-row]").forEach((row) => {
      const haystack = [
        row.dataset.adminDishRow,
        row.dataset.dishName,
        row.dataset.dishCategory,
        row.dataset.dishIngredients,
        row.dataset.dishTags,
      ].join(" ").toLowerCase();
      const availableText = row.dataset.dishAvailable === "true" ? "available" : "unavailable";
      row.hidden = (term && !haystack.includes(term)) || (mode !== "all" && mode !== availableText);
      if (!row.hidden) visible += 1;
    });
    if (emptyState) emptyState.hidden = visible !== 0;
  };
  search?.addEventListener("input", filterRows);
  availability?.addEventListener("change", filterRows);

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    const formData = new FormData(form);
    const payload = Object.fromEntries(formData.entries());
    payload.available = formData.has("available");
    const isEdit = form.dataset.mode === "edit";
    const dishId = String(payload.dish_id || "").trim();
    const endpoint = isEdit
      ? `/api/admin/dishes/${encodeURIComponent(dishId)}`
      : "/api/admin/dishes";
    const submitButton = form.querySelector('button[type="submit"]');
    submitButton.disabled = true;
    try {
      const result = await requestJson(endpoint, {
        method: isEdit ? "PUT" : "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      setStatus(result.message, !result.ok);
      if (result.ok) {
        modal.hidden = true;
        // The server is the source of truth for image lookup and display
        // pricing. Reloading after a successful mutation keeps the row markup
        // consistent while all interaction bindings remain delegated.
        window.setTimeout(() => window.location.reload(), 450);
      }
    } catch {
      setStatus("Dish request failed. Please try again.", true);
    } finally {
      submitButton.disabled = false;
    }
  });

  // Event delegation keeps actions working even when rows are replaced after
  // a future in-place refresh or import.
  tableBody.addEventListener("click", async (event) => {
    const button = event.target.closest("button");
    if (!button) return;

    if (button.matches("[data-edit-dish]")) {
      setStatus("Loading dish...");
      try {
        const result = await requestJson(
          `/api/admin/dishes/${encodeURIComponent(button.dataset.editDish)}`,
          { cache: "no-store" }
        );
        if (!result.ok || !result.data) {
          setStatus(result.message || "Dish could not be loaded.", true);
          return;
        }
        setStatus("");
        openForm("edit", result.data);
      } catch {
        setStatus("Dish could not be loaded.", true);
      }
      return;
    }

    if (button.matches("[data-toggle-dish]")) {
      button.disabled = true;
      try {
        const result = await requestJson(
          `/api/admin/dishes/${encodeURIComponent(button.dataset.toggleDish)}/availability`,
          {
            method: "PATCH",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ available: button.dataset.available === "true" }),
          }
        );
        setStatus(result.message, !result.ok);
        if (result.ok) {
          window.setTimeout(() => window.location.reload(), 500);
        }
      } catch (error) {
        setStatus(error.message || "Availability update failed.", true);
      } finally {
        button.disabled = false;
      }
      return;
    }

    if (button.matches("[data-delete-dish]")) {
      if (!window.confirm("Permanently remove this dish from the in-memory menu? Dishes referenced by historical orders cannot be deleted.")) {
        return;
      }
      button.disabled = true;
      try {
        const result = await requestJson(
          `/api/admin/dishes/${encodeURIComponent(button.dataset.deleteDish)}`,
          { method: "DELETE" }
        );
        setStatus(result.message, !result.ok);
        if (result.ok) {
          button.closest("[data-admin-dish-row]")?.remove();
          filterRows();
        }
      } catch (error) {
        setStatus(error.message || "Dish deletion failed.", true);
      } finally {
        button.disabled = false;
      }
    }
  });
}

async function postJson(url, payload) {
  return requestJson(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
}

async function setupCustomerOrderStatus() {
  const target = document.getElementById("orders-list");
  if (!target) {
    return;
  }

  let lastVersion = null;
  let timer = null;
  const indicator = document.getElementById("order-sync-status");
  const loadOrders = async () => {
    let result = null;
    try {
      result = await requestJson("/api/customer/orders", { cache: "no-store" });
    } catch (error) {
      console.error("Customer order polling failed", error);
      if (indicator) indicator.textContent = "Reconnecting to order status...";
      return;
    }

    if (!result.ok || !result.data) {
      if (!lastVersion) {
        target.innerHTML = `<div class="info-card"><strong>No current orders yet.</strong><span>${escapeHtml(result.message)}</span></div>`;
      }
      if (indicator) indicator.textContent = result.message || "Unable to refresh order status.";
      return;
    }
    if (!(result.data.orders || []).length) {
      target.innerHTML = `<div class="info-card"><strong>No current orders yet.</strong><span>Place an order from the cart to track it here.</span></div>`;
      return;
    }
    if (lastVersion === result.data.version) return;
    lastVersion = result.data.version;

    target.innerHTML = result.data.orders.map(renderOrderCard).join("");
    if (indicator) indicator.textContent = `Last updated: ${escapeHtml(result.data.updated_at)}`;
    setupOrderFilters();
  };

  const schedule = () => {
    window.clearTimeout(timer);
    timer = window.setTimeout(async () => {
      if (!document.hidden) await loadOrders();
      schedule();
    }, document.hidden ? 12000 : 3000);
  };

  await loadOrders();
  schedule();
  document.addEventListener("visibilitychange", () => {
    if (!document.hidden) {
      loadOrders();
    }
    schedule();
  });
}

function renderOrderCard(order) {
  const status = String(order.status || "Pending");
  const statusLower = status.toLowerCase();
  const statusMessages = {
    Pending: "Your order has been received.",
    Preparing: "The kitchen is preparing your food.",
    Ready: "Your food is ready.",
    Completed: "Your order is completed.",
    Cancelled: "Your order was cancelled. Please contact staff.",
  };
  return `
    <article class="order-card" data-order-status-card="${escapeHtml(statusLower)}">
      <div class="order-card-main">
        <div>
          <p class="eyebrow">${escapeHtml(order.order_id)}</p>
          <h2>${escapeHtml(status)} · ${escapeHtml(order.total_price)}</h2>
        </div>
        <span class="status-badge ${escapeHtml(statusLower)}">${escapeHtml(status)}</span>
      </div>
      <div class="order-progress" aria-label="Order progress">${renderOrderSteps(status)}</div>
      <p class="order-status-explanation">${escapeHtml(statusMessages[status] || "Order status updated.")}</p>
      <p><strong>Customer:</strong> ${escapeHtml(order.customer_name || "-")}</p>
      <p><strong>Dishes:</strong> ${escapeHtml((order.dish_names || []).join(", "))}</p>
      <p><strong>Time:</strong> ${escapeHtml(order.timestamp || "-")}</p>
      ${order.table_number ? `<p><strong>Table:</strong> ${escapeHtml(order.table_number)}</p>` : ""}
    </article>
  `;
}

function renderOrderSteps(status) {
  const steps = ["Pending", "Preparing", "Ready", "Completed"];
  if (status === "Cancelled") {
    return `<span class="progress-step active cancelled">Cancelled</span>`;
  }
  const activeIndex = Math.max(0, steps.indexOf(status));
  return steps
    .map((step, index) => `<span class="progress-step ${index <= activeIndex ? "active" : ""}">${escapeHtml(step)}</span>`)
    .join("");
}

function setupAdminRecommendationTester() {
  const tabs = Array.from(document.querySelectorAll("[data-experiment-tab]"));
  if (!tabs.length) return;

  const activateTab = (name, focus = false) => {
    tabs.forEach((tab) => {
      const active = tab.dataset.experimentTab === name;
      tab.classList.toggle("active", active);
      tab.setAttribute("aria-selected", String(active));
      tab.tabIndex = active ? 0 : -1;
      if (active && focus) tab.focus();
    });
    document.querySelectorAll("[data-experiment-panel]").forEach((panel) => {
      panel.hidden = panel.dataset.experimentPanel !== name;
    });
  };

  tabs.forEach((tab, index) => {
    tab.addEventListener("click", () => activateTab(tab.dataset.experimentTab));
    tab.addEventListener("keydown", (event) => {
      if (!["ArrowLeft", "ArrowRight"].includes(event.key)) return;
      event.preventDefault();
      const offset = event.key === "ArrowRight" ? 1 : -1;
      const next = tabs[(index + offset + tabs.length) % tabs.length];
      activateTab(next.dataset.experimentTab, true);
    });
  });
  activateTab("ingredient");

  const selectedValues = (kind) =>
    Array.from(document.querySelectorAll(`[data-experiment-option="${kind}"]:checked`)).map(
      (item) => item.value
    );
  const clearChecks = (prefix) => {
    document.querySelectorAll(`[data-experiment-option^="${prefix}"]`).forEach((item) => {
      item.checked = false;
    });
  };

  document.querySelectorAll("[data-experiment-option-search]").forEach((search) => {
    search.addEventListener("input", () => {
      const kind = search.dataset.experimentOptionSearch;
      const term = search.value.trim().toLowerCase();
      document
        .querySelectorAll(`[data-experiment-option-list="${kind}"] .ingredient-option`)
        .forEach((option) => {
          option.hidden = Boolean(term) && !String(option.dataset.optionLabel || "").includes(term);
        });
    });
  });

  // Liked and disliked selections are mutually exclusive in each experiment.
  document.querySelectorAll("[data-experiment-option]").forEach((input) => {
    input.addEventListener("change", () => {
      if (!input.checked) return;
      const [scope, side] = input.dataset.experimentOption.split("-");
      const opposite = side === "liked" ? "disliked" : "liked";
      document
        .querySelector(
          `[data-experiment-option="${scope}-${opposite}"][value="${CSS.escape(input.value)}"]`
        )
        ?.removeAttribute("checked");
      const oppositeInput = document.querySelector(
        `[data-experiment-option="${scope}-${opposite}"][value="${CSS.escape(input.value)}"]`
      );
      if (oppositeInput) oppositeInput.checked = false;
    });
  });

  document.querySelectorAll("[data-ingredient-preset]").forEach((button) => {
    button.addEventListener("click", () => {
      clearChecks("ingredient-");
      if (button.dataset.ingredientPreset === "all") {
        document.querySelectorAll('[data-experiment-option="ingredient-liked"]').forEach((item) => {
          item.checked = true;
        });
      }
      if (button.dataset.ingredientPreset === "example") {
        ["chicken", "rice"].forEach((value) => {
          const input = document.querySelector(
            `[data-experiment-option="ingredient-liked"][value="${CSS.escape(value)}"]`
          );
          if (input) input.checked = true;
        });
        ["beef", "anchovies"].forEach((value) => {
          const input = document.querySelector(
            `[data-experiment-option="ingredient-disliked"][value="${CSS.escape(value)}"]`
          );
          if (input) input.checked = true;
        });
      }
    });
  });

  const historicalOrder = document.getElementById("method-historical-order");
  const hiddenDish = document.getElementById("method-hidden-dish");
  const populateHiddenTargets = () => {
    if (!historicalOrder || !hiddenDish) return;
    const selected = historicalOrder.selectedOptions[0];
    const ids = String(selected?.dataset.dishIds || "")
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean);
    hiddenDish.innerHTML = ids.length
      ? `<option value="">Select hidden target</option>${ids
          .map((id) => {
            const dish = (window.MENU_DISHES || []).find((item) => item.dish_id === id);
            return `<option value="${escapeHtml(id)}">${escapeHtml(dish?.name || id)} (${escapeHtml(id)})</option>`;
          })
          .join("")}`
      : `<option value="">Select an order first</option>`;
    hiddenDish.disabled = !ids.length;
  };
  historicalOrder?.addEventListener("change", populateHiddenTargets);

  const payloadFor = (type) => {
    if (type === "ingredient") {
      return {
        experiment_type: type,
        liked_ingredients: selectedValues("ingredient-liked"),
        disliked_ingredients: selectedValues("ingredient-disliked"),
        top_k: Number(document.getElementById("ingredient-top-k")?.value || 5),
      };
    }
    if (type === "coorder") {
      const anchor = document.getElementById("coorder-anchor-dish")?.value || "";
      const candidate = document.getElementById("coorder-candidate-dish")?.value || "";
      if (!anchor) throw new Error("Select an anchor dish.");
      if (!candidate) throw new Error("Select a candidate dish.");
      if (anchor === candidate) throw new Error("Anchor and candidate must be different.");
      return {
        experiment_type: type,
        anchor_dish_id: anchor,
        candidate_dish_id: candidate,
        additional_coorders: Number(document.getElementById("coorder-additional")?.value || 0),
        top_k: Number(document.getElementById("coorder-top-k")?.value || 5),
      };
    }
    const orderId = historicalOrder?.value || "";
    const target = hiddenDish?.value || "";
    if (!orderId) throw new Error("Select a historical order.");
    if (!target) throw new Error("Select a hidden target dish.");
    return {
      experiment_type: type,
      historical_order_id: orderId,
      hidden_dish_id: target,
      liked_ingredients: selectedValues("method-liked"),
      disliked_ingredients: selectedValues("method-disliked"),
      top_k: Number(document.getElementById("method-top-k")?.value || 5),
    };
  };

  document.querySelectorAll("[data-run-experiment]").forEach((button) => {
    button.addEventListener("click", async () => {
      const type = button.dataset.runExperiment;
      const target = document.getElementById(`experiment-result-${type}`);
      if (!target) return;
      let payload;
      try {
        payload = payloadFor(type);
      } catch (error) {
        target.innerHTML = experimentMessage(error.message, true);
        return;
      }
      button.disabled = true;
      const original = button.textContent;
      button.textContent = "Running...";
      target.innerHTML = experimentMessage("Running controlled experiment...");
      try {
        const result = await requestJson("/api/admin/experiment-lab", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });
        if (!result.ok || !result.data) {
          target.innerHTML = experimentMessage(
            `The experiment request failed: ${result.message || "Unknown error."}`,
            true
          );
          return;
        }
        target.innerHTML = renderExperimentResult(type, result.data);
      } catch {
        target.innerHTML = experimentMessage("The experiment request failed: unable to reach the server.", true);
      } finally {
        button.disabled = false;
        button.textContent = original;
      }
    });
  });

  document.querySelectorAll("[data-clear-experiment]").forEach((button) => {
    button.addEventListener("click", () => {
      const target = document.getElementById(`experiment-result-${button.dataset.clearExperiment}`);
      if (target) target.innerHTML = "";
    });
  });

  document.querySelectorAll("[data-reset-experiment]").forEach((button) => {
    button.addEventListener("click", () => {
      const type = button.dataset.resetExperiment;
      if (type === "ingredient") {
        clearChecks("ingredient-");
        document.getElementById("ingredient-top-k").value = "5";
      } else if (type === "coorder") {
        document.getElementById("coorder-anchor-dish").value = "";
        document.getElementById("coorder-candidate-dish").value = "";
        document.getElementById("coorder-additional").value = "10";
        document.getElementById("coorder-top-k").value = "5";
      } else {
        clearChecks("method-");
        historicalOrder.value = "";
        document.getElementById("method-top-k").value = "5";
        populateHiddenTargets();
      }
      if (type === "ingredient" || type === "method") {
        document
          .querySelectorAll(`[data-experiment-option-search^="${type}-"]`)
          .forEach((search) => {
            search.value = "";
            search.dispatchEvent(new Event("input"));
          });
      }
      const target = document.getElementById(`experiment-result-${type}`);
      if (target) target.innerHTML = "";
    });
  });
}

function experimentMessage(message, isError = false) {
  return `<div class="info-card slim ${isError ? "error" : ""}"><strong>${escapeHtml(message)}</strong></div>`;
}

function renderExperimentResult(type, data) {
  const rows = data.rows || [];
  let table = "";
  if (type === "ingredient") {
    const dishes = new Map();
    rows.forEach((row) => {
      const item = dishes.get(row.dish_id) || { dish_id: row.dish_id, dish_name: row.dish_name };
      if (String(row.method).startsWith("Before")) item.before = row;
      else item.after = row;
      dishes.set(row.dish_id, item);
    });
    const body = Array.from(dishes.values())
      .map((item) => {
        const before = item.before?.rank ?? "-";
        const after = item.after?.rank ?? "-";
        const wasHardExcluded =
          after === "-" && item.before?.excluded && item.before.excluded !== "-";
        const change =
          before === "-" ? "New in Top-K" :
          wasHardExcluded ? "Excluded" :
          after === "-" ? "Left Top-K" :
          before > after ? `↑ ${before - after} position(s)` :
          before < after ? `↓ ${after - before} position(s)` : "No change";
        const detail = item.after
          ? item.after.matched || "-"
          : wasHardExcluded
            ? item.before.excluded
            : "No longer in Top-K";
        return `<tr><td>${escapeHtml(item.dish_name)} (${escapeHtml(item.dish_id)})</td><td>${before}</td><td>${after}</td><td>${item.after ? Number(item.after.ingredient_score).toFixed(2) : "-"}</td><td>${escapeHtml(change)}</td><td>${escapeHtml(detail)}</td></tr>`;
      })
      .join("");
    table = experimentTable(
      ["Dish", "Before rank", "After rank", "Ingredient score", "Change", "Matched / exclusion"],
      body
    );
  } else if (type === "coorder") {
    const before = rows[0] || {};
    const after = rows[1] || {};
    const beforeMetrics = parseCoorderMetrics(before.matched);
    const afterMetrics = parseCoorderMetrics(after.matched);
    const metrics = [
      ["Pair count", beforeMetrics.pairCount, afterMetrics.pairCount],
      ["Co-order score", Number(before.co_order_score || 0), Number(after.co_order_score || 0)],
      ["Support", beforeMetrics.support, afterMetrics.support],
      ["Confidence", beforeMetrics.confidence, afterMetrics.confidence],
      ["Lift", beforeMetrics.lift, afterMetrics.lift],
      ["Candidate rank", before.rank ?? "-", after.rank ?? "-"],
    ];
    const body = metrics
      .map(([label, first, second]) => {
        const numeric = Number.isFinite(first) && Number.isFinite(second);
        const change = numeric ? second - first : "-";
        const format = (value) =>
          Number.isFinite(value)
            ? value.toFixed(label === "Pair count" || label === "Candidate rank" ? 0 : 2)
            : "-";
        return `<tr><td>${escapeHtml(label)}</td><td>${format(first)}</td><td>${format(second)}</td><td>${format(change)}</td></tr>`;
      })
      .join("");
    table = experimentTable(["Metric", "Before", "After", "Change"], body);
  } else {
    const methods = [...new Set(rows.map((row) => row.method))];
    const methodTables = methods
      .map((method) => {
        const body = rows
          .filter((row) => row.method === method)
          .map((row) => `<tr class="${row.hidden_match ? "highlight-row" : ""}"><td>${row.rank ?? "-"}</td><td>${escapeHtml(row.dish_id)}</td><td>${escapeHtml(row.dish_name)}</td><td>${Number(row.final_score).toFixed(2)}</td><td>${row.hidden_match ? "Hidden target" : "-"}</td></tr>`)
          .join("");
        return `<h4>${escapeHtml(method)}</h4>${experimentTable(["Rank", "Dish ID", "Dish", "Score", "Target"], body)}`;
      })
      .join("");
    const summaryRows = methods
      .map((method) => {
        const methodRows = rows.filter((row) => row.method === method);
        const target = methodRows.find((row) => row.hidden_match);
        const details = String(methodRows[0]?.matched || "");
        const matchRate = details.match(/preference match rate:\s*([0-9.]+)/i)?.[1] || "-";
        const violations = details.match(/violations:\s*(\d+)/i)?.[1] || "-";
        return `<tr><td>${escapeHtml(method)}</td><td>${target ? "Yes" : "No"}</td><td>${target?.rank ?? "-"}</td><td>${escapeHtml(matchRate)}</td><td>${escapeHtml(violations)}</td></tr>`;
      })
      .join("");
    table = `${methodTables}<h4>Method summary</h4>${experimentTable(
      ["Method", "Hit@K", "Hidden rank", "Preference match rate", "Violations"],
      summaryRows
    )}`;
  }
  return `${table}<div class="reason-box"><strong>Conclusion</strong><p>${escapeHtml(data.conclusion || "-")}</p><p>Production data and weights were not changed.</p></div>`;
}

function parseCoorderMetrics(text) {
  const value = String(text || "");
  const number = (pattern) => {
    const parsed = Number(value.match(pattern)?.[1] || 0);
    return Number.isFinite(parsed) ? parsed : 0;
  };
  return {
    pairCount: number(/Pair count:\s*(\d+)/i),
    support: number(/support\s*([0-9]+(?:\.[0-9]+)?)/i),
    confidence: number(/confidence\s*([0-9]+(?:\.[0-9]+)?)/i),
    lift: number(/lift\s*([0-9]+(?:\.[0-9]+)?)/i),
  };
}

function experimentTable(headers, body) {
  return `<div class="table-wrap"><table><thead><tr>${headers
    .map((header) => `<th>${escapeHtml(header)}</th>`)
    .join("")}</tr></thead><tbody>${body || `<tr><td colspan="${headers.length}">No result rows.</td></tr>`}</tbody></table></div>`;
}

function setupSimulationTester() {
  const runButton = document.getElementById("run-simulation");
  const resetButton = document.getElementById("reset-simulation");
  const target = document.getElementById("simulation-results");
  const forcedA = document.getElementById("simulation-forced-a");
  const forcedB = document.getElementById("simulation-forced-b");
  if (!runButton || !target) {
    return;
  }

  const forcedOptions = [`<option value="">No forced pair</option>`]
    .concat(
      (window.MENU_DISHES || []).map(
        (dish) => `<option value="${escapeHtml(dish.dish_id)}">${escapeHtml(dish.name)} (${escapeHtml(dish.dish_id)})</option>`
      )
    )
    .join("");
  if (forcedA) forcedA.innerHTML = forcedOptions;
  if (forcedB) forcedB.innerHTML = forcedOptions;

  runButton.addEventListener("click", async () => {
    target.innerHTML = `<div class="info-card slim"><strong>Running simulation...</strong><span>Generated orders are in memory only.</span></div>`;
    const preferences = collectPreferences("admin");
    const payload = {
      ...preferences,
      order_count: Number(document.getElementById("simulation-order-count")?.value || 20),
      min_dishes: Number(document.getElementById("simulation-min-dishes")?.value || 2),
      max_dishes: Number(document.getElementById("simulation-max-dishes")?.value || 4),
      seed: Number(document.getElementById("simulation-seed")?.value || 42),
      popularity_skew: document.getElementById("simulation-skew")?.value || "uniform",
      forced_dish_a: forcedA?.value || null,
      forced_dish_b: forcedB?.value || null,
      pair_probability: Number(document.getElementById("simulation-pair-probability")?.value || 35),
    };
    try {
      const result = await requestJson("/api/admin/simulation", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!result.ok) {
        target.innerHTML = `<div class="info-card slim"><strong>Simulation failed</strong><span>${escapeHtml(result.message)}</span></div>`;
        return;
      }
      renderSimulationResults(result.data, target);
    } catch (error) {
      console.error("Simulation failed", error);
      target.innerHTML = `<div class="info-card slim"><strong>Simulation failed</strong><span>Please try again.</span></div>`;
    }
  });

  resetButton?.addEventListener("click", () => {
    target.innerHTML = `<div class="info-card slim"><strong>Simulation reset</strong><span>The tester is using the real historical order dataset only.</span></div>`;
  });
}

function renderSimulationResults(data, target) {
  const preview = (data.preview || [])
    .map((order) => `<li><strong>${escapeHtml(order.order_id)}</strong>: ${escapeHtml((order.dish_names || order.dish_ids || []).join(", "))}</li>`)
    .join("");
  const pairs = (data.changed_pairs || [])
    .map(
      (pair) => `<tr><td>${escapeHtml(pair.label)}</td><td>${pair.before_count}</td><td>${pair.after_count}</td><td>${Number(pair.support_before).toFixed(2)}</td><td>${Number(pair.support_after).toFixed(2)}</td></tr>`
    )
    .join("");
  const ranks = (data.rank_changes || [])
    .map(
      (item) => `<tr><td>${escapeHtml(item.dish_name)} (${escapeHtml(item.dish_id)})</td><td>${item.before_rank ?? "-"}</td><td>${item.after_rank ?? "-"}</td><td>${Number(item.before_score).toFixed(2)}</td><td>${Number(item.after_score).toFixed(2)}</td><td>${escapeHtml(item.explanation)}</td></tr>`
    )
    .join("");

  target.innerHTML = `
    <div class="info-card slim"><strong>${escapeHtml(data.note || "Simulation completed.")}</strong><span>${data.generated_order_count || 0} generated basket(s).</span></div>
    <details open><summary>Generated order preview</summary><ul class="simulation-preview">${preview || "<li>No generated orders.</li>"}</ul></details>
    <details open><summary>Top changed co-order pairs</summary><div class="table-wrap"><table><thead><tr><th>Pair</th><th>Before</th><th>After</th><th>Support before</th><th>Support after</th></tr></thead><tbody>${pairs || "<tr><td colspan='5'>No pair changes found.</td></tr>"}</tbody></table></div></details>
    <details open><summary>Recommendation rank changes</summary><div class="table-wrap"><table><thead><tr><th>Dish</th><th>Before rank</th><th>After rank</th><th>Before score</th><th>After score</th><th>Explanation</th></tr></thead><tbody>${ranks || "<tr><td colspan='6'>No rank changes found.</td></tr>"}</tbody></table></div></details>
  `;
}

function setupAdminInsights() {
  const summary = document.getElementById("admin-insight-summary");
  const grid = document.getElementById("admin-insight-grid");
  const refresh = document.getElementById("refresh-admin-insights");
  if (!summary || !grid) {
    return;
  }

  const renderList = (title, values) => `
    <div class="insight-card">
      <strong>${escapeHtml(title)}</strong>
      ${
        values && values.length
          ? `<ul>${values.map((value) => `<li>${escapeHtml(value)}</li>`).join("")}</ul>`
          : `<p class="muted">No data yet.</p>`
      }
    </div>
  `;

  const loadInsights = async () => {
    summary.textContent = "Loading insights...";
    try {
      const result = await requestJson("/api/admin/insights");
      summary.textContent = result.summary || "";
      grid.innerHTML =
        renderList("Popular dishes", result.popular || []) +
        renderList("Co-order patterns", result.co_order_pairs || []) +
        renderList("Low exposure dishes", result.low_exposure || []);
    } catch {
      summary.textContent = "Unable to load insight summary.";
    }
  };

  refresh?.addEventListener("click", loadInsights);
  loadInsights();
}

document.addEventListener("DOMContentLoaded", () => {
  updateCartCount();
  setupDishLocator();
  setupCarouselControls();
  setupDragScrolling();
  setupPageDragScrolling();
  setupOrderFilters();
  setupPreferencePanels();
  setupDiversitySelector();
  setupMealSetBuilder();
  setupCartButtons();
  setupDetailButtons();
  renderCartPage();
  setupCheckout();
  setupSmartMenuAssistant();
  setupAdminTools();
  setupCustomerOrderStatus();
  refreshCustomerRecommendations();
});
