import React, { useState } from "react";
import ImportCards from "./CardListNavButtons/ImportCards";
import DeleteCollection from "./CardListNavButtons/DeleteCollection";
import ExportCollection from "./CardListNavButtons/ExportCollection";
import CollectionStats from "./CollectionStats";
import RenameCollection from "./CardListNavButtons/RenameCollection";
import CollectionProxyToggle from "./CardListNavButtons/CollectionProxyToggle";

export default function CardListNav() {
  const [otherOptionsOpen, setOtherOptionsOpen] = useState(false);

  return (
    <nav className="collection-action-panel" data-bs-theme="dark" aria-label="Collection actions">
      <CollectionStats />

      {/* Bulk collection actions are hidden until card selection works in the default grid view. */}

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
            <RenameCollection />
            <CollectionProxyToggle />
            <ExportCollection />
            <DeleteCollection />
          </div>
        )}
      </section>
    </nav>
  );
}
