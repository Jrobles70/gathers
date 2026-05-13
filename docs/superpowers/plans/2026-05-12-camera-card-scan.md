# Camera Card Scan — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `/scan` route to the mobile UI that uses the device camera to identify MtG card printings via OCR and add them to a collection.

**Architecture:** Frontend-only — no backend changes. The existing `POST /api/mtg/cards/search` endpoint already accepts `set_code` and `collector_number` filters. Tesseract.js runs OCR in-browser on two regions of a captured camera frame (bottom strip for set+number, top strip for card name). A 5-step fallback chain resolves ambiguous reads.

**Tech Stack:** React 18, `react-webcam`, `tesseract.js`, existing `useOperations` hook, `fetch` API.

---

## File Map

| File | Action | Responsibility |
|---|---|---|
| `webui/src/Views/ScanView.js` | Create | State machine orchestrator for the scan flow |
| `webui/src/Components/CameraFeed.js` | Create | Camera feed + alignment guide + capture button |
| `webui/src/Components/ScanResultPanel.js` | Create | Bottom sheet showing matched card + add-to-collection |
| `webui/src/Components/PrintingPicker.js` | Create | Horizontal scroll list of printings when match is ambiguous |
| `webui/src/Components/useScanOcr.js` | Create | OCR hook: crop regions, run Tesseract, parse results |
| `webui/src/Components/scanSearch.js` | Create | Fallback search chain (set+number → name → fuzzy → picker) |
| `webui/src/Components/MobileCollectionView.js` | Modify | Wire dead Scan button to `/scan` Link |
| `webui/src/BaseApp.js` | Modify | Add `/scan` route |
| `webui/src/index.css` | Modify | Scan view styles |

---

## Task 1: Install dependencies

**Files:**
- Modify: `webui/package.json` (via npm install)

- [ ] **Step 1: Install react-webcam and tesseract.js**

```bash
cd webui && npm install react-webcam tesseract.js
```

Expected output: two new entries in `package.json` dependencies, no peer dep errors.

- [ ] **Step 2: Verify install**

```bash
ls node_modules/react-webcam && ls node_modules/tesseract.js
```

Expected: both directories exist.

- [ ] **Step 3: Commit**

```bash
git add webui/package.json webui/package-lock.json
git commit -m "feat(scan): install react-webcam and tesseract.js"
```

---

## Task 2: Wire Scan nav button and add route

**Files:**
- Modify: `webui/src/Components/MobileCollectionView.js:84-91`
- Modify: `webui/src/BaseApp.js`

- [ ] **Step 1: Wire the Scan button to a Link in MobileCollectionView.js**

Find this block (lines 88-91):
```jsx
<button type="button" className="mobile-bottom-nav-item">
  <span aria-hidden="true">▢</span>
  Scan
</button>
```

Replace with:
```jsx
<Link to="/scan" className={"mobile-bottom-nav-item" + (activeTab === "scan" ? " active" : "")}>
  <span aria-hidden="true">▢</span>
  Scan
</Link>
```

(`Link` is already imported at the top of MobileCollectionView.js.)

- [ ] **Step 2: Add ScanView import and `/scan` route to BaseApp.js**

Add import after the existing view imports:
```jsx
import ScanView from "./Views/ScanView";
```

Add route inside `<Routes>` before the closing tag, after the `/search` route:
```jsx
<Route path="/scan" element={<ScanView />} />
```

- [ ] **Step 3: Create a placeholder ScanView so the route doesn't crash**

Create `webui/src/Views/ScanView.js`:
```jsx
import React from "react";

export default function ScanView() {
  return (
    <div style={{ padding: "2rem", color: "#fff" }}>
      Scan coming soon
    </div>
  );
}
```

- [ ] **Step 4: Start the dev server and verify tapping Scan navigates to `/scan`**

```bash
cd webui && npm start
```

Open on mobile (or mobile emulation in devtools). Tap Scan in the bottom nav. URL should change to `/scan` and show "Scan coming soon".

- [ ] **Step 5: Commit**

```bash
git add webui/src/Components/MobileCollectionView.js webui/src/BaseApp.js webui/src/Views/ScanView.js
git commit -m "feat(scan): wire Scan nav to /scan route with placeholder view"
```

---

## Task 3: OCR parsing utilities and tests

**Files:**
- Create: `webui/src/Components/scanSearch.js`
- Create: `webui/src/Components/scanSearch.test.js`

This task covers only the pure parsing functions — no fetch calls, no React. Test these first.

- [ ] **Step 1: Write failing tests for `parseBottomStrip`**

Create `webui/src/Components/scanSearch.test.js`:
```javascript
import { parseBottomStrip, parseCardName } from "./scanSearch";

describe("parseBottomStrip", () => {
  test("parses standard format: number then set code", () => {
    expect(parseBottomStrip("179 RNA • EN Justine Jones")).toEqual({
      collectorNumber: "179",
      setCode: "RNA",
    });
  });

  test("parses leading M prefix on serialized collector number", () => {
    expect(parseBottomStrip("M 0748 LTR • EN")).toEqual({
      collectorNumber: "0748",
      setCode: "LTR",
    });
  });

  test("parses when bullet is a middle dot", () => {
    expect(parseBottomStrip("853 SLD · EN Chris Seaman")).toEqual({
      collectorNumber: "853",
      setCode: "SLD",
    });
  });

  test("returns null fields when set code not found", () => {
    expect(parseBottomStrip("Illus. Chris Seaman 853")).toEqual({
      collectorNumber: "853",
      setCode: null,
    });
  });

  test("returns null fields for empty/unreadable text", () => {
    expect(parseBottomStrip("")).toEqual({
      collectorNumber: null,
      setCode: null,
    });
  });
});

describe("parseCardName", () => {
  test("trims whitespace and control characters", () => {
    expect(parseCardName("  Lightning Helix\n")).toBe("Lightning Helix");
  });

  test("returns null for empty string", () => {
    expect(parseCardName("")).toBeNull();
  });

  test("returns null for whitespace-only string", () => {
    expect(parseCardName("   ")).toBeNull();
  });

  test("strips common OCR noise characters", () => {
    expect(parseCardName("|Stitcher's Supplier|")).toBe("Stitcher's Supplier");
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd webui && npm test -- --testPathPattern=scanSearch --watchAll=false
```

Expected: FAIL — `Cannot find module './scanSearch'`

- [ ] **Step 3: Implement `parseBottomStrip` and `parseCardName` in scanSearch.js**

Create `webui/src/Components/scanSearch.js`:
```javascript
// Matches a 2-5 uppercase-letter set code followed by a bullet (• or ·)
const SET_CODE_RE = /([A-Z]{2,5})\s*[•·]/;
// Matches one or more digits, optionally preceded by a letter+space like "M 0748"
const COLLECTOR_NUM_RE = /(?:[A-Z]\s+)?(\d+)/;
// Strip leading/trailing pipe characters and other OCR noise
const OCR_NOISE_RE = /^[|[\]{}]+|[|[\]{}]+$/g;

export function parseBottomStrip(text) {
  if (!text || !text.trim()) return { collectorNumber: null, setCode: null };

  const setMatch = SET_CODE_RE.exec(text);
  const numMatch = COLLECTOR_NUM_RE.exec(text);

  return {
    collectorNumber: numMatch ? numMatch[1] : null,
    setCode: setMatch ? setMatch[1] : null,
  };
}

export function parseCardName(text) {
  if (!text) return null;
  const cleaned = text.replace(OCR_NOISE_RE, "").trim();
  // Take just the first line — card name is always a single line
  const firstLine = cleaned.split("\n")[0].trim();
  return firstLine.length > 0 ? firstLine : null;
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd webui && npm test -- --testPathPattern=scanSearch --watchAll=false
```

Expected: all 8 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add webui/src/Components/scanSearch.js webui/src/Components/scanSearch.test.js
git commit -m "feat(scan): add OCR parsing utilities with tests"
```

---

## Task 4: Search chain and tests

**Files:**
- Modify: `webui/src/Components/scanSearch.js`
- Modify: `webui/src/Components/scanSearch.test.js`

Add the async search functions that use the parsed OCR values to find a card.

- [ ] **Step 1: Write failing tests for the search functions**

Append to `webui/src/Components/scanSearch.test.js`:
```javascript
import { searchBySetAndNumber, searchByName, fuzzySearch } from "./scanSearch";

const MOCK_CARD = {
  id: "abc123",
  name: "Lightning Helix",
  setCode: "RNA",
  collectorNumber: "179",
  cardIdentifiers: { scryfallId: "fake-scryfall-id" },
};

beforeEach(() => {
  global.fetch = jest.fn();
});

afterEach(() => {
  jest.resetAllMocks();
});

describe("searchBySetAndNumber", () => {
  test("returns card when API returns exactly one result", async () => {
    fetch.mockResolvedValueOnce({
      ok: true,
      status: 200,
      text: async () => JSON.stringify([MOCK_CARD]),
      json: async () => [MOCK_CARD],
    });

    const result = await searchBySetAndNumber("RNA", "179");
    expect(result).toEqual([MOCK_CARD]);
    const [url, opts] = fetch.mock.calls[0];
    expect(url).toBe("/api/mtg/cards/search?limit=5");
    const body = JSON.parse(opts.body);
    expect(body.setCode).toBe("RNA");
    expect(body.collectorNumber).toBe("179");
  });

  test("returns null when either param is null", async () => {
    const result = await searchBySetAndNumber(null, "179");
    expect(result).toBeNull();
    expect(fetch).not.toHaveBeenCalled();
  });
});

describe("searchByName", () => {
  test("returns results array from API", async () => {
    fetch.mockResolvedValueOnce({
      ok: true,
      status: 200,
      text: async () => JSON.stringify([MOCK_CARD]),
      json: async () => [MOCK_CARD],
    });

    const result = await searchByName("Lightning Helix");
    expect(result).toEqual([MOCK_CARD]);
    const body = JSON.parse(fetch.mock.calls[0][1].body);
    expect(body.name).toBe("Lightning Helix");
  });

  test("returns null when name is null", async () => {
    const result = await searchByName(null);
    expect(result).toBeNull();
    expect(fetch).not.toHaveBeenCalled();
  });
});

describe("fuzzySearch", () => {
  test("uses Scryfall fuzzy name then searches local API", async () => {
    // First call: Scryfall fuzzy
    fetch.mockResolvedValueOnce({
      ok: true,
      status: 200,
      text: async () => JSON.stringify({ name: "Lightning Helix" }),
      json: async () => ({ name: "Lightning Helix" }),
    });
    // Second call: local search
    fetch.mockResolvedValueOnce({
      ok: true,
      status: 200,
      text: async () => JSON.stringify([MOCK_CARD]),
      json: async () => [MOCK_CARD],
    });

    const result = await fuzzySearch("Lightnig Helix");
    expect(result).toEqual([MOCK_CARD]);
    expect(fetch.mock.calls[0][0]).toContain("api.scryfall.com/cards/named?fuzzy=");
  });

  test("returns null when Scryfall returns 404", async () => {
    fetch.mockResolvedValueOnce({ ok: false, status: 404 });
    const result = await fuzzySearch("xyzzy garbage");
    expect(result).toBeNull();
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd webui && npm test -- --testPathPattern=scanSearch --watchAll=false
```

Expected: FAIL — `searchBySetAndNumber is not a function`

- [ ] **Step 3: Implement search functions in scanSearch.js**

Append to `webui/src/Components/scanSearch.js`:
```javascript
async function postSearch(filters, limit = 5) {
  const response = await fetch(`/api/mtg/cards/search?limit=${limit}`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Accept: "application/json" },
    body: JSON.stringify(filters),
  });
  if (!response.ok) return null;
  const text = await response.text();
  return text.length > 0 ? JSON.parse(text) : null;
}

export async function searchBySetAndNumber(setCode, collectorNumber) {
  if (!setCode || !collectorNumber) return null;
  return postSearch({ setCode, collectorNumber });
}

export async function searchByName(name) {
  if (!name) return null;
  return postSearch({ name });
}

export async function fuzzySearch(rawText) {
  if (!rawText) return null;
  const scryfallRes = await fetch(
    `https://api.scryfall.com/cards/named?fuzzy=${encodeURIComponent(rawText)}`
  );
  if (!scryfallRes.ok) return null;
  const scryfallCard = await scryfallRes.json();
  const resolvedName = scryfallCard?.name;
  if (!resolvedName) return null;
  return postSearch({ name: resolvedName }, 20);
}
```

- [ ] **Step 4: Run all scanSearch tests**

```bash
cd webui && npm test -- --testPathPattern=scanSearch --watchAll=false
```

Expected: all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add webui/src/Components/scanSearch.js webui/src/Components/scanSearch.test.js
git commit -m "feat(scan): add search chain (set+number, name, fuzzy) with tests"
```

---

## Task 5: useScanOcr hook

**Files:**
- Create: `webui/src/Components/useScanOcr.js`

This hook accepts a base64 screenshot data URL, crops two regions, runs Tesseract.js on both in parallel, and returns parsed OCR results.

The guide box is defined by these constants (as fractions of the image dimensions):
- Width: 85% of image width, centered
- Aspect ratio: 63:88 (standard card, portrait)
- Vertical center: 42% from the top of the image

Both the CSS overlay and the crop math must use the same constants.

- [ ] **Step 1: Write the hook**

Create `webui/src/Components/useScanOcr.js`:
```javascript
import { useRef, useCallback, useState } from "react";
import { createWorker } from "tesseract.js";
import { parseBottomStrip, parseCardName } from "./scanSearch";

// Guide box geometry — must match CameraFeed.js overlay constants
export const GUIDE = {
  widthFraction: 0.85,
  aspectRatio: 63 / 88,   // card width / card height
  centerYFraction: 0.50,  // guide box is vertically centered — must match CameraFeed CSS
};

function cropRegion(canvas, ctx, src, x, y, w, h) {
  canvas.width = w;
  canvas.height = h;
  ctx.drawImage(src, x, y, w, h, 0, 0, w, h);
  return canvas.toDataURL("image/png");
}

export function useScanOcr() {
  const workerRef = useRef(null);
  const [loading, setLoading] = useState(false);

  const ensureWorker = useCallback(async () => {
    if (!workerRef.current) {
      const worker = await createWorker("eng");
      workerRef.current = worker;
    }
    return workerRef.current;
  }, []);

  const runOcr = useCallback(async (screenshotDataUrl) => {
    setLoading(true);
    try {
      const worker = await ensureWorker();

      // Load the screenshot into an Image to get natural dimensions
      const img = await new Promise((resolve, reject) => {
        const el = new Image();
        el.onload = () => resolve(el);
        el.onerror = reject;
        el.src = screenshotDataUrl;
      });

      const iw = img.naturalWidth;
      const ih = img.naturalHeight;

      // Guide box bounds in pixel coordinates
      const gw = iw * GUIDE.widthFraction;
      const gh = gw / GUIDE.aspectRatio;
      const gx = (iw - gw) / 2;
      const gy = ih * GUIDE.centerYFraction - gh / 2;

      // Name strip: top 10% of guide box
      const nameH = gh * 0.10;
      const nameY = gy;

      // Bottom strip: bottom 8% of guide box
      const bottomH = gh * 0.08;
      const bottomY = gy + gh - bottomH;

      const offscreen = document.createElement("canvas");
      const ctx = offscreen.getContext("2d");

      const nameDataUrl = cropRegion(offscreen, ctx, img, gx, nameY, gw, nameH);
      const bottomDataUrl = cropRegion(offscreen, ctx, img, gx, bottomY, gw, bottomH);

      const [nameResult, bottomResult] = await Promise.all([
        worker.recognize(nameDataUrl),
        worker.recognize(bottomDataUrl),
      ]);

      return {
        cardName: parseCardName(nameResult.data.text),
        ...parseBottomStrip(bottomResult.data.text),
      };
    } finally {
      setLoading(false);
    }
  }, [ensureWorker]);

  const terminate = useCallback(async () => {
    if (workerRef.current) {
      await workerRef.current.terminate();
      workerRef.current = null;
    }
  }, []);

  return { runOcr, terminate, loading };
}
```

- [ ] **Step 2: Commit**

```bash
git add webui/src/Components/useScanOcr.js
git commit -m "feat(scan): add useScanOcr hook for parallel OCR on two card regions"
```

---

## Task 6: CameraFeed component

**Files:**
- Create: `webui/src/Components/CameraFeed.js`
- Modify: `webui/src/index.css`

Camera feed with a card-shaped alignment guide overlay. Uses the same `GUIDE` constants as `useScanOcr` so the overlay matches the crop region exactly.

- [ ] **Step 1: Create CameraFeed.js**

Create `webui/src/Components/CameraFeed.js`:
```jsx
import React, { useRef, useCallback } from "react";
import Webcam from "react-webcam";
import { GUIDE } from "./useScanOcr";

const VIDEO_CONSTRAINTS = {
  facingMode: "environment",
  aspectRatio: 9 / 16,
};

// Guide box CSS: widthFraction maps to vw, vertical center as vh
// These match the pixel math in useScanOcr — both derive from GUIDE constants.
const guideStyle = {
  position: "absolute",
  width: `${GUIDE.widthFraction * 100}vw`,
  aspectRatio: `${GUIDE.aspectRatio}`,
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",  // centered — matches GUIDE.centerYFraction: 0.50
  border: "2px solid rgba(100, 220, 100, 0.85)",
  borderRadius: "8px",
  boxShadow: "0 0 0 9999px rgba(0,0,0,0.45)",
  pointerEvents: "none",
  zIndex: 2,
};

export default function CameraFeed({ onCapture, disabled }) {
  const webcamRef = useRef(null);

  const handleCapture = useCallback(() => {
    if (!webcamRef.current || disabled) return;
    const screenshot = webcamRef.current.getScreenshot();
    if (screenshot) onCapture(screenshot);
  }, [onCapture, disabled]);

  return (
    <div className="scan-camera-container">
      <Webcam
        ref={webcamRef}
        audio={false}
        screenshotFormat="image/jpeg"
        screenshotQuality={0.92}
        videoConstraints={VIDEO_CONSTRAINTS}
        className="scan-webcam"
      />
      <div style={guideStyle} aria-label="Align card inside this box" />
      <button
        type="button"
        className="scan-capture-btn"
        onClick={handleCapture}
        disabled={disabled}
        aria-label="Capture card"
      >
        {disabled ? "Processing…" : "Capture"}
      </button>
    </div>
  );
}
```

- [ ] **Step 2: Add CSS for scan camera layout to index.css**

Append to `webui/src/index.css`:
```css
/* ── Scan view ─────────────────────────────────────────────── */
.scan-view {
    position: fixed;
    inset: 0;
    background: #000;
    display: flex;
    flex-direction: column;
    z-index: 800;
}

.scan-camera-container {
    position: relative;
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
}

.scan-webcam {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.scan-capture-btn {
    position: absolute;
    bottom: 2rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 3;
    padding: 0.9rem 2.8rem;
    border: none;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.9);
    color: #111;
    font-size: 1.1rem;
    font-weight: 700;
    cursor: pointer;
    transition: opacity 0.15s;
}

.scan-capture-btn:disabled {
    opacity: 0.5;
    cursor: default;
}

.scan-processing {
    position: absolute;
    inset: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.6);
    color: #fff;
    font-size: 1.2rem;
    font-weight: 600;
}

.scan-error-toast {
    position: absolute;
    bottom: 6rem;
    left: 1rem;
    right: 1rem;
    z-index: 10;
    padding: 1rem 1.25rem;
    border-radius: 0.75rem;
    background: #2a1a1a;
    border: 1px solid #7a3030;
    color: #f88;
    font-size: 0.95rem;
    text-align: center;
}

.scan-error-toast button {
    display: block;
    margin: 0.5rem auto 0;
    padding: 0.4rem 1.2rem;
    border: 1px solid #f88;
    border-radius: 999px;
    background: transparent;
    color: #f88;
    font-size: 0.9rem;
    cursor: pointer;
}
```

- [ ] **Step 3: Verify camera renders on mobile**

Open `/scan` on mobile. Camera permission dialog should appear. After granting, live camera feed should show with a green card-shaped guide box and a Capture button.

- [ ] **Step 4: Commit**

```bash
git add webui/src/Components/CameraFeed.js webui/src/index.css
git commit -m "feat(scan): CameraFeed component with alignment guide overlay"
```

---

## Task 7: ScanResultPanel component

**Files:**
- Create: `webui/src/Components/ScanResultPanel.js`
- Modify: `webui/src/index.css`

Bottom sheet that shows the matched card and an "Add to collection" button. Reuses the same `/collection/cards/{id}/add` API pattern as the rest of the app.

- [ ] **Step 1: Create ScanResultPanel.js**

Create `webui/src/Components/ScanResultPanel.js`:
```jsx
import React, { useState } from "react";
import { useOperations } from "../OperationsContext";
import { useCollections } from "./CollectionContext";
import { formatCents } from "./priceUtils";

function scryfallImageUrl(scryfallId) {
  return `https://api.scryfall.com/cards/${scryfallId}?format=image&version=normal`;
}

function displayPrice(price) {
  if (!price?.usd) return null;
  return formatCents(Math.round(parseFloat(price.usd) * 100));
}

export default function ScanResultPanel({ card, onDismiss, onScanAnother }) {
  const ops = useOperations();
  const collections = useCollections();
  const [added, setAdded] = useState(false);
  const [adding, setAdding] = useState(false);

  const scryfallId = card?.cardIdentifiers?.scryfallId;
  const imgSrc = scryfallId ? scryfallImageUrl(scryfallId) : null;
  const price = displayPrice(card?.price);

  // Use first collection by default — user can switch later from the collection view
  const targetCollection = collections?.[0]?.id ?? "Main";

  async function handleAdd() {
    if (adding || added) return;
    setAdding(true);
    try {
      await ops.fetch(
        `Adding ${card.name} to collection`,
        null,
        `/collection/cards/${encodeURIComponent(targetCollection)}/add`,
        {
          method: "post",
          headers: { "Content-Type": "application/json", Accept: "application/json" },
          body: JSON.stringify({
            id: card.id,
            collectionId: targetCollection,
            quantity: 1,
            foilQuantity: 0,
          }),
        }
      );
      setAdded(true);
    } finally {
      setAdding(false);
    }
  }

  return (
    <>
      <div className="mobile-card-sheet-backdrop" onClick={onDismiss} />
      <div className="scan-result-panel" role="dialog" aria-label="Card match">
        <div className="mobile-sheet-handle" />
        <div className="scan-result-content">
          {imgSrc && (
            <img className="scan-result-thumb" src={imgSrc} alt={card.name} />
          )}
          <div className="scan-result-info">
            <div className="scan-result-name">{card.name}</div>
            <div className="scan-result-set">
              {card.setCode} #{card.collectorNumber}
            </div>
            {price && <div className="scan-result-price">{price}</div>}
          </div>
          <button
            type="button"
            className={"scan-add-btn" + (added ? " added" : "")}
            onClick={handleAdd}
            disabled={adding || added}
          >
            {added ? "Added ✓" : adding ? "Adding…" : `+ Add to ${targetCollection}`}
          </button>
        </div>
        <button type="button" className="scan-another-btn" onClick={onScanAnother}>
          Scan another
        </button>
      </div>
    </>
  );
}
```

- [ ] **Step 2: Add ScanResultPanel CSS to index.css**

Append to `webui/src/index.css`:
```css
.scan-result-panel {
    position: fixed;
    right: 0;
    bottom: 0;
    left: 0;
    z-index: 1010;
    display: flex;
    flex-direction: column;
    border-radius: 1.1rem 1.1rem 0 0;
    background: var(--mobile-panel, #1c1e20);
    box-shadow: 0 -0.5rem 2.5rem rgba(0, 0, 0, 0.55);
    animation: sheet-slide-up 0.28s ease-out;
    padding-bottom: calc(1rem + env(safe-area-inset-bottom));
}

.scan-result-content {
    display: flex;
    align-items: center;
    gap: 0.9rem;
    padding: 0.75rem 1rem 0.5rem;
}

.scan-result-thumb {
    width: 52px;
    height: 72px;
    object-fit: cover;
    border-radius: 4px;
    flex-shrink: 0;
}

.scan-result-info {
    flex: 1;
    min-width: 0;
}

.scan-result-name {
    font-size: 1rem;
    font-weight: 700;
    color: #f1f3f5;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.scan-result-set {
    font-size: 0.8rem;
    color: #8a9bab;
    margin-top: 0.15rem;
}

.scan-result-price {
    font-size: 0.85rem;
    color: #ffa40a;
    margin-top: 0.1rem;
}

.scan-add-btn {
    flex-shrink: 0;
    padding: 0.55rem 1rem;
    border: none;
    border-radius: 999px;
    background: #ffa40a;
    color: #111;
    font-size: 0.9rem;
    font-weight: 700;
    cursor: pointer;
    white-space: nowrap;
}

.scan-add-btn.added {
    background: #2a7a2a;
    color: #9ef09e;
}

.scan-add-btn:disabled {
    opacity: 0.6;
    cursor: default;
}

.scan-another-btn {
    margin: 0.25rem 1rem 0.75rem;
    padding: 0.65rem;
    border: 1px solid rgba(255,255,255,0.12);
    border-radius: 999px;
    background: transparent;
    color: #c5c5c8;
    font-size: 0.95rem;
    cursor: pointer;
    width: calc(100% - 2rem);
}
```

- [ ] **Step 3: Commit**

```bash
git add webui/src/Components/ScanResultPanel.js webui/src/index.css
git commit -m "feat(scan): ScanResultPanel bottom sheet with add-to-collection"
```

---

## Task 8: PrintingPicker component

**Files:**
- Create: `webui/src/Components/PrintingPicker.js`
- Modify: `webui/src/index.css`

Shown when OCR identifies a card name but multiple printings exist. Horizontal scroll row of small card images.

- [ ] **Step 1: Create PrintingPicker.js**

Create `webui/src/Components/PrintingPicker.js`:
```jsx
import React from "react";

function scryfallImageUrl(scryfallId) {
  return `https://api.scryfall.com/cards/${scryfallId}?format=image&version=small`;
}

export default function PrintingPicker({ cards, onSelect, onDismiss }) {
  return (
    <>
      <div className="mobile-card-sheet-backdrop" onClick={onDismiss} />
      <div className="scan-result-panel" role="dialog" aria-label="Select printing">
        <div className="mobile-sheet-handle" />
        <p className="scan-picker-label">Multiple printings found — tap yours</p>
        <div className="scan-picker-scroll">
          {cards.map((card) => {
            const scryfallId = card?.cardIdentifiers?.scryfallId;
            return (
              <button
                key={card.id}
                type="button"
                className="scan-picker-card"
                onClick={() => onSelect(card)}
                aria-label={`${card.name} ${card.setCode} #${card.collectorNumber}`}
              >
                {scryfallId ? (
                  <img
                    src={scryfallImageUrl(scryfallId)}
                    alt={`${card.name} ${card.setCode}`}
                    loading="lazy"
                  />
                ) : (
                  <div className="scan-picker-placeholder">{card.setCode}</div>
                )}
                <span className="scan-picker-set">{card.setCode}</span>
              </button>
            );
          })}
        </div>
      </div>
    </>
  );
}
```

- [ ] **Step 2: Add PrintingPicker CSS to index.css**

Append to `webui/src/index.css`:
```css
.scan-picker-label {
    margin: 0.5rem 1rem 0.25rem;
    font-size: 0.9rem;
    color: #8a9bab;
    text-align: center;
}

.scan-picker-scroll {
    display: flex;
    gap: 0.75rem;
    padding: 0.5rem 1rem 1rem;
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
}

.scan-picker-scroll::-webkit-scrollbar {
    display: none;
}

.scan-picker-card {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    background: transparent;
    border: 2px solid transparent;
    border-radius: 6px;
    padding: 0.25rem;
    cursor: pointer;
}

.scan-picker-card img {
    width: 60px;
    height: 84px;
    object-fit: cover;
    border-radius: 4px;
}

.scan-picker-placeholder {
    width: 60px;
    height: 84px;
    border-radius: 4px;
    background: #2a2e31;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #8a9bab;
    font-size: 0.7rem;
}

.scan-picker-set {
    font-size: 0.7rem;
    color: #8a9bab;
}
```

- [ ] **Step 3: Commit**

```bash
git add webui/src/Components/PrintingPicker.js webui/src/index.css
git commit -m "feat(scan): PrintingPicker component for ambiguous printing selection"
```

---

## Task 9: ScanView orchestrator

**Files:**
- Modify: `webui/src/Views/ScanView.js`

Replaces the placeholder with the full state machine. States: `idle → processing → result | picking | error`.

- [ ] **Step 1: Replace ScanView.js with the full implementation**

Overwrite `webui/src/Views/ScanView.js`:
```jsx
import React, { useState, useCallback, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { CollectionsProvider } from "../Components/CollectionContext";
import CameraFeed from "../Components/CameraFeed";
import ScanResultPanel from "../Components/ScanResultPanel";
import PrintingPicker from "../Components/PrintingPicker";
import { useScanOcr } from "../Components/useScanOcr";
import {
  searchBySetAndNumber,
  searchByName,
  fuzzySearch,
} from "../Components/scanSearch";
import { MobileBottomNav } from "../Components/MobileCollectionView";

// State machine values
const IDLE = "idle";
const PROCESSING = "processing";
const RESULT = "result";
const PICKING = "picking";
const ERROR = "error";

export default function ScanView() {
  const [phase, setPhase] = useState(IDLE);
  const [matchedCard, setMatchedCard] = useState(null);
  const [candidates, setCandidates] = useState([]);
  const [errorMsg, setErrorMsg] = useState("");
  const { runOcr, terminate, loading: ocrLoading } = useScanOcr();
  const navigate = useNavigate();

  useEffect(() => () => terminate(), [terminate]);

  const handleCapture = useCallback(async (screenshotDataUrl) => {
    setPhase(PROCESSING);
    setErrorMsg("");

    try {
      const { cardName, setCode, collectorNumber } = await runOcr(screenshotDataUrl);

      // Step 1: exact match by set + collector number
      if (setCode && collectorNumber) {
        const bySet = await searchBySetAndNumber(setCode, collectorNumber);
        if (bySet && bySet.length === 1) {
          setMatchedCard(bySet[0]);
          setPhase(RESULT);
          return;
        }
      }

      // Step 2: exact name search
      if (cardName) {
        const byName = await searchByName(cardName);
        if (byName && byName.length === 1) {
          setMatchedCard(byName[0]);
          setPhase(RESULT);
          return;
        }
        if (byName && byName.length > 1) {
          setCandidates(byName);
          setPhase(PICKING);
          return;
        }
      }

      // Step 3: Scryfall fuzzy search
      const fuzzyText = cardName || (setCode && collectorNumber ? `${setCode} ${collectorNumber}` : null);
      if (fuzzyText) {
        const byFuzzy = await fuzzySearch(fuzzyText);
        if (byFuzzy && byFuzzy.length === 1) {
          setMatchedCard(byFuzzy[0]);
          setPhase(RESULT);
          return;
        }
        if (byFuzzy && byFuzzy.length > 1) {
          setCandidates(byFuzzy);
          setPhase(PICKING);
          return;
        }
      }

      // Step 4: nothing found
      setErrorMsg("Couldn't identify card. Try holding it flatter and closer.");
      setPhase(ERROR);
    } catch (err) {
      setErrorMsg("Something went wrong: " + err.message);
      setPhase(ERROR);
    }
  }, [runOcr]);

  function handleScanAnother() {
    setPhase(IDLE);
    setMatchedCard(null);
    setCandidates([]);
    setErrorMsg("");
  }

  function handlePickerSelect(card) {
    setMatchedCard(card);
    setCandidates([]);
    setPhase(RESULT);
  }

  function goToSearch() {
    navigate("/search");
  }

  const isProcessing = phase === PROCESSING || ocrLoading;

  return (
    <CollectionsProvider>
      <div className="scan-view">
        <CameraFeed onCapture={handleCapture} disabled={isProcessing} />

        {isProcessing && (
          <div className="scan-processing" aria-live="polite">
            Reading card…
          </div>
        )}

        {phase === ERROR && (
          <div className="scan-error-toast" role="alert">
            {errorMsg}
            <button type="button" onClick={goToSearch}>
              Search manually
            </button>
            <button type="button" onClick={handleScanAnother} style={{ marginTop: "0.25rem" }}>
              Try again
            </button>
          </div>
        )}

        {phase === RESULT && matchedCard && (
          <ScanResultPanel
            card={matchedCard}
            onDismiss={handleScanAnother}
            onScanAnother={handleScanAnother}
          />
        )}

        {phase === PICKING && candidates.length > 0 && (
          <PrintingPicker
            cards={candidates}
            onSelect={handlePickerSelect}
            onDismiss={handleScanAnother}
          />
        )}

        <MobileBottomNav activeTab="scan" />
      </div>
    </CollectionsProvider>
  );
}
```

- [ ] **Step 2: Run the app and do an end-to-end test**

```bash
cd webui && npm start
```

On mobile:
1. Tap Scan → camera opens with green guide
2. Hold a standard MtG card in the guide box
3. Tap Capture
4. "Reading card…" overlay appears
5. ScanResultPanel slides up with card name, set, price, Add button
6. Tap Add → button changes to "Added ✓"
7. Tap "Scan another" → returns to camera

- [ ] **Step 3: Test the alternate-art path (The One Ring)**

Hold the borderless alternate-art One Ring card. OCR will fail on the name but should read `LTR 0748` from the bottom strip and still return the correct printing.

- [ ] **Step 4: Test the retro-frame path (Stitcher's Supplier)**

Hold the retro-frame Stitcher's Supplier. Bottom strip won't give a set code. PrintingPicker should appear with all Stitcher's Supplier printings to choose from.

- [ ] **Step 5: Test the OCR failure path**

Cover the card partially. Error toast should appear with "Try again" and "Search manually" buttons.

- [ ] **Step 6: Commit**

```bash
git add webui/src/Views/ScanView.js
git commit -m "feat(scan): ScanView state machine with full 5-step fallback chain"
```

---

## Task 10: Final polish and cleanup

**Files:**
- Modify: `webui/src/index.css` (ensure `--mobile-panel` variable is accessible in scan view)
- Modify: `webui/src/Components/MobileCollectionView.js` (ensure `activeTab="scan"` propagates)

- [ ] **Step 1: Verify `--mobile-panel` CSS variable is defined globally**

```bash
grep -n "\-\-mobile-panel" webui/src/index.css | head -5
```

If not defined at `:root` level, add to the top of the scan section in index.css:
```css
:root {
    --mobile-panel: #1c1e20;
    --mobile-accent: #ffa40a;
}
```

- [ ] **Step 2: Run the full test suite**

```bash
cd webui && npm test -- --watchAll=false
```

Expected: all tests pass including the scanSearch tests from Tasks 3–4.

- [ ] **Step 3: Final commit**

```bash
git add webui/src/index.css
git commit -m "feat(scan): finalize scan MVP - camera OCR card identification"
```

---

## Edge Case Reference

| Scenario | Expected behaviour |
|---|---|
| Alternate art (One Ring) — name unreadable | Bottom strip `LTR 0748` → exact match |
| Retro frame (Stitcher's Supplier) — no set code | Name OCR → PrintingPicker |
| OCR typo (e.g. "Lightnig Helix") | Scryfall fuzzy → resolves to correct name → result |
| Card partially covered | Error toast → Try again / Search manually |
| No network (Scryfall down) | fuzzySearch returns null → error toast |
| Camera permission denied | react-webcam renders fallback error; user sees black screen + browser permission prompt |
