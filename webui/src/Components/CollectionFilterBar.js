import React, { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useSystems } from "./SystemTypeContext";
import SortControls from "./SortControls";
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

const SORT_FIELDS = [
  { value: "Name",             label: "Name" },
  { value: "SetCode",          label: "Set" },
  { value: "Rarity",           label: "Rarity" },
  { value: "CollectorNumber",  label: "No." },
];

const SORT_FIELD_VALUES = new Set(SORT_FIELDS.map(({ value }) => value));

const SYSTEM_LABELS = {
  MagicSQLite: "MTG",
  RiftboundSQLite: "Riftbound",
  PokemonSQLite: "Pokémon",
  Scryfall: "Scryfall",
};

function readFilters(searchParams) {
  const sortBy = searchParams.get("cf_sortBy");

  return {
    name:            searchParams.get("cf_name") ?? "",
    setCode:         searchParams.get("cf_setCode") ?? "",
    rarity:          searchParams.get("cf_rarity") ?? "",
    artist:          searchParams.get("cf_artist") ?? "",
    text:            searchParams.get("cf_text") ?? "",
    provider:        searchParams.get("cf_provider") ?? "",
    sortBy:          SORT_FIELD_VALUES.has(sortBy) ? sortBy : "Name",
    sortOrder:       searchParams.get("cf_sortOrder") ?? "Asc",
    viewMode:        searchParams.get("cf_viewMode") ?? "grid",
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
    filters.colorIdentities.length > 0
  );
}

export default function CollectionFilterBar() {
  const systems = useSystems();
  const [searchParams, setSearchParams] = useSearchParams();
  const [open, setOpen] = useState(false);
  const filters = readFilters(searchParams);

  const setFilter = (key, value) => {
    const next = new URLSearchParams(searchParams);
    if (value === "" || value === null) {
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
     "cf_sortBy","cf_sortOrder","cf_color","cf_domain","cf_energy","page"].forEach((k) => next.delete(k));
    setSearchParams(next);
  };

  const hasActive = collectionFiltersActive(filters);

  return (
    <section className="collection-filter-bar collection-panel-section" data-bs-theme="dark">
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

          <SortControls
            sortBy={filters.sortBy}
            sortOrder={filters.sortOrder}
            fields={SORT_FIELDS}
            onChange={(field, order) => {
              const next = new URLSearchParams(searchParams);
              next.set("cf_sortBy", field);
              next.set("cf_sortOrder", order);
              next.set("page", "1");
              setSearchParams(next);
            }}
          />

          <div className="collection-filter-row">
            <span className="collection-filter-label">View</span>
            <div className="btn-group btn-group-sm" role="group" aria-label="View mode">
              <button
                type="button"
                className={`btn ${filters.viewMode === "grid" ? "btn-secondary" : "btn-outline-secondary"}`}
                onClick={() => setFilter("cf_viewMode", "grid")}
                title="Grid view"
              >
                Grid
              </button>
              <button
                type="button"
                className={`btn ${filters.viewMode === "list" ? "btn-secondary" : "btn-outline-secondary"}`}
                onClick={() => setFilter("cf_viewMode", "list")}
                title="List view"
              >
                List
              </button>
            </div>
          </div>

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
