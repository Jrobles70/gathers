import React, { act } from "react";
import { createRoot } from "react-dom/client";
import { OperationsContext } from "../../OperationsContext";
import OperationsTracker from "./OperationsTracker";

globalThis.IS_REACT_ACT_ENVIRONMENT = true;

function renderTracker(contextValue) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  act(() => {
    root.render(
      <OperationsContext.Provider value={contextValue}>
        <OperationsTracker />
      </OperationsContext.Provider>,
    );
  });

  return {
    container,
    cleanup: () => {
      act(() => root.unmount());
      container.remove();
    },
  };
}

describe("OperationsTracker", () => {
  it("hides the debug operation tracker unless debug is enabled", () => {
    const { container, cleanup } = renderTracker({
      debugEnabled: false,
      operations: {},
      operationLogs: [],
    });

    expect(container.textContent).not.toContain("0 operations active");
    expect(container.querySelector("[aria-label='Operation logs']")).toBeNull();

    cleanup();
  });

  it("shows only the debug log list when debug is enabled", () => {
    const { container, cleanup } = renderTracker({
      debugEnabled: true,
      operations: {
        importing: { message: "Importing cards" },
      },
      operationLogs: [],
    });

    expect(container.textContent).not.toContain("operations active");
    expect(container.textContent).not.toContain("Importing cards");
    expect(container.querySelector("[aria-label='Operation logs']")).not.toBeNull();

    cleanup();
  });
});
