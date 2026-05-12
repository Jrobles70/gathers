import React from "react";
import { Link, useLocation, useSearchParams } from "react-router-dom";
import { useMode } from "../OperationsContext";
import { useQuickSearch } from "./QuickSearchContext";

const SORT_FIELDS = [
  { value: "Name",            label: "Name" },
  { value: "SetCode",         label: "Set" },
  { value: "Rarity",         label: "Rarity" },
  { value: "CollectorNumber", label: "No." },
  { value: "PurchasePrice",   label: "Price" },
  { value: "TimeAdded",       label: "Added" },
];

const SORT_FIELD_VALUES = new Set(SORT_FIELDS.map(({ value }) => value));

function SortBar() {
  const [searchParams, setSearchParams] = useSearchParams();
  const sortBy = (() => {
    const v = searchParams.get("cf_sortBy");
    return SORT_FIELD_VALUES.has(v) ? v : "Name";
  })();
  const sortOrder = searchParams.get("cf_sortOrder") ?? "Asc";
  const viewMode = searchParams.get("cf_viewMode") ?? "grid";
  const proxyMode = searchParams.get("cf_proxy") ?? "all";

  const setParam = (key, value) => {
    const next = new URLSearchParams(searchParams);
    if (value === "" || value === null || (key === "cf_proxy" && value === "all")) {
      next.delete(key);
    } else {
      next.set(key, value);
    }
    next.set("page", "1");
    setSearchParams(next);
  };

  const handleSort = (field) => {
    const next = new URLSearchParams(searchParams);
    if (field === sortBy) {
      next.set("cf_sortOrder", sortOrder === "Asc" ? "Desc" : "Asc");
    } else {
      next.set("cf_sortBy", field);
      next.set("cf_sortOrder", "Asc");
    }
    next.set("page", "1");
    setSearchParams(next);
  };

  return (
    <div className="top-bar-sort-controls">
      <div className="top-bar-sort-group">
        <span className="top-bar-sort-label">Sort</span>
        <div className="btn-group btn-group-sm" role="group" aria-label="Sort by">
          {SORT_FIELDS.map(({ value, label }) => {
            const active = sortBy === value;
            return (
              <button
                key={value}
                type="button"
                onClick={() => handleSort(value)}
                className={`btn ${active ? "btn-secondary" : "btn-outline-secondary"}`}
              >
                {label}
                {active && <span className="ms-1">{sortOrder === "Asc" ? "↑" : "↓"}</span>}
              </button>
            );
          })}
        </div>
      </div>

      <div className="top-bar-sort-group">
        <span className="top-bar-sort-label">Proxy</span>
        <div className="btn-group btn-group-sm" role="group" aria-label="Proxy filter">
          {[["all", "All"], ["regular", "Regular"], ["proxy", "Proxy"]].map(([value, label]) => (
            <button
              key={value}
              type="button"
              className={`btn ${proxyMode === value ? "btn-secondary" : "btn-outline-secondary"}`}
              onClick={() => setParam("cf_proxy", value)}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      <div className="top-bar-sort-group">
        <span className="top-bar-sort-label">View</span>
        <div className="btn-group btn-group-sm" role="group" aria-label="View mode">
          <button
            type="button"
            className={`btn ${viewMode === "grid" ? "btn-secondary" : "btn-outline-secondary"}`}
            onClick={() => setParam("cf_viewMode", "grid")}
          >
            Grid
          </button>
          <button
            type="button"
            className={`btn ${viewMode === "list" ? "btn-secondary" : "btn-outline-secondary"}`}
            onClick={() => setParam("cf_viewMode", "list")}
          >
            List
          </button>
        </div>
      </div>
    </div>
  );
}

export default function TopBar() {
  const { mode, collectionsEnabled } = useMode();
  const isSearchOnly = mode === "search-only";
  const { openQuickSearch } = useQuickSearch();
  const location = useLocation();

  const isCollectionPage =
    location.pathname.startsWith("/c/") || location.pathname.startsWith("/collections");

  const showCollectionTools = !isSearchOnly && collectionsEnabled && isCollectionPage;

  return (
    <div className="top-bar" data-bs-theme="dark">
      <div className="top-bar-left">
        <Link className="top-bar-brand" to={collectionsEnabled && !isSearchOnly ? "/collections/1" : "/search"}>
          GatheRs
        </Link>
        <div className="top-bar-actions">
          <Link to="/search" className="btn btn-sm btn-outline-secondary">
            Search
          </Link>
          {!isSearchOnly && collectionsEnabled && (
            <button type="button" className="btn btn-sm btn-primary" onClick={openQuickSearch}>
              Quick Add
            </button>
          )}
          {!isSearchOnly && collectionsEnabled && (
            <Link
              to="/duplicates"
              className={"btn btn-sm btn-outline-info" + (location.pathname.startsWith("/duplicates") ? " active" : "")}
            >
              Duplicates
            </Link>
          )}
        </div>
      </div>

      {showCollectionTools && <SortBar />}
    </div>
  );
}
