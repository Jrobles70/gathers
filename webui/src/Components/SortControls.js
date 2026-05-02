import React from "react";

export default function SortControls({ sortBy, sortOrder, fields, onChange }) {
  const handleClick = (field) => {
    if (field === sortBy) {
      onChange(field, sortOrder === "Asc" ? "Desc" : "Asc");
    } else {
      onChange(field, "Asc");
    }
  };

  return (
    <div className="d-flex align-items-center gap-1 flex-wrap">
      <small className="text-muted me-1">Sort:</small>
      <div className="btn-group btn-group-sm" role="group" aria-label="Sort by">
        {fields.map(({ value, label }) => {
          const active = sortBy === value;
          return (
            <button
              key={value}
              type="button"
              onClick={() => handleClick(value)}
              className={`btn ${active ? "btn-secondary" : "btn-outline-secondary"}`}
              title={`Sort by ${label}`}
            >
              {label}
              {active && (
                <span className="ms-1">{sortOrder === "Asc" ? "↑" : "↓"}</span>
              )}
            </button>
          );
        })}
      </div>
    </div>
  );
}
