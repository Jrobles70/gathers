import React, { act } from "react";
import { createRoot } from "react-dom/client";
import {
  MemoryRouter,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";
import CollectionFilterBar, { collectionFiltersActive } from "./CollectionFilterBar";

globalThis.IS_REACT_ACT_ENVIRONMENT = true;

function LocationOutput() {
  const location = useLocation();
  return <output aria-label="location">{location.search}</output>;
}

function renderFilterBar(initialEntry = "/collections/1") {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  act(() => {
    root.render(
      <MemoryRouter
        initialEntries={[initialEntry]}
        future={{ v7_relativeSplatPath: true, v7_startTransition: true }}
      >
        <Routes>
          <Route
            path="/collections/:pageNumber"
            element={(
              <>
                <CollectionFilterBar />
                <LocationOutput />
              </>
            )}
          />
        </Routes>
      </MemoryRouter>,
    );
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

describe("CollectionFilterBar", () => {
  it("tracks proxy filters through query params", () => {
    const { container, cleanup } = renderFilterBar();

    const openFilters = Array.from(container.querySelectorAll("button"))
      .find((button) => button.textContent.includes("Filters"));

    act(() => {
      openFilters.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    const proxyButton = Array.from(container.querySelectorAll("button"))
      .find((button) => button.textContent.trim() === "Proxy");

    act(() => {
      proxyButton.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(container.querySelector("output[aria-label='location']").textContent)
      .toContain("cf_proxy=proxy");
    expect(container.querySelector("output[aria-label='location']").textContent)
      .toContain("page=1");

    const allButton = Array.from(container.querySelectorAll("button"))
      .find((button) => button.textContent.trim() === "All");

    act(() => {
      allButton.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(container.querySelector("output[aria-label='location']").textContent)
      .not.toContain("cf_proxy");

    cleanup();
  });

  it("treats non-default proxy filters as active", () => {
    expect(collectionFiltersActive({
      name: "",
      setCode: "",
      rarity: "",
      artist: "",
      text: "",
      colorIdentities: [],
      proxyMode: "proxy",
    })).toBe(true);
  });
});
