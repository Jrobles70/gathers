# Multi-Select Cards Design

**Date:** 2026-05-12  
**Status:** Approved

## Overview

Allow users to select multiple cards from a collection view and perform bulk actions: move to another collection (existing or new) or remove from the current collection. Selection is triggered via a hover checkbox on each card. A floating action bar appears at the bottom of the screen when any cards are selected.

## Context

The app already has `SelectedCardsContext`, `MoveCards.js`, and `DeleteCards.js`. A comment in `CardListNav.js` explicitly notes these are hidden pending grid-view selection UI. This feature builds that UI on top of the existing infrastructure.

## Architecture

### 1. Card Checkbox Overlay — `CardShell.js`

- Add a small checkbox `<button>` positioned absolute top-left of the card art area
- Visible when `hovered || selected` (uses existing `hovered` and `selected` state)
- Clicking it calls the existing `toggleSelected()` function
- The card image `<Link>` continues to navigate to the detail page normally (no behavior change)
- Replace `border border-primary` (Bootstrap blue, harsh) with a CSS class `card-selected` using `outline: 2px solid rgba(147, 197, 253, 0.65)` — a soft blue that doesn't shift layout
- Same selection border applies in list mode (existing `.card-list-item.selected` style already exists; adjust color to match)

### 2. `SelectionActionBar` Component — new file

Fixed-position bar anchored to the bottom-center of the viewport. Animates in (slide-up) when `selected.length > 0`, hidden otherwise.

Contents:
- **"X selected"** count label
- **Select All** — dispatches `added` for every card in `CardsContext` (current page, up to 50)
- **Deselect All** — dispatches `empty` to `SelectedCardsContext`
- **Move To** — opens `MoveToDialog`
- **Remove** — reuses `DeleteCards` confirmation + API logic; removes each card from its `collectionId`

The bar is rendered inside `CardListView` so it has access to all required contexts.

### 3. `MoveToDialog` Component — new file

A modal with two tabs:

**Existing Collection tab**
- Dropdown of all collections (excludes current collection and `__all__`)
- Confirm button calls `POST /collection/move/{destinationId}` with the selected cards array

**New Collection tab**
- Text input for the new collection name
- Parent dropdown defaulting to:
  - The current collection's `parent` value (if it has one)
  - No parent (top-level) if the current collection is top-level
- On confirm: calls `POST /collection/add` with `{ id, parent }`, then calls `POST /collection/move/{newId}`
- After success: refreshes card list, clears selection, closes modal, dispatches `added` to `CollectionsContext`

Both tabs: after successful move, call `triggerRefresh(true)` and `selectedDispatch({ type: "empty" })`.

### 4. Page Size — `CardsContext.js`

Change `pageSize` constant from `24` to `50`.

### 5. Select All Wiring — `CardList.js`

`CardList` already holds the current page's card details in its `cards` state (from `CardsContext`). "Select All" in the action bar will dispatch `added` for each entry in `useCards()`. No new data fetching needed.

## Data Flow

```
User hovers card → checkbox appears
User clicks checkbox → toggleSelected() → SelectedCardsContext updated
SelectionActionBar reads selected.length → appears when > 0

Select All → useCards() entries → batch dispatch "added"
Deselect All → dispatch "empty"

Move To → MoveToDialog:
  Existing: POST /collection/move/{id}
  New:      POST /collection/add → POST /collection/move/{newId}

Remove → confirm dialog → POST /collection/cards/{collectionId}/delete (per card)
```

## Files Changed

| File | Change |
|------|--------|
| `webui/src/Components/CardShell.js` | Add checkbox overlay, update selection border class |
| `webui/src/Components/CardListContexts/CardsContext.js` | `pageSize` 24 → 50 |
| `webui/src/Views/CardListView.js` | Render `<SelectionActionBar />` |
| `webui/src/Components/SelectionActionBar.js` | New — floating action bar |
| `webui/src/Components/MoveToDialog.js` | New — move/create collection modal |
| `webui/src/index.css` | Checkbox overlay styles, `.card-selected` outline, floating bar styles |

## CSS Notes

- Checkbox: `position: absolute; top: 6px; left: 6px; z-index: 10; width: 20px; height: 20px` — visible at `opacity: 1` when hovered/selected, `opacity: 0` otherwise (still keyboard accessible)
- Selection outline: `outline: 2px solid rgba(147, 197, 253, 0.65); outline-offset: 2px` — soft blue, no layout shift
- Floating bar: `position: fixed; bottom: 1.5rem; left: 50%; transform: translateX(-50%); z-index: 200` — centered, above all content; CSS `transition: opacity + translateY` for slide-in animation

## Edge Cases

- **All Collections view**: Remove uses each card's `collectionId` (already on the details object). Move works the same way.
- **Creating a collection that already exists**: Server will return an error; surface it in the dialog.
- **Navigating pages while cards are selected**: Selection persists across page changes (context lives above CardList). "Select All" only adds the current page's visible cards.
- **No destination collections**: The "Existing Collection" tab's confirm button is disabled if the dropdown is empty or matches the current collection.
