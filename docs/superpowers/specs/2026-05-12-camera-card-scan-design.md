# Camera Card Scan — Design Spec

**Date:** 2026-05-12  
**Scope:** MtG only, mobile browser, MVP proof of concept

---

## Goal

Add a "Scan" tab to the mobile UI that lets the user point their phone camera at a Magic: the Gathering card and identify it — including the specific printing — then add it to a collection. Controlled lighting is assumed (personal use).

---

## Architecture

This is a **frontend-only feature**. The existing `POST /api/mtg/cards/search` endpoint already accepts `set_code` and `collector_number` filters, so no backend changes are required for the MVP.

The mobile bottom nav already has a dead "Scan" button. We wire it to a new `/scan` route.

---

## Components

### `ScanView` (new route `/scan`)
Top-level page component. Owns scan state machine: `idle → scanning → processing → result | picker | error`. Renders `CameraFeed` and conditionally `ScanResultPanel` or `PrintingPicker`.

### `CameraFeed`
Wraps `react-webcam` with rear camera (`facingMode: environment`). Overlays a card-shaped alignment guide rectangle the user positions the card inside. Shows a capture button. On capture, draws the video frame to an off-screen `<canvas>` and returns the image data to `ScanView`.

### `ScanResultPanel`
Bottom sheet shown on a successful match. Displays: Scryfall card image (existing image URL pattern), card name, set, collector number, price. "Add to collection" button (reuses existing add-to-collection logic). Dismissible to return to scanning.

### `PrintingPicker`
Shown when name is identified but printing is ambiguous. Horizontal scroll list of small Scryfall card images, one per printing. User taps the correct one, which resolves to `ScanResultPanel`.

---

## OCR Strategy

Uses **Tesseract.js** (runs entirely in-browser, ~4 MB language data download on first use).

Two crop regions are extracted from the captured canvas before OCR:

| Region | Position in card guide box | Reads | Notes |
|---|---|---|---|
| Bottom strip | Bottom 8% of card area | Set code (e.g. `LTR`), collector number (e.g. `0748`) | Present on all cards, standard font even on alternate arts |
| Name strip | Top 10% of card area | Card name | Fails on extreme alternate arts (One Ring style) |

OCR runs on both regions in parallel.

---

## Identification Fallback Chain

```
1. Bottom strip → set_code + collector_number → POST /api/mtg/cards/search
   → 1 result: show ScanResultPanel ✓

2. Name strip → POST /api/mtg/cards/search (exact name)
   → 1 result: show ScanResultPanel ✓
   → multiple: show PrintingPicker

3. Name strip (raw OCR text) → GET https://api.scryfall.com/cards/named?fuzzy=<text>
   → match found: use returned name → POST /api/mtg/cards/search
   → show ScanResultPanel or PrintingPicker depending on result count

4. All above fail → dismiss to manual Search tab with OCR text pre-filled
```

Steps 1 and 2 run in parallel after OCR. Step 3 only fires if both fail. Step 4 is the escape hatch.

---

## New Dependencies

| Package | Purpose | Size |
|---|---|---|
| `react-webcam` | Camera feed in React | ~15 KB |
| `tesseract.js` | In-browser OCR | ~4 MB lang data (cached after first load) |

---

## UI Flow

1. User taps **Scan** in bottom nav → `/scan` route opens
2. Camera feed activates (browser prompts for permission on first use)
3. Alignment guide shows a card-shaped outline — user positions card inside it
4. User taps **Capture** button
5. Processing indicator shown while OCR runs (~1–3 seconds)
6. On match: `ScanResultPanel` slides up from bottom
7. User taps **Add to [collection]** or dismisses to scan another
8. On ambiguous: `PrintingPicker` shown instead
9. On failure: toast with "Couldn't identify card" + button to open Search with OCR text pre-filled

---

## Edge Cases

| Card type | Bottom strip | Name OCR | Resolution |
|---|---|---|---|
| Standard card | ✓ | ✓ | Step 1 — exact match |
| Alternate art (e.g. One Ring borderless) | ✓ | ✗ | Step 1 — exact match via set+number |
| Retro frame (e.g. Stitcher's Supplier) | Partial (number, no set code) | ✓ | Step 2/3 → PrintingPicker |
| OCR typo/partial read | — | Partial | Step 3 — Scryfall fuzzy |
| Complete failure | ✗ | ✗ | Step 4 — manual search |

---

## Out of Scope (MVP)

- Auto card detection / bounding box overlay (alignment guide instead)
- Pokemon or Riftbound cards
- Batch scanning (scan multiple cards in sequence without tapping)
- Offline mode (Scryfall fuzzy requires network)
- Persistent scan history

---

## Files to Create / Modify

**New:**
- `webui/src/Views/ScanView.js`
- `webui/src/Components/CameraFeed.js`
- `webui/src/Components/ScanResultPanel.js`
- `webui/src/Components/PrintingPicker.js`

**Modified:**
- `webui/src/Components/MobileCollectionView.js` — wire Scan button to `/scan` route
- `webui/src/BaseApp.js` — add `/scan` route
- `webui/src/index.css` — scan view styles
