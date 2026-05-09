import React, { useEffect } from "react";
import Card from "./Card";
import { useOperations, useMode } from "../OperationsContext";
import { useCardSets } from "./ReusableConstants/CardSets";
import { useCollections } from "./CollectionContext";
import useCardSearch from "./useCardSearch";
import SearchPagination from "./SearchPagination";
import SortControls from "./SortControls";
import {
  groupMagicSearchResults,
  listMagicSearchResultsByPrinting,
} from "./searchPrintings";

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
    { value: "White", label: "W" },
    { value: "Blue",  label: "U" },
    { value: "Black", label: "B" },
    { value: "Red",   label: "R" },
    { value: "Green", label: "G" },
  ];
  const shouldGroupPrintings = !collectionsEnabled || searchCollection === "";
  const cardGroups = shouldGroupPrintings
    ? groupMagicSearchResults(cards, collectionsEnabled)
    : listMagicSearchResultsByPrinting(cards, collectionsEnabled);

  return (
    <div
      className={dedicatedPage === true || sidePanel === true ? "" : "collapse"}
      id={dedicatedPage ? "main-search" : "search"}
    >
      {showTitle && <h2>Search</h2>}
      <form onSubmit={(e) => { e.preventDefault(); triggerSearch(); }} className="list-group list-group-flush mx-3 mt-4">
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
          {colors.map(({ value, label }, i) => (
            <div key={value} className="form-check form-check-inline">
              <input
                onChange={(e) => handleArrayInput("colorIdentities", e)}
                className="form-check-input"
                type="checkbox"
                id={`inlineCheckbox${i + 1}`}
                value={value}
                checked={searchOptions.colorIdentities.includes(value)}
              />
              <label className="form-check-label" htmlFor={`inlineCheckbox${i + 1}`}>{label}</label>
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
        <div className="input-group">
          <button className="btn btn-outline-secondary" type="submit" id="button-addon2">
            Search
          </button>
          {collectionsEnabled && (
            <select onChange={(e) => setSearchCollection(e.target.value)} className="form-control" id="searchInCollection">
              <option value="">in MtG database</option>
              <option value="skipNotOwned">in all collections</option>
              {collections.map((c) => (
                <option key={"searchincol-" + c.id} value={c.id}>{"in " + c.id}</option>
              ))}
            </select>
          )}
        </div>
        <div className="search-results" id="search-results">
          {loading ? (
            <p>Loading...</p>
          ) : (
            <div className="card-grid list">
              {collectionsEnabled
                ? cardGroups.map(({ primary, printings }) => (
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
                : cardGroups.map(({ primary, printings }) => (
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
        <SearchPagination cards={cards} pageSize={PAGE_SIZE} pageNumber={pageNumber} onPageChange={handlePageChange} />
      </form>
      <hr />
    </div>
  );
}

export default SearchMagic;
