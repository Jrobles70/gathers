import React, { useState, useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import RiftboundCard from "./RiftboundCard";
import { useOperations } from "../OperationsContext";
import ReactPaginate from "react-paginate";
import { useCollections } from "./CollectionContext";

function SearchRiftbound({ startSearch = false, dedicatedPage = false }) {
  const ops = useOperations();
  const [cards, setCards] = useState([]);
  const [loading, setLoading] = useState(false);
  const [pageNumber, setPageNumber] = useState(1);
  const [shouldSearch, setShouldSearch] = useState(startSearch);

  const collections = useCollections();

  const [searchParams, setSearchParams] = useSearchParams();
  const [searchOptions, setSearchOptions] = useState({
    name: searchParams.get("name") != null ? searchParams.get("name") : "",
    setCode:
      searchParams.get("setCode") != null ? searchParams.get("setCode") : "",
    artist:
      searchParams.get("artist") != null ? searchParams.get("artist") : "",
    collectorNumber:
      searchParams.get("collectorNumber") != null
        ? searchParams.get("collectorNumber")
        : "",
    text: searchParams.get("text") != null ? searchParams.get("text") : "",
    rarity:
      searchParams.get("rarity") != null ? searchParams.get("rarity") : "",
    colorIdentities:
      searchParams.getAll("colorIdentities") != null
        ? searchParams.getAll("colorIdentities")
        : [],
  });
  const [searchCollection, setSearchCollection] = useState("");

  let pageSize = 24;

  useEffect(() => {
    if (shouldSearch) {
      setLoading(true);

      // Use Riftbound-specific endpoint
      let url =
        "/riftbound/cards/search?pageSize=" +
        pageSize +
        "&offset=" +
        (pageNumber - 1) * pageSize;

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
  }, [pageNumber, shouldSearch]);

  const handleSearchInput = (event, field) => {
    let newState = Object.assign({}, searchOptions);
    newState[field] = event.target.value;
    setSearchOptions(newState);
    setSearchParams(newState);
  };

  const handleColourIdentitiesInput = (event, colour) => {
    let newState = Object.assign({}, searchOptions);
    if (event.target.checked) {
      newState["colorIdentities"] = [
        ...newState["colorIdentities"],
        event.target.value,
      ];
    } else {
      newState["colorIdentities"] = newState["colorIdentities"].filter(
        (c) => c != event.target.value,
      );
    }
    setSearchOptions(newState);
    setSearchParams(newState);
  };

  const handleCollectionInput = (event) => {
    setSearchCollection(event.target.value);
  };

  const handlePageChange = (event) => {
    setShouldSearch(true);
    setPageNumber(parseInt(event.selected) + 1);
  };

  return (
    <React.Fragment>
      <div
        className={dedicatedPage === true ? "" : "collapse"}
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
            {/* Riftbound domains (Calm, Chaos, Fury, Mind, Body, Order) */}
            <>
              <div className="form-check form-check-inline">
                <input
                  onChange={(e) => handleColourIdentitiesInput(e, "Calm")}
                  className="form-check-input"
                  type="checkbox"
                  id="inlineCheckbox1"
                  value="Calm"
                />
                <label className="form-check-label" htmlFor="inlineCheckbox1">
                  Calm
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={(e) => handleColourIdentitiesInput(e, "Chaos")}
                  className="form-check-input"
                  type="checkbox"
                  id="inlineCheckbox2"
                  value="Chaos"
                />
                <label className="form-check-label" htmlFor="inlineCheckbox2">
                  Chaos
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={(e) => handleColourIdentitiesInput(e, "Fury")}
                  className="form-check-input"
                  type="checkbox"
                  id="inlineCheckbox3"
                  value="Fury"
                />
                <label className="form-check-label" htmlFor="inlineCheckbox3">
                  Fury
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={(e) => handleColourIdentitiesInput(e, "Mind")}
                  className="form-check-input"
                  type="checkbox"
                  id="inlineCheckbox4"
                  value="Mind"
                />
                <label className="form-check-label" htmlFor="inlineCheckbox4">
                  Mind
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={(e) => handleColourIdentitiesInput(e, "Body")}
                  className="form-check-input"
                  type="checkbox"
                  id="inlineCheckbox5"
                  value="Body"
                />
                <label className="form-check-label" htmlFor="inlineCheckbox5">
                  Body
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={(e) => handleColourIdentitiesInput(e, "Order")}
                  className="form-check-input"
                  type="checkbox"
                  id="inlineCheckbox6"
                  value="Order"
                />
                <label className="form-check-label" htmlFor="inlineCheckbox6">
                  Order
                </label>
              </div>
            </>
          </div>
          {false ? (
            <div className="input-group">
              <div className="form-check form-check-inline">
                <input
                  onChange={(e) => handleSearchInput(e, "rarity")}
                  className="form-check-input"
                  type="radio"
                  id="rarityRadio1"
                  value="C"
                />
                <label className="form-check-label" htmlFor="rarityRadio1">
                  Common
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={(e) => handleSearchInput(e, "rarity")}
                  className="form-check-input"
                  type="radio"
                  id="rarityRadio2"
                  value="U"
                />
                <label className="form-check-label" htmlFor="rarityRadio2">
                  Uncommon
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={(e) => handleSearchInput(e, "rarity")}
                  className="form-check-input"
                  type="radio"
                  id="rarityRadio3"
                  value="R"
                />
                <label className="form-check-label" htmlFor="rarityRadio3">
                  Rare
                </label>
              </div>
              <div className="form-check form-check-inline">
                <input
                  onChange={(e) => handleSearchInput(e, "rarity")}
                  className="form-check-input"
                  type="radio"
                  id="rarityRadio4"
                  value="M"
                />
                <label className="form-check-label" htmlFor="rarityRadio4">
                  Mythic
                </label>
              </div>
            </div>
          ) : null}
          <div className="input-group">
            <button
              onClick={(event) => {
                setPageNumber(1);
                setShouldSearch(true);
              }}
              className="btn btn-outline-secondary"
              type="button"
              id="button-addon2"
            >
              Search
            </button>
            <select
              onChange={(e) => handleCollectionInput(e)}
              className="form-control"
              id="searchInCollection"
            >
              <option key={"searchincol-empty"} dropdown="in Riftbound database" value={""}>
                in Riftbound database
              </option>
              <option
                key={"searchincol-collections"}
                dropdown="in all collections"
                value={"skipNotOwned"}
              >
                in all collections
              </option>
              {collections.map((c) => (
                <option key={"searchincol-" + c.id} dropdown={c.id} value={c.id}>
                  {"in " + c.id}
                </option>
              ))}
            </select>
          </div>
          <div className="search-results" id="search-results">
            {loading ? (
              <p>Loading...</p>
            ) : (
              <div className="card-grid list">
                {cards.map((card) => (
                  <RiftboundCard
                    key={
                      card.id +
                      "-" +
                      (card.details != null ? card.details.collectionId : "")
                    }
                    id={card.id}
                    card={card}
                    details={card.details}
                  />
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
    </React.Fragment>
  );
}

export default SearchRiftbound;
