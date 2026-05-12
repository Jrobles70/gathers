import React, { useState, useMemo, useEffect, useRef } from "react";
import useCardSearch from "./useCardSearch";
import { useCardSets } from "./ReusableConstants/CardSets";
import { useCollections } from "./CollectionContext";
import { useMode, useOperations } from "../OperationsContext";
import { MobileBottomNav } from "./MobileCollectionView";
import Card from "./Card";
import SearchPagination from "./SearchPagination";
import SortControls from "./SortControls";
import {
  groupMagicSearchResults,
  listMagicSearchResultsByPrinting,
} from "./searchPrintings";
import {
  bulkSearchTotals,
  flattenBulkMatches,
  parseBulkSearchInput,
} from "./bulkSearch";
import { ReactComponent as ColorlessSymbol } from "../assets/card-symbols/C.svg";

const colors = [
  { value: "White",     label: "W" },
  { value: "Blue",      label: "U" },
  { value: "Black",     label: "B" },
  { value: "Red",       label: "R" },
  { value: "Green",     label: "G" },
  { value: "Colorless", Symbol: ColorlessSymbol, title: "Colorless" },
];

export default function MobileSearchView() {
  const ops = useOperations();
  const cardSets = useCardSets();
  const collections = useCollections();
  const { collectionsEnabled } = useMode();

  const [searchMode, setSearchMode] = useState("single"); // "single" | "bulk"
  const [bulkText, setBulkText] = useState("");
  const [bulkResults, setBulkResults] = useState([]);
  const [searchCollection, setSearchCollection] = useState("");
  const [setCodeFocused, setSetCodeFocused] = useState(false);
  const [hasInteracted, setHasInteracted] = useState(false);

  const {
    cards, setCards,
    loading, setLoading,
    pageNumber,
    shouldSearch, setShouldSearch,
    searchOptions,
    handleSearchInput,
    handleArrayInput,
    handleMultiInput,
    handlePageChange,
    triggerSearch,
  } = useCardSearch({
    stringFields: ["name", "setCode", "artist", "text", "sortBy", "sortOrder"],
    arrayFields: ["colorIdentities"],
    startSearch: false,
    defaults: { sortBy: "Name", sortOrder: "Asc" },
  });

  const triggerSearchRef = useRef(triggerSearch);
  useEffect(() => { triggerSearchRef.current = triggerSearch; });

  const searchOptionsKey = JSON.stringify(searchOptions);
  useEffect(() => {
    if (!hasInteracted || searchMode !== "single") return;
    const t = setTimeout(() => triggerSearchRef.current(), 350);
    return () => clearTimeout(t);
  }, [searchOptionsKey, searchCollection, hasInteracted, searchMode]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleBulkSearch = () => {
    const cardsToFind = parseBulkSearchInput(bulkText);
    if (cardsToFind.length === 0) { setBulkResults([]); setCards([]); return; }
    setLoading(true);
    setShouldSearch(false);
    const params = new URLSearchParams();
    if (searchCollection !== "" && searchCollection !== "skipNotOwned")
      params.set("collection", searchCollection);
    const query = params.toString();
    ops.fetch("Bulk searching", [], `/collection/bulk-search${query ? `?${query}` : ""}`, {
      method: "post",
      headers: { Accept: "application/json", "Content-Type": "application/json" },
      body: JSON.stringify({ cards: cardsToFind }),
    }).then((data) => { setBulkResults(data); setCards(flattenBulkMatches(data)); setLoading(false); });
  };

  useEffect(() => {
    if (!shouldSearch) return;
    setLoading(true);
    let url;
    if (!collectionsEnabled) {
      url = `/mtg/cards/search?limit=24&skip=${(pageNumber - 1) * 24}`;
    } else {
      const params = new URLSearchParams();
      params.set("pageSize", "24");
      params.set("offset", String((pageNumber - 1) * 24));
      if (searchCollection === "skipNotOwned") params.set("skipNotOwned", "true");
      else if (searchCollection !== "") params.set("collection", searchCollection);
      url = `/collection/search?${params.toString()}`;
    }
    ops.fetch("Searching", [], url, {
      method: "post",
      headers: { Accept: "application/json", "Content-Type": "application/json" },
      body: JSON.stringify(searchOptions),
    }).then((data) => {
      setCards(data);
      setLoading(false);
      setShouldSearch(false);
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pageNumber, shouldSearch]);

  const bulkMode = collectionsEnabled && searchMode === "bulk";
  const parsedBulkCards = useMemo(() => parseBulkSearchInput(bulkText), [bulkText]);
  const shouldGroupPrintings = !collectionsEnabled || searchCollection === "";
  const cardGroups = shouldGroupPrintings
    ? groupMagicSearchResults(cards, collectionsEnabled)
    : listMagicSearchResultsByPrinting(cards, collectionsEnabled);
  const bulkCardGroups = listMagicSearchResultsByPrinting(flattenBulkMatches(bulkResults), true);
  const displayedCardGroups = bulkMode ? bulkCardGroups : cardGroups;
  const missingBulkCards = bulkResults.filter((r) => (r.neededQuantity ?? 0) > 0);
  const totals = bulkSearchTotals(bulkResults);

  return (
    <div className="mobile-collection-app">
      {/* Single / Bulk tabs — only show if collectionsEnabled */}
      {collectionsEnabled && (
        <div className="mobile-search-tabs" role="tablist">
          <button type="button" role="tab"
            className={"mobile-search-tab" + (!bulkMode ? " active" : "")}
            onClick={() => setSearchMode("single")} aria-selected={!bulkMode}>
            Single
          </button>
          <button type="button" role="tab"
            className={"mobile-search-tab" + (bulkMode ? " active" : "")}
            onClick={() => setSearchMode("bulk")} aria-selected={bulkMode}>
            Bulk
          </button>
        </div>
      )}

      {/* Scrollable filters body */}
      <main className="mobile-search-filters-body">
        {bulkMode ? (
          /* Bulk mode body */
          <div className="mobile-search-bulk-body">
            <textarea
              className="mobile-filter-input"
              value={bulkText}
              onChange={(e) => setBulkText(e.target.value)}
              placeholder={"5 Swamp\n1 Tainted Peak\n1 Talisman of Indulgence"}
              rows={6}
            />
            <div className="bulk-search-meta">
              <span>{parsedBulkCards.length} cards parsed</span>
              {bulkResults.length > 0 && <span>{totals.owned}/{totals.requested} owned</span>}
            </div>
            {missingBulkCards.length > 0 && (
              <div className="bulk-needed-list">
                {missingBulkCards.map((r) => (
                  <span className="bulk-needed-pill" key={r.name}>{r.neededQuantity} {r.name}</span>
                ))}
              </div>
            )}
          </div>
        ) : (
          /* Single mode filters */
          <>
            {/* Name */}
            <div className="mobile-filter-group">
              <span className="mobile-filter-group-label">Name</span>
              <input
                className="mobile-filter-input"
                type="search"
                placeholder="Search cards"
                value={searchOptions.name}
                onChange={(e) => { setHasInteracted(true); handleSearchInput(e, "name"); }}
                aria-label="Card name"
              />
            </div>

            {/* Colors */}
            <div className="mobile-filter-group">
              <span className="mobile-filter-group-label">Colors</span>
              <div className="mobile-color-circles">
                {colors.map(({ value, label, Symbol, title = value }) => (
                  <label key={value} className="mobile-color-circle" title={title}>
                    <input
                      type="checkbox"
                      value={value}
                      checked={searchOptions.colorIdentities.includes(value)}
                      onChange={(e) => { setHasInteracted(true); handleArrayInput("colorIdentities", e); }}
                      aria-label={title}
                    />
                    <span className={"mobile-color-circle-swatch mobile-color-" + value.toLowerCase()}>
                      {Symbol ? <Symbol aria-hidden="true" /> : label}
                    </span>
                  </label>
                ))}
              </div>
            </div>

            {/* Set */}
            <div className="mobile-filter-group">
              <span className="mobile-filter-group-label">Set</span>
              <input
                className="mobile-filter-input"
                type="text"
                list="mobile-datalist-sets"
                placeholder="Set code"
                value={searchOptions.setCode}
                onFocus={() => setSetCodeFocused(true)}
                onBlur={() => setSetCodeFocused(false)}
                onChange={(e) => {
                  setHasInteracted(true);
                  const raw = e.target.value;
                  const code = raw.includes(" — ") ? raw.split(" — ")[0] : raw;
                  handleSearchInput({ target: { value: code } }, "setCode");
                }}
              />
              <datalist id="mobile-datalist-sets">
                {setCodeFocused && cardSets
                  .filter(c => c.code && (!searchOptions.setCode ||
                    c.code.toLowerCase().startsWith(searchOptions.setCode.toLowerCase()) ||
                    c.name.toLowerCase().includes(searchOptions.setCode.toLowerCase())))
                  .slice(0, 20)
                  .map((c) => <option key={c.code} value={`${c.code} — ${c.name}`} />)}
              </datalist>
            </div>

            {/* Text */}
            <div className="mobile-filter-group">
              <span className="mobile-filter-group-label">Text</span>
              <input className="mobile-filter-input" type="text" placeholder="Card text"
                value={searchOptions.text}
                onChange={(e) => { setHasInteracted(true); handleSearchInput(e, "text"); }} />
            </div>

            {/* Artist */}
            <div className="mobile-filter-group">
              <span className="mobile-filter-group-label">Artist</span>
              <input className="mobile-filter-input" type="text" placeholder="Artist name"
                value={searchOptions.artist}
                onChange={(e) => { setHasInteracted(true); handleSearchInput(e, "artist"); }} />
            </div>

            {/* Sort */}
            <div className="mobile-filter-group">
              <span className="mobile-filter-group-label">Sort</span>
              <SortControls
                sortBy={searchOptions.sortBy}
                sortOrder={searchOptions.sortOrder}
                fields={[
                  { value: "Name", label: "Name" },
                  { value: "Rarity", label: "Rarity" },
                  { value: "SetCode", label: "Set Code" },
                  { value: "Artist", label: "Artist" },
                ]}
                onChange={(field, order) => { setHasInteracted(true); handleMultiInput({ sortBy: field, sortOrder: order }); }}
              />
            </div>

            {/* Search in collection — only when collectionsEnabled */}
            {collectionsEnabled && (
              <div className="mobile-filter-group">
                <span className="mobile-filter-group-label">Search in</span>
                <select className="mobile-filter-input" value={searchCollection}
                  onChange={(e) => { setHasInteracted(true); setSearchCollection(e.target.value); }}>
                  <option value="">MTG database</option>
                  <option value="skipNotOwned">All collections</option>
                  {collections.map((c) => (
                    <option key={c.id} value={c.id}>in {c.id}</option>
                  ))}
                </select>
              </div>
            )}
          </>
        )}

        {/* Results */}
        <div className="mobile-search-results mobile-card-grid">
          {loading ? (
            <p>Loading…</p>
          ) : (
            <div className="card-grid list">
              {collectionsEnabled
                ? displayedCardGroups.map(({ primary, printings }) => (
                    <Card key={primary.id + "-" + (primary.details?.collectionId ?? "")}
                      id={primary.id} card={primary.card} details={primary.details}
                      printings={printings} showCollectionSelect={primary.details == null} />
                  ))
                : displayedCardGroups.map(({ primary, printings }) => (
                    <Card key={primary.id} id={primary.id} card={primary.card}
                      details={null} printings={printings} />
                  ))}
            </div>
          )}
          {!bulkMode && (
            <SearchPagination cards={cards} pageSize={24} pageNumber={pageNumber} onPageChange={handlePageChange} />
          )}
        </div>
      </main>

      {/* Floating search FAB */}
      <button
        type="button"
        className="mobile-search-fab"
        aria-label="Search"
        onClick={() => { setHasInteracted(true); bulkMode ? handleBulkSearch() : triggerSearch(); }}
      >
        ⌕
      </button>

      <MobileBottomNav activeTab="search" />
    </div>
  );
}
