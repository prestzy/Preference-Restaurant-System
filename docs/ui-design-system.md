# Preston's Restaurant UI Design System

## Brand Direction

Customer pages use **Preston's Restaurant**. Admin research pages use the
academic name **Preference-Driven Restaurant Ordering System**. Both surfaces
share the Catppuccin Latte design system so the prototype feels consistent
without making the denser admin tools resemble the customer menu.

## Catppuccin Latte

The canonical tokens live in `static/app.css`. The application uses the
official Catppuccin Latte values, then maps them to semantic roles:

| Role | Application token | Catppuccin source |
|---|---|---|
| Page background | `--app-background` | Base `#eff1f5` |
| Secondary background | `--app-background-secondary` | Mantle `#e6e9ef` |
| Raised surface | `--app-surface` | White `#ffffff` |
| Muted surface | `--app-surface-muted` | Crust `#dce0e8` |
| Main text | `--app-text` | Text `#4c4f69` |
| Muted text | `--app-text-muted` | Subtext 0 `#6c6f85` |
| Primary action | `--app-primary` | Maroon `#e64553` |
| Primary hover | `--app-primary-hover` | Flamingo `#dd7878` |
| Supporting accent | `--app-accent` | Peach `#fe640b` |
| Success | `--app-success` | Green `#40a02b` |
| Warning | `--app-warning` | Yellow `#df8e1d` |
| Danger | `--app-danger` | Red `#d20f39` |
| Information | `--app-info` | Blue `#1e66f5` |
| Keyboard focus | `--app-focus` | Lavender `#7287fd` |

Legacy aliases remain temporarily so existing components can migrate without
duplicating values. Those aliases resolve to the semantic variables and must
not introduce alternative shades.

## Typography

The application uses the local system stack:

```css
Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif
```

Headings are compact and strong. Mobile inputs use at least `16px` text to
avoid browser zoom. Supporting copy uses Catppuccin Subtext.

## Spacing, Radius, and Shadows

- Touch targets: at least `44px`.
- Small radius: `10px`.
- Medium radius: `16px`.
- Large radius: `22px`.
- Pills: `999px`.
- Shadows use translucent Catppuccin Text.
- Major sections use distinct vertical spacing and avoid decorative nested
  cards.

## Buttons and Status

- Primary: Maroon fill with a light label.
- Secondary: white surface with Catppuccin border and Text/Maroon label.
- Destructive: Red, reserved for delete, cancel, and clear operations.
- Completed, ready, and available: Green.
- Pending: Yellow.
- Preparing: Peach.
- Information: Blue.
- Every status includes text and never relies on colour alone.
- Keyboard focus uses a Lavender outline with a non-`color-mix()` fallback.

## Icon System

The server templates and dynamic JavaScript use one inline, Lucide-compatible
SVG subset. All icons use `24 x 24` view boxes, round line caps, two-pixel
strokes, and `currentColor`, so component semantics determine their
Catppuccin colour.

Decorative icons have `aria-hidden="true"` and `focusable="false"`.
Icon-only controls, such as Cart quantity and remove buttons, have
dish-specific `aria-label` and `title` text. Interface controls do not use
emoji or unrelated raster symbols.

## Cards and Chips

Food cards prioritize the dish image, name, price, category, and primary
action. Long ingredients are line-clamped with full details available in the
dish dialog. Inactive chips use neutral surfaces; active chips use Maroon or
Peach with readable text.

## Cart Layout

Every desktop and tablet Cart row uses the same CSS Grid columns:

```text
76px | dish details | quantity stepper | line total | 44px remove action
```

The quantity stepper uses fixed `44px / 40-52px / 44px` tracks. Quantity,
unit price, line total, and summary values use tabular numerals. Browser code
uses one `formatCurrency()` helper and displays all prices as `RM0.00`.

At widths up to `640px`, the row becomes a controlled three-row grid. The
image and remove action stay fixed, while quantity and total receive their own
rows. This prevents long dish names from moving controls or creating
horizontal overflow.

The summary distinguishes unique dishes, total portions, and subtotal.
Quantity changes recalculate the line total and summary in place without a
page reload.

## Mobile Navigation

The bottom navigation uses four equal touch targets and safe-area padding.
The active item includes a label, Maroon icon/text, and Peach underline.
Content includes matching bottom padding so checkout controls are not hidden.

## Tablet and Admin Layouts

At tablet widths, menu grids expand where space permits. Administrative forms
use additional columns. Tables convert to labelled row cards on phones.
Recommendation Tester tools remain grouped into Production, Experiments,
Explainability, and Learning History.

## Accessibility Rules

- Visible Lavender `:focus-visible` outline.
- Minimum `44px` action targets.
- Native labels for fields and `aria-live` result/status regions.
- Dialogs fit within the viewport and scroll internally.
- No interaction depends on hover.
- No page-level horizontal overflow.
- Destructive timeline actions require explicit confirmation.
