import React, { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useSystems } from "./SystemTypeContext";
import { ReactComponent as ColorlessSymbol } from "../assets/card-symbols/C.svg";
import "mana-font/css/mana.min.css";

const MTG_COLORS = [
  { value: "White",      mana: "w", bg: "#f9faf4", border: "#c4b78a" },
  { value: "Blue",       mana: "u", bg: "#0e68ab", border: "#0a4f82" },
  { value: "Black",      mana: "b", bg: "#2a2a2a", border: "#555" },
  { value: "Red",        mana: "r", bg: "#d3202a", border: "#a01820" },
  { value: "Green",      mana: "g", bg: "#00733e", border: "#005a30" },
  { value: "Colorless",  Symbol: ColorlessSymbol, bg: "#cac5c0", border: "#8f8780", title: "Colorless" },
];

const SYSTEM_LABELS = {
  MagicSQLite: "MTG",
  RiftboundSQLite: "Riftbound",
  PokemonSQLite: "Pokémon",
  Scryfall: "Scryfall",
};

const VALID_SORT_FIELDS = new Set(["Name", "SetCode", "Rarity", "CollectorNumber", "PurchasePrice", "TimeAdded"]);

const SORT_FIELDS = [
  { value: "Name",            label: "Name" },
  { value: "SetCode",         label: "Set" },
  { value: "Rarity",          label: "Rarity" },
  { value: "CollectorNumber", label: "No." },
  { value: "PurchasePrice",   label: "Price" },
  { value: "TimeAdded",       label: "Added" },
];

function readFilters(searchParams) {
  const sortBy = searchParams.get("cf_sortBy");

  return {
    name:            searchParams.get("cf_name") ?? "",
    setCode:         searchParams.get("cf_setCode") ?? "",
    rarity:          searchParams.get("cf_rarity") ?? "",
    artist:          searchParams.get("cf_artist") ?? "",
    text:            searchParams.get("cf_text") ?? "",
    provider:        searchParams.get("cf_provider") ?? "",
    sortBy:          VALID_SORT_FIELDS.has(sortBy) ? sortBy : "Name",
    sortOrder:       searchParams.get("cf_sortOrder") ?? "Asc",
    viewMode:        searchParams.get("cf_viewMode") ?? "grid",
    proxyMode:       searchParams.get("cf_proxy") ?? "all",
    colorIdentities: searchParams.getAll("cf_color"),
    domains:         [],
    energyTypes:     [],
  };
}

export function useCollectionFilters() {
  const [searchParams] = useSearchParams();
  return readFilters(searchParams);
}

export function collectionFiltersActive(filters) {
  return (
    filters.name !== "" ||
    filters.setCode !== "" ||
    filters.rarity !== "" ||
    filters.artist !== "" ||
    filters.text !== "" ||
    filters.colorIdentities.length > 0 ||
    filters.proxyMode !== "all"
  );
}

export default function CollectionFilterBar() {
  const systems = useSystems();
  const [searchParams, setSearchParams] = useSearchParams();
  const [open, setOpen] = useState(false);
  const filters = readFilters(searchParams);

  const setFilter = (key, value) => {
    const next = new URLSearchParams(searchParams);
    if (value === "" || value === null || (key === "cf_proxy" && value === "all")) {
      next.delete(key);
    } else {
      next.set(key, value);
    }
    next.set("page", "1");
    setSearchParams(next);
  };

  const setArrayFilter = (key, value, checked) => {
    const next = new URLSearchParams(searchParams);
    const existing = next.getAll(key).filter((v) => v !== value);
    next.delete(key);
    const updated = checked ? [...existing, value] : existing;
    updated.forEach((v) => next.append(key, v));
    next.set("page", "1");
    setSearchParams(next);
  };

  const clearFilters = () => {
    const next = new URLSearchParams(searchParams);
    ["cf_name","cf_setCode","cf_rarity","cf_artist","cf_text","cf_provider",
     "cf_sortBy","cf_sortOrder","cf_proxy","cf_color","cf_domain","cf_energy","page"].forEach((k) => next.delete(k));
    setSearchParams(next);
  };

  const hasActive = collectionFiltersActive(filters);

  const handleSort = (field) => {
    const next = new URLSearchParams(searchParams);
    if (field === filters.sortBy) {
      next.set("cf_sortOrder", filters.sortOrder === "Asc" ? "Desc" : "Asc");
    } else {
      next.set("cf_sortBy", field);
      next.set("cf_sortOrder", "Asc");
    }
    next.set("page", "1");
    setSearchParams(next);
  };

  return (
    <section className="collection-filter-bar collection-panel-section" data-bs-theme="dark">
      <div className="collection-sort-section">
        <div className="collection-filter-row">
          <span className="collection-filter-label">Sort</span>
          <div className="collection-filter-btn-group collection-sort-btn-group">
            {SORT_FIELDS.map(({ value, label }) => {
              const active = filters.sortBy === value;
              return (
                <button
                  key={value}
                  type="button"
                  onClick={() => handleSort(value)}
                  className={`btn btn-sm ${active ? "btn-secondary" : "btn-outline-secondary"}`}
                >
                  {label}
                  {active && <span className="ms-1">{filters.sortOrder === "Asc" ? "↑" : "↓"}</span>}
                </button>
              );
            })}
          </div>
        </div>

        <div className="collection-filter-row">
          <span className="collection-filter-label">View</span>
          <div className="collection-filter-btn-group">
            {[["grid", "Grid"], ["list", "List"]].map(([value, label]) => (
              <button
                key={value}
                type="button"
                className={`btn btn-sm ${filters.viewMode === value ? "btn-secondary" : "btn-outline-secondary"}`}
                onClick={() => setFilter("cf_viewMode", value)}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      </div>

      <button
        className="collection-panel-toggle"
        aria-expanded={open}
        onClick={() => setOpen((o) => !o)}
        type="button"
      >
        <span className="collection-panel-toggle-label">
          Filters
          {hasActive && <span className="filter-active-dot" aria-label="Active filters" />}
        </span>
        <span aria-hidden="true">{open ? "^" : "v"}</span>
      </button>

      {open && (
        <div className="collection-panel-dropdown collection-filters-dropdown">
          <div className="collection-filter-field">
            <label htmlFor="collection-filter-name">Name</label>
            <input
              type="text"
              className="form-control form-control-sm"
              id="collection-filter-name"
              placeholder="Name"
              value={filters.name}
              onChange={(e) => setFilter("cf_name", e.target.value)}
            />
          </div>

          <div className="collection-filter-field">
            <label htmlFor="collection-filter-set-code">Set code</label>
            <input
              type="text"
              className="form-control form-control-sm"
              id="collection-filter-set-code"
              placeholder="Set code"
              value={filters.setCode}
              onChange={(e) => setFilter("cf_setCode", e.target.value)}
            />
          </div>

          <div className="collection-filter-field">
            <label htmlFor="collection-filter-rarity">Rarity</label>
            <select
              className="form-select form-select-sm"
              id="collection-filter-rarity"
              value={filters.rarity}
              onChange={(e) => setFilter("cf_rarity", e.target.value)}
            >
              <option value="">Any rarity</option>
              <option value="Common">Common</option>
              <option value="Uncommon">Uncommon</option>
              <option value="Rare">Rare</option>
              <option value="Mythic">Mythic</option>
            </select>
          </div>

          <div className="collection-filter-field">
            <label htmlFor="collection-filter-artist">Artist</label>
            <input
              type="text"
              className="form-control form-control-sm"
              id="collection-filter-artist"
              placeholder="Artist"
              value={filters.artist}
              onChange={(e) => setFilter("cf_artist", e.target.value)}
            />
          </div>

          <div className="collection-filter-field">
            <label htmlFor="collection-filter-text">Card text</label>
            <input
              type="text"
              className="form-control form-control-sm"
              id="collection-filter-text"
              placeholder="Card text"
              value={filters.text}
              onChange={(e) => setFilter("cf_text", e.target.value)}
            />
          </div>

          <div className="collection-filter-row">
            <span className="collection-filter-label">Proxy</span>
            <div className="collection-filter-btn-group">
              {[
                { val: "all", label: "All" },
                { val: "proxy", label: "Proxy" },
                { val: "real", label: "Real" },
              ].map(({ val, label }) => (
                <button
                  key={val}
                  type="button"
                  className={`btn btn-sm ${filters.proxyMode === val ? "btn-primary" : "btn-outline-secondary"}`}
                  onClick={() => setFilter("cf_proxy", val)}
                >
                  {label}
                </button>
              ))}
            </div>
          </div>

          {systems.length > 1 && (
            <div className="collection-filter-field">
              <label htmlFor="collection-filter-provider">Game</label>
              <select
                className="form-select form-select-sm"
                id="collection-filter-provider"
                value={filters.provider}
                onChange={(e) => setFilter("cf_provider", e.target.value)}
              >
                <option value="">All games</option>
                {systems.map((s) => (
                  <option key={s} value={s}>{SYSTEM_LABELS[s] ?? s}</option>
                ))}
              </select>
            </div>
          )}

          {(systems.includes("MagicSQLite") || systems.includes("Scryfall")) && (
            <div className="collection-filter-row">
              <span className="collection-filter-label">Colors</span>
              <div className="collection-color-controls">
                {MTG_COLORS.map(({ value, mana, Symbol, bg, border, title = value }) => {
                  const active = filters.colorIdentities.includes(value);
                  return (
                    <button
                      key={value}
                      type="button"
                      title={title}
                      aria-label={title}
                      aria-pressed={active}
                      onClick={() => setArrayFilter("cf_color", value, !active)}
                      className="mana-toggle-btn"
                      style={{
                        background: active ? bg : "transparent",
                        borderColor: active ? border : "rgba(255,255,255,0.3)",
                        opacity: active ? 1 : 0.45,
                        transform: active ? "scale(1.08)" : "scale(1)",
                      }}
                    >
                      {Symbol ? (
                        <Symbol className="mana-toggle-symbol" aria-hidden="true" />
                      ) : (
                        <i className={`ms ms-${mana} ms-cost`} />
                      )}
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          {hasActive && (
            <button className="btn btn-outline-danger btn-sm" onClick={clearFilters} type="button">
              Clear filters
            </button>
          )}
        </div>
      )}
    </section>
  );
}
