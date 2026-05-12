import React, { act } from "react";
import { createRoot } from "react-dom/client";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { ModeProvider, OperationsProvider } from "../OperationsContext";
import CardListView from "./CardListView";

globalThis.IS_REACT_ACT_ENVIRONMENT = true;

function mockMatchMedia(matches) {
  window.matchMedia = jest.fn().mockImplementation((query) => ({
    matches,
    media: query,
    onchange: null,
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    addListener: jest.fn(),
    removeListener: jest.fn(),
    dispatchEvent: jest.fn(),
  }));
}

function jsonResponse(body) {
  return Promise.resolve({
    ok: true,
    status: 200,
    text: () => Promise.resolve(JSON.stringify(body)),
  });
}

function mockFetch() {
  global.fetch = jest.fn((url) => {
    if (url === "/system") {
      return jsonResponse({ system: "MagicSQLite", systems: ["MagicSQLite"], downloading: {} });
    }
    if (url === "/mtg/sets") {
      return jsonResponse([]);
    }
    if (url === "/collection/list") {
      return jsonResponse([
        { id: "A1", canRemove: true, isProxy: false },
        { id: "A2", canRemove: true, isProxy: false },
      ]);
    }
    if (url === "/collection/stats") {
      return jsonResponse({
        totalValueCents: 411000,
        changeCents: 111000,
        changePercent: 37,
        copyCount: 2616,
        pricedCopyCount: 2616,
      });
    }
    if (String(url).startsWith("/collection/cards/")) {
      const id = decodeURIComponent(String(url).split("/")[3]);
      return jsonResponse({
        totalValueCents: id === "A1" ? 64300 : 45100,
        changeCents: id === "A1" ? 8910 : 2249,
        changePercent: id === "A1" ? 16.1 : 5.24,
        copyCount: id === "A1" ? 893 : 361,
        pricedCopyCount: id === "A1" ? 893 : 361,
      });
    }
    return jsonResponse(null);
  });
}

async function renderMobileCollectionView() {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  await act(async () => {
    root.render(
      <MemoryRouter
        initialEntries={["/collections/1"]}
        future={{ v7_relativeSplatPath: true, v7_startTransition: true }}
      >
        <ModeProvider mode="full" collectionsEnabled>
          <OperationsProvider>
            <Routes>
              <Route path="/collections/:pageNumber" element={<CardListView />} />
            </Routes>
          </OperationsProvider>
        </ModeProvider>
      </MemoryRouter>,
    );
  });

  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });

  return {
    container,
    cleanup: () => {
      act(() => {
        root.unmount();
      });
      container.remove();
    },
  };
}

describe("CardListView mobile layout", () => {
  beforeEach(() => {
    Object.defineProperty(global, "crypto", {
      configurable: true,
      value: { randomUUID: jest.fn(() => "test-operation-id") },
    });
    mockMatchMedia(true);
    mockFetch();
  });

  afterEach(() => {
    jest.restoreAllMocks();
    delete global.crypto;
    delete global.fetch;
  });

  it("shows the mobile collection overview on collection routes", async () => {
    const { container, cleanup } = await renderMobileCollectionView();

    expect(container.querySelector(".mobile-collection-app")).not.toBeNull();
    expect(container.querySelector("input[placeholder='Search binders']")).not.toBeNull();
    expect(container.textContent).toContain("A1");
    expect(container.textContent).toContain("$4.11K");

    cleanup();
  });
});
