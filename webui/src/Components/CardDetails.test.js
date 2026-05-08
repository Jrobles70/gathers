import React, { act } from "react";
import { createRoot } from "react-dom/client";
import CardDetails from "./CardDetails";
import { ModeProvider } from "../OperationsContext";

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
    cleanup: () => {
      act(() => {
        root.unmount();
      });
      container.remove();
    },
  };
}

describe("CardDetails", () => {
  it("does not expose collection add controls as form submit buttons", () => {
    const { container, cleanup } = renderIntoForm(
      <ModeProvider collectionsEnabled={true}>
        <CardDetails id="card-1" toggleSelected={() => {}} />
      </ModeProvider>,
    );

    const buttons = Array.from(container.querySelectorAll("button"));

    expect(buttons.map((button) => button.textContent.trim())).toEqual([
      "Add",
      "Add Foil",
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
});
