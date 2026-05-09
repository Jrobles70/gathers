import React, { useState } from "react";
import SelectionTracker from "./CardListNavButtons/SelectionTracker";
import DeleteCards from "./CardListNavButtons/DeleteCards";
import MoveCards from "./CardListNavButtons/MoveCards";
import ImportCards from "./CardListNavButtons/ImportCards";
import DeleteCollection from "./CardListNavButtons/DeleteCollection";
import ExportCollection from "./CardListNavButtons/ExportCollection";

export default function CardListNav() {
  const [otherOptionsOpen, setOtherOptionsOpen] = useState(false);

  return (
    <nav className="collection-action-panel" data-bs-theme="dark" aria-label="Collection actions">
      <section className="collection-panel-section">
        <div className="collection-panel-heading">Collection</div>
        <div className="collection-selection-actions">
          <SelectionTracker />
          <DeleteCards />
        </div>
        <MoveCards />
      </section>

      <section className="collection-panel-section">
        <ImportCards />
      </section>

      <section className="collection-panel-section">
        <button
          type="button"
          className="collection-panel-toggle"
          aria-expanded={otherOptionsOpen}
          onClick={() => setOtherOptionsOpen((open) => !open)}
        >
          <span>Other options</span>
          <span aria-hidden="true">{otherOptionsOpen ? "^" : "v"}</span>
        </button>
        {otherOptionsOpen && (
          <div className="collection-panel-dropdown">
            <ExportCollection />
            <DeleteCollection />
          </div>
        )}
      </section>
    </nav>
  );
}
