import React, { act, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { OperationsContext } from "../../OperationsContext";
import { CardCacheProvider } from "./CardCacheContext";
import { CardLoaderProvider, useCardLoader } from "./CardLoaderContext";

globalThis.IS_REACT_ACT_ENVIRONMENT = true;

function renderLoader(ui, operations) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  act(() => {
    root.render(
      <OperationsContext.Provider value={operations}>
        <CardCacheProvider>
          <CardLoaderProvider>{ui}</CardLoaderProvider>
        </CardCacheProvider>
      </OperationsContext.Provider>,
    );
  });

  return {
    cleanup: () => {
      act(() => {
        root.unmount();
      });
      container.remove();
    },
  };
}

async function flushPromises() {
  await Promise.resolve();
  await Promise.resolve();
}

describe("CardLoaderProvider", () => {
  it("loads card details quietly and keeps a stable loader after cache updates", async () => {
    const trackedFetch = jest.fn();
    const quietFetch = jest.fn().mockResolvedValue({
      "card-a": { id: "card-a", name: "Card A" },
    });
    const loaderIdentities = [];

    function LoaderConsumer() {
      const loadCard = useCardLoader();
      loaderIdentities.push(loadCard);

      useEffect(() => {
        loadCard("card-a", "MagicSQLite");
      }, [loadCard]);

      return null;
    }

    const { cleanup } = renderLoader(
      <LoaderConsumer />,
      { fetch: trackedFetch, quietFetch },
    );

    await act(async () => {
      await flushPromises();
    });

    expect(quietFetch).toHaveBeenCalledWith({}, "/mtg/cards?ids=card-a");
    expect(trackedFetch).not.toHaveBeenCalled();
    expect(new Set(loaderIdentities).size).toBe(1);

    cleanup();
  });
});
