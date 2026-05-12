import React, { useEffect } from "react";
import Card from "./Card";
import { useOperations, useMode } from "../OperationsContext";
import { useCardSets } from "./ReusableConstants/CardSets";
import { useCollections } from "./CollectionContext";
import useCardSearch from "./useCardSearch";
import SearchPagination from "./SearchPagination";
import SortControls from "./SortControls";
import { ReactComponent as ColorlessSymbol } from "../assets/card-symbols/C.svg";
import {
  groupMagicSearchResults,
  listMagicSearchResultsByPrinting,
} from "./searchPrintings";
import {
  bulkSearchTotals,
  flattenBulkMatches,
  parseBulkSearchInput,
} from "./bulkSearch";

const PAGE_SIZE = 24;

const SORT_FIELDS = [
  { value: "Name",            label: "Name" },
  { value: "Rarity",         label: "Rarity" },
  { value: "SetCode",        label: "Set Code" },
  { value: "CollectorNumber",label: "Collector Number" },
  { value: "Artist",         label: "Artist" },
];

function SearchMagic({
  startSearch = false,
  dedicatedPage = false,
  sidePanel = false,
  showTitle = true,
  targetCollection = null,
  detailReturnPath = null,
}) {
  const ops = useOperations();
  const cardSets = useCardSets();
  const collections = useCollections();
  const { collectionsEnabled } = useMode();

  const [searchCollection, setSearchCollection] = React.useState("");
  const [setCodeFocused, setSetCodeFocused] = React.useState(false);
  const [searchMode, setSearchMode] = React.useState("single");
  const [bulkText, setBulkText] = React.useState("");
  const [bulkResults, setBulkResults] = React.useState([]);

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
    stringFields: ["name", "setCode", "artist", "collectorNumber", "text", "rarity", "sortBy", "sortOrder"],
    arrayFields: ["colorIdentities"],
    startSearch,
    defaults: { sortBy: "Name", sortOrder: "Asc" },
  });

  const bulkMode = collectionsEnabled && searchMode === "bulk";
  const parsedBulkCards = React.useMemo(() => parseBulkSearchInput(bulkText), [bulkText]);

  const handleBulkSearch = () => {
    const cardsToFind = parseBulkSearchInput(bulkText);
    if (cardsToFind.length === 0) {
      setBulkResults([]);
      setCards([]);
      return;
    }

    setLoading(true);
    setShouldSearch(false);

    const params = new URLSearchParams();
    if (searchCollection !== "" && searchCollection !== "skipNotOwned") {
      params.set("collection", searchCollection);
    }
    const query = params.toString();

    ops
      .fetch("Bulk searching", [], `/collection/bulk-search${query ? `?${query}` : ""}`, {
        method: "post",
        headers: { Accept: "application/json", "Content-Type": "application/json" },
        body: JSON.stringify({ cards: cardsToFind }),
      })
      .then((data) => {
        setBulkResults(data);
        setCards(flattenBulkMatches(data));
        setLoading(false);
      });
  };

  useEffect(() => {
    if (!shouldSearch) return;
    setLoading(true);

    let url;
    if (!collectionsEnabled) {
      url = `/mtg/cards/search?limit=${PAGE_SIZE}&skip=${(pageNumber - 1) * PAGE_SIZE}`;
    } else {
      const params = new URLSearchParams();
      params.set("pageSize", String(PAGE_SIZE));
      params.set("offset", String((pageNumber - 1) * PAGE_SIZE));
      if (searchCollection === "skipNotOwned") {
        params.set("skipNotOwned", "true");
      } else if (searchCollection !== "") {
        params.set("collection", searchCollection);
      }
      url = `/collection/search?${params.toString()}`;
    }

    ops
      .fetch("Searching", [], url, {
        method: "post",
        headers: { Accept: "application/json", "Content-Type": "application/json" },
        body: JSON.stringify(searchOptions),
      })
      .then((data) => {
        setCards(data);
        setLoading(false);
        setShouldSearch(false);
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pageNumber, shouldSearch]);

  const colors = [
    { value: "White",      label: "W" },
    { value: "Blue",       label: "U" },
    { value: "Black",      label: "B" },
    { value: "Red",        label: "R" },
    { value: "Green",      label: "G" },
    { value: "Colorless",  Symbol: ColorlessSymbol, title: "Colorless" },
  ];
  const shouldGroupPrintings = !collectionsEnabled || searchCollection === "";
  const cardGroups = shouldGroupPrintings
    ? groupMagicSearchResults(cards, collectionsEnabled)
    : listMagicSearchResultsByPrinting(cards, collectionsEnabled);
  const bulkCardGroups = listMagicSearchResultsByPrinting(flattenBulkMatches(bulkResults), true);
  const displayedCardGroups = bulkMode ? bulkCardGroups : cardGroups;
  const missingBulkCards = bulkResults.filter((result) => (result.neededQuantity ?? 0) > 0);
  const totals = bulkSearchTotals(bulkResults);

  return (
    <div
      className={dedicatedPage === true || sidePanel === true ? "" : "collapse"}
      id={dedicatedPage ? "main-search" : "search"}
    >
      {showTitle && <h2>Search</h2>}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          bulkMode ? handleBulkSearch() : triggerSearch();
        }}
        className="list-group list-group-flush mx-3 mt-4"
      >
        {collectionsEnabled && (
          <div className="search-mode-toggle" role="group" aria-label="Search mode">
            <button
              type="button"
              className={"btn " + (!bulkMode ? "btn-secondary" : "btn-outline-secondary")}
              onClick={() => setSearchMode("single")}
            >
              Single
            </button>
            <button
              type="button"
              className={"btn " + (bulkMode ? "btn-secondary" : "btn-outline-secondary")}
              onClick={() => setSearchMode("bulk")}
            >
              Bulk
            </button>
          </div>
        )}
        {bulkMode ? (
          <div className="bulk-search-panel">
            <textarea
              className="form-control bulk-search-input"
              value={bulkText}
              onChange={(event) => setBulkText(event.target.value)}
              placeholder={"5 Swamp\n1 Tainted Peak\n1 Talisman of Indulgence\n1 The Master, Multiplied\n1 Twinflame"}
              rows={7}
            />
            <div className="bulk-search-meta">
              <span>{parsedBulkCards.length} cards parsed</span>
              {bulkResults.length > 0 && (
                <span>{totals.owned}/{totals.requested} owned</span>
              )}
            </div>
          </div>
        ) : (
          <>
            <div className="input-group">
              <input onChange={(e) => handleSearchInput(e, "name")} type="text" className="form-control" placeholder="Name" value={searchOptions.name} />
              <input
                onChange={(e) => {
                  const raw = e.target.value;
                  const code = raw.includes(" — ") ? raw.split(" — ")[0] : raw;
                  handleSearchInput({ target: { value: code } }, "setCode");
                }}
                onFocus={() => setSetCodeFocused(true)}
                onBlur={() => setSetCodeFocused(false)}
                className="form-control"
                list="datalistOptions"
                placeholder="Set Code"
                value={searchOptions.setCode}
              />
              <datalist id="datalistOptions">
                {setCodeFocused && cardSets
                  .filter(c => c.code && (!searchOptions.setCode ||
                    c.code.toLowerCase().startsWith(searchOptions.setCode.toLowerCase()) ||
                    c.name.toLowerCase().includes(searchOptions.setCode.toLowerCase())))
                  .slice(0, 20)
                  .map((c) => <option key={c.code} value={`${c.code} — ${c.name}`} />)}
              </datalist>
            </div>
            <div className="input-group">
              <input onChange={(e) => handleSearchInput(e, "artist")} type="text" className="form-control" placeholder="Artist" value={searchOptions.artist} />
              <input onChange={(e) => handleSearchInput(e, "collectorNumber")} type="text" className="form-control" placeholder="Collector Number" value={searchOptions.collectorNumber} />
            </div>
            <div className="input-group">
              <input onChange={(e) => handleSearchInput(e, "text")} type="text" className="form-control" placeholder="Text" value={searchOptions.text} />
            </div>
            <div className="input-group">
              {colors.map(({ value, label, Symbol, title = value }, i) => (
                <div key={value} className="form-check form-check-inline">
                  <input
                    onChange={(e) => handleArrayInput("colorIdentities", e)}
                    className="form-check-input"
                    type="checkbox"
                    id={`inlineCheckbox${i + 1}`}
                    value={value}
                    checked={searchOptions.colorIdentities.includes(value)}
                  />
                  <label className="form-check-label" htmlFor={`inlineCheckbox${i + 1}`} title={title}>
                    {Symbol ? (
                      <Symbol className="mana-checkbox-symbol" aria-label={title} />
                    ) : (
                      label
                    )}
                  </label>
                </div>
              ))}
            </div>
            <div className="input-group">
              <SortControls
                sortBy={searchOptions.sortBy}
                sortOrder={searchOptions.sortOrder}
                fields={SORT_FIELDS}
                onChange={(field, order) => handleMultiInput({ sortBy: field, sortOrder: order }, { search: true })}
              />
            </div>
          </>
        )}
        <div className="input-group">
          <button className="btn btn-outline-secondary" type="submit" id="button-addon2">
            {bulkMode ? "Bulk Search" : "Search"}
          </button>
          {collectionsEnabled && (
            <select value={searchCollection} onChange={(e) => setSearchCollection(e.target.value)} className="form-control" id="searchInCollection">
              <option value="">{bulkMode ? "in all collections" : "in MtG database"}</option>
              {!bulkMode && <option value="skipNotOwned">in all collections</option>}
              {collections.map((c) => (
                <option key={"searchincol-" + c.id} value={c.id}>{"in " + c.id}</option>
              ))}
            </select>
          )}
        </div>
        {bulkMode && bulkResults.length > 0 && (
          <div className="bulk-search-summary">
            <div className="bulk-search-summary-header">
              <strong>Still need {totals.needed}</strong>
              <span>{missingBulkCards.length} card names</span>
            </div>
            {missingBulkCards.length > 0 ? (
              <div className="bulk-needed-list">
                {missingBulkCards.map((result) => (
                  <span className="bulk-needed-pill" key={result.name}>
                    {result.neededQuantity} {result.name}
                  </span>
                ))}
              </div>
            ) : (
              <div className="bulk-search-complete">All requested cards are covered.</div>
            )}
          </div>
        )}
        <div className="search-results" id="search-results">
          {loading ? (
            <p>Loading...</p>
          ) : (
            <div className="card-grid list">
              {collectionsEnabled
                ? displayedCardGroups.map(({ primary, printings }) => (
                    <Card
                      key={primary.id + "-" + (primary.details != null ? primary.details.collectionId : "")}
                      id={primary.id}
                      card={primary.card}
                      details={primary.details}
                      printings={printings}
                      showCollectionSelect={dedicatedPage && targetCollection == null && primary.details == null}
                      targetCollection={targetCollection}
                      detailReturnPath={detailReturnPath}
                    />
                  ))
                : displayedCardGroups.map(({ primary, printings }) => (
                    <Card
                      key={primary.id}
                      id={primary.id}
                      card={primary.card}
                      details={null}
                      printings={printings}
                      targetCollection={targetCollection}
                      detailReturnPath={detailReturnPath}
                    />
                  ))}
            </div>
          )}
        </div>
        {!bulkMode && (
          <SearchPagination cards={cards} pageSize={PAGE_SIZE} pageNumber={pageNumber} onPageChange={handlePageChange} />
        )}
      </form>
      <hr />
    </div>
  );
}

export default SearchMagic;
