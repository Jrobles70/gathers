import React, { useState, useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import Card from "./Card";
import { useOperations, useMode } from "../OperationsContext";
import ReactPaginate from "react-paginate";
import { useCardSets } from "./ReusableConstants/CardSets";
import { useCollections } from "./CollectionContext";

function SearchMagic({ startSearch = false, dedicatedPage = false, sidePanel = false }) {
  const ops = useOperations();
  const [cards, setCards] = useState([]);
  const [loading, setLoading] = useState(false);

  const cardSets = useCardSets();
  const collections = useCollections();
  const { collectionsEnabled } = useMode();

  const [searchParams, setSearchParams] = useSearchParams();
  const [pageNumber, setPageNumber] = useState(parseInt(searchParams.get("page") ?? "1"));
  const initialOptions = {
    name: searchParams.get("name") ?? "",
    setCode: searchParams.get("setCode") ?? "",
    artist: searchParams.get("artist") ?? "",
    collectorNumber: searchParams.get("collectorNumber") ?? "",
    text: searchParams.get("text") ?? "",
    rarity: searchParams.get("rarity") ?? "",
    colorIdentities: searchParams.getAll("colorIdentities"),
  };
  const [searchOptions, setSearchOptions] = useState(initialOptions);
  const [searchCollection, setSearchCollection] = useState("");

  const hasParams = Object.values(initialOptions).some((v) =>
    Array.isArray(v) ? v.length > 0 : v !== ""
  );
  const [shouldSearch, setShouldSearch] = useState(startSearch || hasParams);

  let pageSize = 24;

  useEffect(() => {
    if (shouldSearch) {
      setLoading(true);

      let url;
      if (!collectionsEnabled) {
        url =
          "/mtg/cards/search?limit=" +
          pageSize +
          "&skip=" +
          (pageNumber - 1) * pageSize;
      } else if (searchCollection !== "" && searchCollection !== "skipNotOwned") {
        url =
          "/collection/cards/" +
          searchCollection +
          "/search?pageSize=" +
          pageSize +
          "&offset=" +
          (pageNumber - 1) * pageSize;
      } else {
        url =
          "/collection/search?pageSize=" +
          pageSize +
          "&offset=" +
          (pageNumber - 1) * pageSize;
        if (searchCollection === "skipNotOwned") {
          url = url + "&skipNotOwned=true";
        }
      }

      ops
        .fetch("Searching", [], url, {
          method: "post",
          headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
          },
          body: JSON.stringify(searchOptions),
        })
        .then((data) => {
          setCards(data);
          setLoading(false);
          setShouldSearch(false);
        });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pageNumber, shouldSearch]);

  const handleSearchInput = (event, field) => {
    const newState = { ...searchOptions, [field]: event.target.value };
    setSearchOptions(newState);
    setSearchParams({ ...newState, page: "1" });
  };

  const handleColourIdentitiesInput = (event) => {
    const filtered = searchOptions.colorIdentities.filter((c) => c !== event.target.value);
    const newState = {
      ...searchOptions,
      colorIdentities: event.target.checked ? [...filtered, event.target.value] : filtered,
    };
    setSearchOptions(newState);
    setSearchParams({ ...newState, page: "1" });
  };

  const handleCollectionInput = (event) => {
    setSearchCollection(event.target.value);
  };

  const handlePageChange = (event) => {
    const newPage = parseInt(event.selected) + 1;
    setShouldSearch(true);
    setPageNumber(newPage);
    setSearchParams({ ...searchOptions, page: String(newPage) });
  };

  return (
    <div
        className={dedicatedPage === true || sidePanel === true ? "" : "collapse"}
        id={dedicatedPage ? "main-search" : "search"}
      >
        <h2>Search</h2>
        <div className="list-group list-group-flush mx-3 mt-4">
          <div className="input-group">
            <input
              onChange={(event) => handleSearchInput(event, "name")}
              type="text"
              className="form-control"
              id="search-bar-name"
              placeholder="Name"
              value={searchOptions["name"]}
            />
            <input
              onChange={(event) => handleSearchInput(event, "setCode")}
              className="form-control"
              list="datalistOptions"
              id="search-bar-set"
              placeholder="Set Code"
              value={searchOptions["setCode"]}
            />
            <datalist id="datalistOptions">
              {cardSets.map((c) => (
                <option key={c} value={c} />
              ))}
            </datalist>
          </div>
          <div className="input-group">
            <input
              onChange={(event) => handleSearchInput(event, "artist")}
              type="text"
              className="form-control"
              id="search-bar-artist"
              placeholder="Artist"
              value={searchOptions["artist"]}
            />
            <input
              onChange={(event) => handleSearchInput(event, "collectorNumber")}
              type="text"
              className="form-control"
              id="search-bar-collector-number"
              placeholder="Collector Number"
              value={searchOptions["collectorNumber"]}
            />
          </div>
          <div className="input-group">
            <input
              onChange={(event) => handleSearchInput(event, "text")}
              type="text"
              className="form-control"
              id="search-bar-text"
              placeholder="Text"
              value={searchOptions["text"]}
            />
          </div>
          <div className="input-group">
            {/* Magic colors (W, U, B, R, G) */}
            <>
              <div className="form-check form-check-inline">
                <input
                  onChange={handleColourIdentitiesInput}
                  className="form-check-input"
                  type="checkbox"
                  id="inlineCheckbox1"
                  value="White"
                  checked={searchOptions.colorIdentities.includes("White")}
                />
                <label className="form-check-label" htmlFor="inlineCheckbox1">
                  W
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={handleColourIdentitiesInput}
                  className="form-check-input"
                  type="checkbox"
                  id="inlineCheckbox2"
                  value="Blue"
                  checked={searchOptions.colorIdentities.includes("Blue")}
                />
                <label className="form-check-label" htmlFor="inlineCheckbox2">
                  U
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={handleColourIdentitiesInput}
                  className="form-check-input"
                  type="checkbox"
                  id="inlineCheckbox3"
                  value="Black"
                  checked={searchOptions.colorIdentities.includes("Black")}
                />
                <label className="form-check-label" htmlFor="inlineCheckbox3">
                  B
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={handleColourIdentitiesInput}
                  className="form-check-input"
                  type="checkbox"
                  id="inlineCheckbox4"
                  value="Red"
                  checked={searchOptions.colorIdentities.includes("Red")}
                />
                <label className="form-check-label" htmlFor="inlineCheckbox4">
                  R
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={handleColourIdentitiesInput}
                  className="form-check-input"
                  type="checkbox"
                  id="inlineCheckbox5"
                  value="Green"
                  checked={searchOptions.colorIdentities.includes("Green")}
                />
                <label className="form-check-label" htmlFor="inlineCheckbox5">
                  G
                </label>
              </div>
            </>
          </div>
          <div className="input-group">
            <button
              onClick={(event) => {
                setPageNumber(1);
                setShouldSearch(true);
                setSearchParams({ ...searchOptions, page: "1" });
              }}
              className="btn btn-outline-secondary"
              type="button"
              id="button-addon2"
            >
              Search
            </button>
            {collectionsEnabled && (
              <select
                onChange={(e) => handleCollectionInput(e)}
                className="form-control"
                id="searchInCollection"
              >
                <option key={"searchincol-empty"} value={""}>
                  in MtG database
                </option>
                <option
                  key={"searchincol-collections"}
                  value={"skipNotOwned"}
                >
                  in all collections
                </option>
                {collections.map((c) => (
                  <option key={"searchincol-" + c.id} value={c.id}>
                    {"in " + c.id}
                  </option>
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
                  ? cards.map((card) => (
                      <Card
                        key={
                          card.mtGCard.id +
                          "-" +
                          (card.mtGCard.details != null
                            ? card.mtGCard.details.collectionId
                            : "")
                        }
                        id={card.mtGCard.id}
                        card={card.mtGCard}
                        details={card.mtGCard.details}
                        showCollectionSelect={dedicatedPage && card.mtGCard.details == null}
                      />
                    ))
                  : cards.map((card) => (
                      <Card key={card.id} id={card.id} card={card} details={null} />
                    ))}
              </div>
            )}
          </div>
          {cards.length > 0 ? (
            <ReactPaginate
              previousLabel="Previous"
              nextLabel="Next"
              pageClassName="page-item"
              pageLinkClassName="page-link"
              previousClassName="page-item"
              previousLinkClassName="page-link"
              nextClassName="page-item"
              nextLinkClassName="page-link"
              breakLabel="..."
              breakClassName="page-item"
              breakLinkClassName="page-link"
              containerClassName="pagination"
              activeClassName="active"
              pageCount={cards.length >= pageSize ? pageNumber + 1 : pageNumber}
              marginPagesDisplayed={2}
              pageRangeDisplayed={5}
              onPageChange={handlePageChange}
              forcePage={cards.length > 0 ? Math.max(0, pageNumber - 1) : -1}
            />
          ) : null}
        </div>
        <hr />
      </div>
  );
}

export default SearchMagic;
