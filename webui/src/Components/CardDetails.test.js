import React, { act } from "react";
import { createRoot } from "react-dom/client";
import CardDetails, {
  applyQuantityDelta,
  resolveCardUpdateCollection,
} from "./CardDetails";
import { ModeProvider, OperationsContext } from "../OperationsContext";

globalThis.IS_REACT_ACT_ENVIRONMENT = true;

function renderIntoForm(ui) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  act(() => {
    root.render(<form>{ui}</form>);
  });

  return {
    container,
    rerender: (nextUi) => {
      act(() => {
        root.render(<form>{nextUi}</form>);
      });
    },
    cleanup: () => {
      act(() => {
        root.unmount();
      });
      container.remove();
    },
  };
}

describe("CardDetails", () => {
  it("uses the quick search target collection when adding unowned cards", () => {
    expect(resolveCardUpdateCollection({
      details: null,
      showCollectionSelect: false,
      selectedCollection: null,
      collections: [{ id: "A1" }, { id: "Default" }],
      currentCollection: "A1",
      targetCollection: "Default",
    })).toBe("Default");
  });

  it("does not expose collection add controls as form submit buttons", () => {
    const { container, cleanup } = renderIntoForm(
      <ModeProvider collectionsEnabled={true}>
        <CardDetails id="card-1" toggleSelected={() => {}} />
      </ModeProvider>,
    );

    const buttons = Array.from(container.querySelectorAll("button"));

    expect(buttons.map((button) => button.textContent.trim())).toEqual([
      "+",
      "-",
      "...",
    ]);
    expect(buttons.every((button) => button.type === "button")).toBe(true);

    cleanup();
  });

  it("does not expose quantity controls as form submit buttons", () => {
    const { container, cleanup } = renderIntoForm(
      <CardDetails
        id="card-1"
        details={{ collectionId: "Main", quantity: 1, foilQuantity: 0 }}
        toggleSelected={() => {}}
      />,
    );

    const buttons = Array.from(container.querySelectorAll("button"));

    expect(buttons.every((button) => button.type === "button")).toBe(true);

    cleanup();
  });

  it("shows foil mode in the card action menu", () => {
    const { container, cleanup } = renderIntoForm(
      <ModeProvider collectionsEnabled={true}>
        <CardDetails id="card-1" details={{ collectionId: "Main", quantity: 1, foilQuantity: 0 }} />
      </ModeProvider>,
    );

    const menuButton = Array.from(container.querySelectorAll("button"))
      .find((button) => button.textContent.trim() === "...");

    act(() => {
      menuButton.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(container.querySelector(".search-card-menu").textContent).toContain("Foil");
    expect(Array.from(container.querySelectorAll("button")).every((button) => button.type === "button")).toBe(true);

    cleanup();
  });

  it("applies quantity deltas without dropping below zero", () => {
    expect(applyQuantityDelta({ quantity: 1, foilQuantity: 0 }, -2, 1)).toEqual({
      quantity: 0,
      foilQuantity: 1,
    });
  });

  it("keeps local quantities scoped to the active printing id", async () => {
    const fetchMock = jest.fn().mockResolvedValue([{ id: "printing-a" }]);
    const renderCardDetails = (id) => (
      <OperationsContext.Provider value={{ fetch: fetchMock }}>
        <ModeProvider collectionsEnabled={true}>
          <CardDetails id={id} />
        </ModeProvider>
      </OperationsContext.Provider>
    );

    const { container, rerender, cleanup } = renderIntoForm(renderCardDetails("printing-a"));
    const quantityOutput = () => container.querySelector(".search-card-quantity").textContent.trim();
    const increaseButton = () => Array.from(container.querySelectorAll("button"))
      .find((button) => button.textContent.trim() === "+");

    expect(quantityOutput()).toBe("0");

    await act(async () => {
      increaseButton().dispatchEvent(new MouseEvent("click", { bubbles: true }));
      await Promise.resolve();
    });

    expect(fetchMock).toHaveBeenCalledWith(
      "Updating quantities for card printing-a",
      {},
      "/collection/cards/Main/add",
      expect.objectContaining({
        body: JSON.stringify({
          id: "printing-a",
          collectionId: "Main",
          quantity: 1,
          foilQuantity: 0,
        }),
      }),
    );
    expect(quantityOutput()).toBe("1");

    rerender(renderCardDetails("printing-b"));
    expect(quantityOutput()).toBe("0");

    rerender(renderCardDetails("printing-a"));
    expect(quantityOutput()).toBe("1");

    cleanup();
  });
});
