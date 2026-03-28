import React, { useState, useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import RiftboundCard from "./RiftboundCard";
import { useOperations } from "../OperationsContext";
import ReactPaginate from "react-paginate";

function SearchRiftbound({ startSearch = false, dedicatedPage = false, sidePanel = false }) {
  const ops = useOperations();
  const [cards, setCards] = useState([]);
  const [loading, setLoading] = useState(false);
  const [pageNumber, setPageNumber] = useState(1);
  const [shouldSearch, setShouldSearch] = useState(startSearch);

  const [searchParams, setSearchParams] = useSearchParams();
  const [searchOptions, setSearchOptions] = useState({
    name: searchParams.get("name") ?? "",
    setCode: searchParams.get("setCode") ?? "",
    artist: searchParams.get("artist") ?? "",
    collectorNumber: searchParams.get("collectorNumber") ?? "",
    text: searchParams.get("text") ?? "",
    rarity: searchParams.get("rarity") ?? "",
    colorIdentities: searchParams.getAll("colorIdentities"),
  });
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pageNumber, shouldSearch]);

  const handleSearchInput = (event, field) => {
    const newState = { ...searchOptions, [field]: event.target.value };
    setSearchOptions(newState);
    setSearchParams(newState);
  };

  const handleColourIdentitiesInput = (event) => {
    const filtered = searchOptions.colorIdentities.filter((c) => c !== event.target.value);
    const newState = {
      ...searchOptions,
      colorIdentities: event.target.checked ? [...filtered, event.target.value] : filtered,
    };
    setSearchOptions(newState);
    setSearchParams(newState);
  };

  const handlePageChange = (event) => {
    setShouldSearch(true);
    setPageNumber(parseInt(event.selected) + 1);
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
            {["Calm", "Chaos", "Fury", "Mind", "Body", "Order"].map((domain, i) => (
              <div key={domain} className="form-check form-check-inline">
                <input
                  onChange={handleColourIdentitiesInput}
                  className="form-check-input"
                  type="checkbox"
                  id={"inlineCheckbox" + (i + 1)}
                  value={domain}
                  checked={searchOptions.colorIdentities.includes(domain)}
                />
                <label className="form-check-label" htmlFor={"inlineCheckbox" + (i + 1)}>
                  {domain}
                </label>
              </div>
            ))}
          </div>
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
                    provider="RiftboundSQLite"
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
  );
}

export default SearchRiftbound;
