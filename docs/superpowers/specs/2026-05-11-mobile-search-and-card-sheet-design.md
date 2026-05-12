# Mobile Search View + Card Sheet Redesign — Design Spec
Date: 2026-05-11

## Overview

Two parallel mobile UI improvements (≤ 760px only). Desktop code paths are untouched in both cases.

- **Feature A:** New `MobileSearchView` — replaces the desktop-style search form on mobile with a filter-panel layout matching the Moxfield-style inspiration.
- **Feature B:** Card sheet layout redesign — the card image now floats above the bottom panel (outside the sheet), matching the Moxfield card-detail popup style.

Both features are independent and can be built by parallel agents with no shared state or merge conflicts. Both agents add CSS in non-overlapping sections of `index.css`. Testing on port 3000 (mobile viewport).

---

## Feature A — Mobile Search View

### Goal
When the user taps Search in the bottom nav on mobile, show a purpose-built mobile layout: prominent name search bar at top, Single/Bulk tabs, always-visible scrollable filter sections, floating search FAB, and the existing bottom nav.

### Architecture

**New file:** `webui/src/Components/MobileSearchView.js`
- Consumes `useCardSearch` (already exists) for all search state and API calls.
- Consumes `useCardSets` for set code datalist.
- Consumes `useCollections` and `useMode` for collection-aware features.
- Imports `MobileBottomNav` (exported from `MobileCollectionView.js`) and renders it with `activeTab="search"`.
- No logic duplication — all API wiring lives in the existing hooks.

**Modified:** `webui/src/Views/SearchView.js`
- Adds a `useIsMobile` hook: `window.matchMedia('(max-width: 760px)')` with a `resize` listener.
- On mobile → renders `<MobileSearchView />`.
- On desktop → renders existing `<Search />` unchanged.
- Also adds `mobile-search-sidebar-hidden` class on the root so the desktop sidebar hides on mobile for the search route.

**Modified:** `webui/src/Components/MobileCollectionView.js`
- `MobileBottomNav` gains an `activeTab` prop (defaults to `"collection"` so existing behavior is unchanged).
- The component is `export`-ed (currently not exported).
- The active state is driven by `activeTab` prop instead of being hardcoded.

**Modified:** `webui/src/index.css`
- New mobile search styles added at end of the `@media (max-width: 760px)` block.
- Classes: `mobile-search-topbar`, `mobile-search-bar`, `mobile-search-tabs`, `mobile-search-tab`, `mobile-search-filters-body`, `mobile-filter-group`, `mobile-filter-group-label`, `mobile-filter-input`, `mobile-color-circles`, `mobile-color-circle`, `mobile-search-fab`, `mobile-search-results`, `mobile-search-bulk-body`.

### UI Layout (Single mode)

```
┌─────────────────────────────────┐
│  [🔍 Search cards            ]  │  ← .mobile-search-bar full-width rounded pill
│  [Single ──────]  [Bulk]        │  ← .mobile-search-tabs, underline active tab
├─────────────────────────────────┤  scrollable body begins
│  FILTERS                        │  ← gold/accent label
│                                 │
│  Colors                         │
│  ⚪ 🔵 ⚫ 🔴 🟢 ◇              │  ← color circle toggles (W/U/B/R/G/C)
│                                 │     checked = filled/colored, unchecked = dim
│  Set                            │
│  ________________________       │  ← underline input + datalist
│                                 │
│  Text                           │
│  ________________________       │
│                                 │
│  Artist                         │
│  ________________________       │
│                                 │
│  Sort                           │
│  [Name ▾]  [Asc ▾]             │  ← existing SortControls, compact
│                                 │
│  (Search in: [collection ▾])    │  ← only shown when collectionsEnabled
│                                 │
│  ── search results grid ──      │
│  ── pagination ──               │
└─────────────────────────────────┘
                          [🔍]    ← .mobile-search-fab fixed, above bottom nav
│  Home  Search  Coll  Decks Scan │
```

### UI Layout (Bulk mode)

Tabs row switches to Bulk active. Filter body is replaced by:
- Large textarea (same placeholder as existing bulk search)
- Meta line: "N cards parsed" / "N/N owned"
- FAB triggers bulk search
- Results grid + missing-cards pills appear below

### Color Circles

The six color identities (White/Blue/Black/Red/Green/Colorless) render as circular toggles (~40px diameter). Each uses the existing mana SVG symbols from `src/assets/card-symbols/`. Selected state: colored background. Unselected: dim/outline. Tapping toggles `colorIdentities` via the existing `handleArrayInput`.

### Floating FAB

`.mobile-search-fab` — fixed position, bottom: calc(bottom-nav-height + 16px), right: 16px. Orange background (`#f0a500` or existing accent), 56px circle, white magnifying glass icon. `onClick` → `triggerSearch()` (single) or `handleBulkSearch()` (bulk). The existing form `onSubmit` still works for keyboard submit.

### Search Results

Results render using the existing `Card` component and `groupMagicSearchResults` / `listMagicSearchResultsByPrinting` logic. Loading state shows "Loading…". `SearchPagination` is rendered below results.

### Out of Scope (Feature A)
- Riftbound / Pokémon system switcher on mobile (deferred — MTG-only for now)
- Set browsing tab (no backend support)
- Collector number field (excluded from mobile filter panel to keep it clean; accessible on desktop)

---

## Feature B — Card Sheet Layout Redesign

### Goal
Redesign `MobileCardSheet` so the card image floats freely above the bottom info panel (overlapping the dimmed grid behind), matching the Moxfield-style popup. The carousel and all swipe/dismiss behavior are preserved; only the visual structure changes.

### Current Problems (observed in production)

1. **No horizontal swiping** — only one card is shown; the carousel does not scroll between cards.
2. **CardDetails controls misplaced** — the `+`, `1`, `-`, `...` buttons float over the right edge of the card image instead of sitting in a bottom info panel.
3. **Dead space** — the bottom half of the sheet below the card details is empty black.
4. **Layout is wrong** — the card image and controls need to be restructured: image floating, controls in a compact bottom panel.

### What Changes

**Current structure:**
```
<backdrop>
<sheet (85vh, fixed bottom)>
  <carousel (top 55% of sheet)>
  <detail panel (bottom 45% of sheet)>
```

**New structure:**
```
<backdrop (dim, 60% opacity, grid visible behind)>
<carousel (fixed, from ~top-bar-bottom to ~bottom-panel-top, full-width)>
<bottom panel (fixed bottom, ~42vh, dark rounded)>
  <detail content>
  <CardDetails actions>
```

The carousel becomes a sibling of the bottom panel, not a child of it. The card image takes up the vertical space between the top bar and the sheet.

### Modified: `webui/src/Components/MobileCollectionView.js` — `MobileCardSheet`

JSX restructure only — the state, hooks, event handlers (`handleCarouselScroll`, `handleTouchStart`, `handleTouchEnd`, `useCardLoader` effect) are unchanged.

New JSX skeleton:
```jsx
<>
  {/* backdrop — dimmed but not opaque so grid shows through */}
  <div className="mobile-card-sheet-backdrop" onClick={onClose} />

  {/* carousel — floats between top bar and bottom panel */}
  <div ref={carouselRef} className="mobile-sheet-carousel" onScroll={handleCarouselScroll}>
    {cards.map((card, index) => (
      <div key={card.id ?? index} className="mobile-sheet-carousel-slide">
        {index === activeIndex && imgSrc
          ? <img src={imgSrc} alt={activeCardData?.name ?? "Card"} />
          : <div className="mobile-sheet-carousel-placeholder" />}
      </div>
    ))}
  </div>

  {/* bottom panel — info + actions */}
  <div
    className="mobile-card-sheet"
    onTouchStart={handleTouchStart}
    onTouchEnd={handleTouchEnd}
    role="dialog"
    aria-label="Card detail"
    aria-modal="true"
  >
    <div className="mobile-sheet-handle" />
    <div className="mobile-sheet-detail">
      <div className="mobile-sheet-detail-name">
        <span className="mobile-sheet-qty">×{qty}</span>
        <strong>{activeCardData?.name ?? activeDetails?.id}</strong>
      </div>
      <div className="mobile-sheet-detail-meta">
        {activeCardData?.setCode && <span className="search-card-set">{activeCardData.setCode}</span>}
        {activeDetails?.collectionId && (
          <span className="collection-pill">{activeDetails.collectionId}</span>
        )}
      </div>
      {unitPrice != null && (
        <div className="mobile-sheet-price-row">
          <span className="mobile-sheet-price-label">MARKET</span>
          <span className="mobile-sheet-price">{formatCents(unitPrice)}</span>
          {trend && trend.direction !== "flat" && (
            <span className={trend.direction === "up" ? "price-up" : "price-down"}>
              {(trend.changeCents >= 0 ? "+" : "") + formatCents(trend.changeCents)}
              {" "}({formatPercent(trend.changePercent)})
            </span>
          )}
        </div>
      )}
      <div className="mobile-sheet-actions">
        {activeDetails && (
          <CardDetails
            id={activeDetails.id}
            details={activeDetails}
            showCollectionSelect={false}
            targetCollection={activeDetails?.collectionId ?? null}
          />
        )}
      </div>
    </div>
  </div>
</>
```

### CSS Changes (Feature B)

All changes are inside `@media (max-width: 760px)`. Existing class names are updated in place.

Existing z-index values from `index.css`: backdrop=1000, sheet=1010. Carousel slots between them.

```
.mobile-card-sheet-backdrop
  background: rgba(0, 0, 0, 0.65)   ← was 0.55; slightly darker to keep grid readable behind floating card
  z-index: 1000                      ← unchanged

.mobile-sheet-carousel
  position: fixed
  top: 3.5rem                        ← matches .mobile-collection-topbar height
  bottom: 42vh                       ← above bottom panel
  left: 0; right: 0
  display: flex
  overflow-x: scroll; scroll-snap-type: x mandatory
  align-items: center
  z-index: 1005                      ← above backdrop (1000), below panel (1010)

.mobile-sheet-carousel-slide
  min-width: 100vw
  scroll-snap-align: center
  display: flex; justify-content: center; align-items: center

.mobile-sheet-carousel-slide img
  max-width: 75vw
  max-height: 100%
  border-radius: 12px
  box-shadow: 0 8px 32px rgba(0,0,0,0.6)

.mobile-card-sheet
  position: fixed
  bottom: 0; left: 0; right: 0
  height: 42vh                       ← reduced from 85dvh; carousel is now outside
  background: var(--mobile-panel)    ← unchanged
  border-radius: 1.1rem 1.1rem 0 0  ← unchanged
  z-index: 1010                      ← unchanged
  overflow-y: auto
  animation: sheet-slide-up 0.28s ease-out  ← unchanged
```

### Swipe-to-dismiss

Unchanged — `handleTouchStart` / `handleTouchEnd` on `.mobile-card-sheet`. The carousel's horizontal scroll is not affected by the vertical dismiss gesture (bail already implemented: `if (e.target.closest(".mobile-sheet-detail")) return`).

### Out of Scope (Feature B)
- "Added on" date field (not in current data model)
- Language / condition tags (not in current data model)
- Any changes to the desktop card detail page

---

## Parallel Agent Split

| Agent | Feature | Files touched |
|-------|---------|--------------|
| A | Mobile Search View | `MobileSearchView.js` (new), `SearchView.js`, `MobileCollectionView.js` (export + prop), `index.css` (new search classes) |
| B | Card Sheet Redesign | `MobileCollectionView.js` (MobileCardSheet JSX + CSS classes), `index.css` (update card sheet classes) |

**Merge:** Agents edit `index.css` in non-overlapping sections (Agent A adds new search classes; Agent B modifies existing sheet classes). Both agents touch `MobileCollectionView.js` but in different functions — this is a real merge conflict when worktrees are integrated. **Resolution:** Apply Agent A's `MobileBottomNav` changes (export + `activeTab` prop) to `main` before launching Agent B, so Agent B's worktree already has the correct base.

## Testing

Both agents test on `http://localhost:3000` at mobile viewport (≤ 760px). Agent A: navigate to `/search`, verify filter layout, test single and bulk modes, verify FAB triggers search. Agent B: navigate to a collection (`/c/<id>/1`), tap a card, verify sheet opens with floating card image, swipe between cards, verify info panel updates, verify swipe-to-dismiss.
