import React, { useState, useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import PokemonCard from "./PokemonCard";
import { useOperations } from "../OperationsContext";
import ReactPaginate from "react-paginate";
import { useCollections } from "./CollectionContext";

function SearchPokemon({ startSearch = false, dedicatedPage = false, sidePanel = false }) {
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
    collectorNumber:
      searchParams.get("collectorNumber") != null
        ? searchParams.get("collectorNumber")
        : "",
    energyTypes:
      searchParams.getAll("energyTypes") != null
        ? searchParams.getAll("energyTypes")
        : [],
  });

  let pageSize = 24;

  useEffect(() => {
    if (shouldSearch) {
      setLoading(true);

      let url =
        "/pokemon/cards/search?limit=" +
        pageSize +
        "&skip=" +
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

  const handleEnergyTypeInput = (event) => {
    let newState = Object.assign({}, searchOptions);
    if (event.target.checked) {
      newState["energyTypes"] = [
        ...newState["energyTypes"],
        event.target.value,
      ];
    } else {
      newState["energyTypes"] = newState["energyTypes"].filter(
        (e) => e !== event.target.value,
      );
    }
    setSearchOptions(newState);
    setSearchParams(newState);
  };

  const handlePageChange = (event) => {
    setShouldSearch(true);
    setPageNumber(parseInt(event.selected) + 1);
  };

  const energyTypes = [
    "Fire",
    "Water",
    "Grass",
    "Lightning",
    "Psychic",
    "Fighting",
    "Darkness",
    "Metal",
    "Dragon",
    "Fairy",
    "Colorless",
  ];

  return (
    <React.Fragment>
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
              id="search-bar-set"
              placeholder="Set Code"
              value={searchOptions["setCode"]}
            />
          </div>
          <div className="input-group">
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
            {energyTypes.map((type) => (
              <div key={type} className="form-check form-check-inline">
                <input
                  onChange={handleEnergyTypeInput}
                  className="form-check-input"
                  type="checkbox"
                  id={"energy-" + type}
                  value={type}
                />
                <label
                  className="form-check-label"
                  htmlFor={"energy-" + type}
                >
                  {type}
                </label>
              </div>
            ))}
          </div>
          <div className="input-group">
            <button
              onClick={() => {
                setPageNumber(1);
                setShouldSearch(true);
              }}
              className="btn btn-outline-secondary"
              type="button"
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
                  <PokemonCard
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

export default SearchPokemon;
