# Mobile UI Pass — Design Spec
Date: 2026-05-11

## Overview

Three focused visual improvements to the existing mobile layout (`≤ 760px`). All changes are additive reformats of data that already exists — no new API endpoints, no new features. Desktop code paths are untouched.

---

## 1. Stats Header with Scroll Fade

**Replaces:** `MobileCollectionSummary` (the static conic-gradient donut ring).

**Renders:**
- Non-proxy total value: `totalValueCents - proxyTotalValueCents`, formatted with `formatMobileCents`
- Change line: `+$1.11K (+37%)` using `changeCents` and `changePercent` from the `/collection/stats` response, colored with `price-up` / `price-down` classes
- Card count: `2616 cards` (non-proxy: `copyCount - proxyCopyCount`)

**Scroll fade behavior:**
- The stats block sits above the `mobile-section-tabs` row
- A `useRef` + `IntersectionObserver` watches whether the stats block is fully in view
- As it leaves the viewport (tabs scroll up to meet it), the stats block transitions `opacity: 1 → 0` over ~80px of scroll travel
- Implemented via a CSS custom property `--stats-opacity` set on the wrapper by an `onScroll` handler; no JS animation libraries

**Proxy exclusion:**
- Displayed total = `totalValueCents - proxyTotalValueCents`
- Displayed count = `copyCount - proxyCopyCount`
- Change figures (`changeCents`, `changePercent`) are taken as-is from the API (already net of proxies at the server level; if not, they're displayed as returned — no server changes in scope)

---

## 2. Per-Card Price Delta in Grid Tiles

**Current behavior:** `CardShell` renders `priceText` as a single joined string (e.g. `"$98.57 (+557%)"`) inside one `<span className={priceClass}>`.

**Change:** Split into two sibling `<span>` elements inside the card footer:
1. `<span className="search-card-price">$98.57</span>` — the unit price, always shown
2. `<span className="search-card-price-delta price-up">+$83.57 (+557%)</span>` — the absolute change + percent, only rendered when `activeTrend` exists and direction is not `"flat"`

The absolute change amount (`changeCents`) is already computed by `priceTrend()` in `priceUtils.js`. The delta span uses `formatCents(activeTrend.changeCents)` with a leading `+` sign when positive.

**Mobile CSS:** The `.mobile-card-grid .search-card-footer` already lays out as a single-column grid. The delta span is styled smaller, uses `price-up` / `price-down` for color, and wraps to a new line naturally. No changes to desktop card footer layout.

**Scope:** Only `CardShell.js` and the mobile section of `index.css`. On desktop, the delta span renders inline after the price (same visual as today, since the existing `priceText` already joined them with a space). No desktop CSS changes needed.

---

## 3. Mobile Card Detail Bottom Sheet with Carousel

**Mobile only.** Lives entirely inside `MobileCollectionView`. Desktop card taps continue navigating to `/card/mtg/:id` unchanged.

### Trigger
`MobileCollectionDetail` adds an `onClick` capture handler on the `.mobile-card-grid` wrapper. When a click reaches a `.search-card-image-link` (or `.search-card-art`) element, the default navigation is `preventDefault()`-ed and the sheet opens with the tapped card's index.

Card index is resolved by matching the clicked element's closest `.search-card` against the ordered `useCards()` array.

### State
```js
const [sheetCardIndex, setSheetCardIndex] = useState(null); // null = closed
```
`sheetCardIndex` is the index into the `useCards()` array of the currently active carousel card.

### MobileCardSheet component
Rendered inside `MobileCollectionDetail` when `sheetCardIndex !== null`.

**Structure (bottom to top in z-order):**
1. **Backdrop** — full-screen dim overlay, `onClick` closes the sheet
2. **Sheet panel** — slides up from bottom, `position: fixed`, covers ~85% of viewport height
   - **Carousel** (top ~55% of sheet): horizontally scrollable container with `scroll-snap-type: x mandatory`. One image per card in `useCards()`. Each image uses the existing Scryfall image URL pattern (same as desktop). Starts scrolled to `sheetCardIndex`.
   - An `IntersectionObserver` on each image (or a `onScroll` + debounce on the carousel container) updates `sheetCardIndex` as the user swipes.
   - **Detail panel** (bottom ~45%): dark rounded panel showing:
     - Quantity + card name (bold, large)
     - Set name + collector number
     - Collection badge (existing `.collection-pill` or similar)
     - Price: `$74.17` + delta `+$24.17 (+48.3%)` in green/red
     - Action row: the existing `<CardDetails>` component rendered inside the sheet. It already handles add/remove quantity, foil toggle, collection move, and delete. No new actions.

**Swipe-to-dismiss:** A `touchstart`/`touchend` handler on the sheet panel. If vertical drag delta > 80px downward, close the sheet (`setSheetCardIndex(null)`).

**Carousel image source:** Same logic as `CardShell` / `MtGCard` — `https://api.scryfall.com/cards/{scryfallId}?format=image`. Falls back to a placeholder div if no `scryfallId`. Works for all three TCG types (Pokemon/Riftbound use their own image paths — carousel uses `details.card.cardIdentifiers?.scryfallId` or equivalent).

### CSS
New classes: `mobile-card-sheet-backdrop`, `mobile-card-sheet`, `mobile-sheet-carousel`, `mobile-sheet-detail`, `mobile-sheet-actions`. All scoped inside `@media (max-width: 760px)`. Sheet uses `transform: translateY(100%)` → `translateY(0)` with `transition: transform 0.28s ease-out` for the slide-up animation.

---

## Implementation Split (for parallel agents)

### Agent A — Per-card price delta
Files: `webui/src/Components/CardShell.js`, `webui/src/index.css` (mobile grid section only)

### Agent B — Stats header + bottom sheet
Files: `webui/src/Components/MobileCollectionView.js`, `webui/src/index.css` (mobile summary + sheet sections)

Merge: Both agents add CSS in non-overlapping sections of `index.css`. Merge is additive — no conflicts expected.

---

## Out of Scope
- Dynamic donut ring (deferred)
- Decks / Scan bottom nav tabs (stub, no backend support)
- Any new API endpoints
- Desktop layout changes
