import React, { useState, useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import SearchMagic from "./SearchMagic";
import SearchRiftbound from "./SearchRiftbound";
import { useOperations } from "../OperationsContext";
import { useCardSets } from "./ReusableConstants/CardSets";
import { useCollections } from "./CollectionContext";

function Search({ startSearch = false, dedicatedPage = false }) {
  const ops = useOperations();
  const [cards, setCards] = useState([]);
  const [loading, setLoading] = useState(false);
  const [pageNumber, setPageNumber] = useState(1);
  const [shouldSearch, setShouldSearch] = useState(startSearch);
  const cardSets = useCardSets();
  const collections = useCollections();
  const [systemType, setSystemType] = useState(null);

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

  // Fetch system type on component mount
  useEffect(() => {
    async function fetchSystemType() {
      try {
        const response = await ops.fetch(
          "Getting system info",
          [],
          "/system",
          {},
        );
        setSystemType(response.system);
      } catch (error) {
        console.error("Failed to get system type:", error);
        // Default to Magic if we can't determine the system
        setSystemType("Sql");
      }
    }
    fetchSystemType();
  }, []);

  useEffect(() => {
    if (shouldSearch && systemType) {
      setLoading(true);

      let url =
        searchCollection !== "" && searchCollection !== "skipNotOwned"
          ? "/collection/cards/" + searchCollection + "/search?pageSize="
          : "/collection/search?pageSize=";
      url = url + pageSize + "&offset=" + (pageNumber - 1) * pageSize;

      if (searchCollection === "skipNotOwned") {
        url = url + "&skipNotOwned=true";
      }

      // Use different endpoints based on system type
      const isRiftboundSystem =
        systemType === "RiftboundSql" ||
        searchParams.get("riftbound") === "true";

      if (isRiftboundSystem) {
        url =
          "/riftbound/cards/search?pageSize=" +
          pageSize +
          "&offset=" +
          (pageNumber - 1) * pageSize;
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
  }, [pageNumber, shouldSearch, systemType]);

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

  // Determine which search component to use based on system type
  const isRiftboundSystem =
    systemType === "RiftboundSql" || searchParams.get("riftbound") === "true";

  return (
    <React.Fragment>
      {isRiftboundSystem ? (
        <SearchRiftbound
          startSearch={startSearch}
          dedicatedPage={dedicatedPage}
        />
      ) : (
        <SearchMagic startSearch={startSearch} dedicatedPage={dedicatedPage} />
      )}
    </React.Fragment>
  );
}

export default Search;
