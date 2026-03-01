import React from "react";

export default function QuickSearch({ onToggle, isOpen }) {
  return (
    <div className="d-flex">
      <button
        className="btn btn-outline-info"
        type="button"
        onClick={onToggle}
      >
        {isOpen ? "Close Search" : "Quick Search"}
      </button>
    </div>
  );
}
