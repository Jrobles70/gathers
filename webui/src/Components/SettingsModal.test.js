import React, { act } from "react";
import { createRoot } from "react-dom/client";
import SettingsModal from "./SettingsModal";
import { OperationsContext } from "../OperationsContext";

globalThis.IS_REACT_ACT_ENVIRONMENT = true;

function renderSettings({ fetch = jest.fn(), debugEnabled = false, setDebugEnabled = jest.fn() } = {}) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  act(() => {
    root.render(
      <OperationsContext.Provider value={{ fetch, debugEnabled, setDebugEnabled }}>
        <SettingsModal open={true} onClose={() => {}} />
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

describe("SettingsModal", () => {
  it("stores the debug log setting in operations context", () => {
    const setDebugEnabled = jest.fn();
    const { container, cleanup } = renderSettings({ setDebugEnabled });

    act(() => {
      container.querySelector("input[type='checkbox']").dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(setDebugEnabled).toHaveBeenCalledWith(true);

    cleanup();
  });

  it("triggers the MTG update endpoint", () => {
    const fetch = jest.fn(() => Promise.resolve("MTG update started"));
    const { container, cleanup } = renderSettings({ fetch });

    act(() => {
      Array.from(container.querySelectorAll("button"))
        .find((button) => button.textContent === "Check MTG Updates")
        .dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(fetch).toHaveBeenCalledWith("Checking MTG card updates", "", "/mtg/update");

    cleanup();
  });
});
