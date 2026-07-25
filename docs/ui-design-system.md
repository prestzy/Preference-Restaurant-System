# UI Design System

## Direction

The interface uses Catppuccin Latte-inspired semantic tokens with a calm,
high-contrast restaurant application layout. Customer pages are mobile-first;
admin pages use responsive cards and scroll-contained tables.

## Principles

- The customer Menu is permanent and complete. Search is a locator only.
- Core actions are visually clear and at least 44px on touch devices.
- Orange is not the system palette; semantic Catppuccin tokens define accents,
  surfaces, status, and focus.
- Cards are individual content units, not nested page-section decoration.
- Long tables scroll inside their own container, never the whole page.
- Loading, empty, success, error, disabled, selected, and focus states are
  explicit.

## Semantic Tokens

`static/app.css` defines the source of truth. Contributors should use variables
for:

- page and elevated surfaces;
- primary/secondary text;
- borders;
- accent and accent hover;
- success, warning, and danger;
- spacing;
- card/control radius;
- shadow; and
- visible keyboard focus.

Do not add hardcoded one-off colours when an existing semantic token represents
the meaning.

## Layout

- Mobile first from 360px.
- Main content tracks use `minmax(0, 1fr)` to permit shrinking.
- Horizontal recommendation rails own their `overflow-x`.
- Category and option chips can scroll/wrap inside their section.
- The bottom navigation remains reachable without covering interactive content.
- Tablet/desktop breakpoints add columns only when content remains readable.

## Components

### Dish Card

Shows local image/placeholder, name, category, tags, ingredient preview, price,
details action, and Add action. The permanent Menu card has a stable
`id="dish-Dxx"` target for search location/highlight.

### Recommendation Card

Adds recommendation reason, score/evidence label, and recommendation badge. It
does not replace or filter Menu cards.

### Chips

Use buttons or form controls with selected state and focus visibility. Liked and
disliked ingredients are mutually exclusive in application state.

### Tables

Wrap wide admin tables in the shared scroll container. Actions use a vertical or
wrapped gap so Edit, availability, and Delete icons do not touch.

### Modal

Use native dialog where supported, label it with the dish title, move focus
inside when opened, close on the standard control/Escape, and return focus to
the trigger.

### Feedback

Use inline errors and non-blocking toast/status regions. Fetch controls restore
their loading state in `finally`. Avoid browser `alert()` as the primary
mechanism.

## Accessibility

- Icon-only buttons require `aria-label`.
- Tabs require `role=tab`, `aria-selected`, and `aria-controls`.
- Form controls require visible labels.
- Status/error regions should use appropriate live announcement.
- Do not remove focus outlines without a visible replacement.
- Images require useful alt text or a decorative empty alt.
- Colour must not be the only status indicator.

## Responsive Verification

Test:

- 360x800
- 390x844
- 414x896
- 768x1024
- 820x1180
- 1024x1366
- desktop

At each width check page horizontal overflow, search dropdown containment,
recommendation rail scrolling, card text wrapping, preference option spacing,
Cart alignment, table containment, and bottom navigation clearance.
