import React, { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useSystems } from "./SystemTypeContext";
import SortControls from "./SortControls";
import colorlessSymbol from "../assets/card-symbols/C.svg";
import "mana-font/css/mana.min.css";

const MTG_COLORS = [
  { value: "White",      mana: "w", bg: "#f9faf4", border: "#c4b78a" },
  { value: "Blue",       mana: "u", bg: "#0e68ab", border: "#0a4f82" },
  { value: "Black",      mana: "b", bg: "#2a2a2a", border: "#555" },
  { value: "Red",        mana: "r", bg: "#d3202a", border: "#a01820" },
  { value: "Green",      mana: "g", bg: "#00733e", border: "#005a30" },
  { value: "Colorless",  symbol: colorlessSymbol, bg: "#cac5c0", border: "#8f8780", title: "Colorless" },
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
    <div className="collection-filter-bar bg-body-tertiary border-bottom" data-bs-theme="dark">
      <div className="d-flex align-items-center gap-2 px-3 py-2">
        <button
          className={`btn btn-sm ${open ? "btn-secondary" : "btn-outline-secondary"}`}
          onClick={() => setOpen((o) => !o)}
          type="button"
        >
          Filters {hasActive && <span className="badge bg-primary ms-1">●</span>}
        </button>

        {hasActive && (
          <button className="btn btn-sm btn-outline-danger" onClick={clearFilters} type="button">
            Clear
          </button>
        )}

        <div className="ms-auto">
          <div className="btn-group btn-group-sm" role="group">
            <button
              type="button"
              className={`btn ${filters.viewMode === "grid" ? "btn-secondary" : "btn-outline-secondary"}`}
              onClick={() => setFilter("cf_viewMode", "grid")}
              title="Grid view"
            >
              ⊞
            </button>
            <button
              type="button"
              className={`btn ${filters.viewMode === "list" ? "btn-secondary" : "btn-outline-secondary"}`}
              onClick={() => setFilter("cf_viewMode", "list")}
              title="List view"
            >
              ☰
            </button>
          </div>
        </div>
      </div>

      {open && (
        <div className="px-3 pb-3">
          <div className="row g-2 mb-2">
            <div className="col-sm-4">
              <input
                type="text"
                className="form-control form-control-sm"
                placeholder="Name"
                value={filters.name}
                onChange={(e) => setFilter("cf_name", e.target.value)}
              />
            </div>
            <div className="col-sm-2">
              <input
                type="text"
                className="form-control form-control-sm"
                placeholder="Set Code"
                value={filters.setCode}
                onChange={(e) => setFilter("cf_setCode", e.target.value)}
              />
            </div>
            <div className="col-sm-2">
              <select
                className="form-select form-select-sm"
                value={filters.rarity}
                onChange={(e) => setFilter("cf_rarity", e.target.value)}
              >
                <option value="">Any Rarity</option>
                <option value="Common">Common</option>
                <option value="Uncommon">Uncommon</option>
                <option value="Rare">Rare</option>
                <option value="Mythic">Mythic</option>
              </select>
            </div>
            <div className="col-sm-2">
              <input
                type="text"
                className="form-control form-control-sm"
                placeholder="Artist"
                value={filters.artist}
                onChange={(e) => setFilter("cf_artist", e.target.value)}
              />
            </div>
            <div className="col-sm-2">
              <input
                type="text"
                className="form-control form-control-sm"
                placeholder="Card Text"
                value={filters.text}
                onChange={(e) => setFilter("cf_text", e.target.value)}
              />
            </div>
          </div>

          <div className="row g-2 mb-2">
            {systems.length > 1 && (
              <div className="col-sm-3">
                <select
                  className="form-select form-select-sm"
                  value={filters.provider}
                  onChange={(e) => setFilter("cf_provider", e.target.value)}
                >
                  <option value="">All Games</option>
                  {systems.map((s) => (
                    <option key={s} value={s}>{SYSTEM_LABELS[s] ?? s}</option>
                  ))}
                </select>
              </div>
            )}

            {(systems.includes("MagicSQLite") || systems.includes("Scryfall")) && (
              <div className="col-auto d-flex align-items-center gap-1">
                <small className="text-muted me-1">Colors:</small>
                {MTG_COLORS.map(({ value, mana, symbol, bg, border, title = value }) => {
                  const active = filters.colorIdentities.includes(value);
                  return (
                    <button
                      key={value}
                      type="button"
                      title={title}
                      aria-label={title}
                      onClick={() => setArrayFilter("cf_color", value, !active)}
                      className="mana-toggle-btn"
                      style={{
                        background: active ? bg : "transparent",
                        borderColor: active ? border : "rgba(255,255,255,0.3)",
                        opacity: active ? 1 : 0.45,
                        transform: active ? "scale(1.15)" : "scale(1)",
                      }}
                    >
                      {symbol ? (
                        <img className="mana-toggle-symbol" src={symbol} alt="" aria-hidden="true" />
                      ) : (
                        <i className={`ms ms-${mana} ms-cost`} />
                      )}
                    </button>
                  );
                })}
              </div>
            )}

          </div>

          <div className="row g-2">
            <div className="col-auto">
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
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
