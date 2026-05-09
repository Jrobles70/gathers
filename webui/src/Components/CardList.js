import React, { useState, useEffect } from "react";
import Card from "./Card";
import RiftboundCard from "./RiftboundCard";
import PokemonCard from "./PokemonCard";
import { useNavigate, useLocation } from "react-router-dom";
import { useCollection, usePageNumber } from "./CollectionContext";
import { useOperations } from "../OperationsContext";
import { useSelectedCardsDispatch } from "./CardListContexts/SelectedCardsContext";
import { useSystemType, useSystems } from "./SystemTypeContext";
import ReactPaginate from "react-paginate";
import {
  useCards,
  useCardsDispatch,
  pageSize,
} from "../Components/CardListContexts/CardsContext";
import {
  useRefresh,
  useRefreshCardList,
} from "./CardListContexts/RefreshCardListContext";
import { useCollectionFilters, collectionFiltersActive } from "./CollectionFilterBar";

function CardComponent({ viewMode, systemType, id, details }) {
  const effectiveSystem = details?.provider || systemType;
  if (effectiveSystem === "RiftboundSQLite") {
    return <RiftboundCard id={id} details={details} provider={effectiveSystem} listMode={viewMode === "list"} />;
  } else if (effectiveSystem === "PokemonSQLite") {
    return <PokemonCard id={id} details={details} provider={effectiveSystem} listMode={viewMode === "list"} />;
  }
  return <Card id={id} details={details} provider={effectiveSystem} listMode={viewMode === "list"} />;
}

function buildListUrl(collection, filters, pageNumber, systems) {
  const params = new URLSearchParams();
  params.set("offset", String((pageNumber - 1) * pageSize));
  params.set("limit", String(pageSize));
  if (filters.sortBy && filters.sortBy !== "Name") params.set("sort_by", filters.sortBy);
  if (filters.sortOrder && filters.sortOrder !== "Asc") params.set("sort_order", filters.sortOrder);
  if (filters.provider) {
    params.set("provider", filters.provider);
  } else if (systems.length > 0) {
    params.set("providers", systems.join(","));
  }
  return `/collection/cards/${collection}/list?${params.toString()}`;
}

function buildSearchBody(filters) {
  const body = {};
  if (filters.name)    body.name = filters.name;
  if (filters.setCode) body.setCode = filters.setCode;
  if (filters.rarity)  body.rarity = filters.rarity;
  if (filters.artist)  body.artist = filters.artist;
  if (filters.text)    body.text = filters.text;
  if (filters.colorIdentities.length > 0) body.colorIdentities = filters.colorIdentities;
  if (filters.domains.length > 0)         body.domains = filters.domains;
  if (filters.energyTypes.length > 0)     body.energyTypes = filters.energyTypes;
  if (filters.sortBy)    body.sortBy = filters.sortBy;
  if (filters.sortOrder) body.sortOrder = filters.sortOrder;
  return body;
}

function impliedProviders(filters, systems) {
  if (filters.colorIdentities.length > 0) {
    return systems.filter((system) => system === "MagicSQLite" || system === "Scryfall");
  }
  if (filters.domains.length > 0) return ["RiftboundSQLite"];
  if (filters.energyTypes.length > 0) return ["PokemonSQLite"];
  return [];
}

function buildSearchUrl(collection, filters, pageNumber, systems, isCount = false) {
  const params = new URLSearchParams();
  params.set("offset", String((pageNumber - 1) * pageSize));
  params.set("limit", String(pageSize));
  if (filters.provider) {
    params.set("provider", filters.provider);
  } else {
    const providers = impliedProviders(filters, systems);
    if (providers.length > 0) {
      params.set("providers", providers.join(","));
    } else if (systems.length > 0) {
      params.set("providers", systems.join(","));
    }
  }
  const base = `/collection/cards/${collection}/search`;
  return isCount ? `${base}/count?${params.toString()}` : `${base}?${params.toString()}`;
}

export default function CardList() {
  const navigate = useNavigate();
  const location = useLocation();
  const ops = useOperations();
  const collection = useCollection();
  const pageNumber = usePageNumber();
  const selectedDispatch = useSelectedCardsDispatch();
  const systemType = useSystemType();
  const systems = useSystems();
  const refresh = useRefresh();
  const setRefresh = useRefreshCardList();
  const filters = useCollectionFilters();
  const filtersActive = collectionFiltersActive(filters);

  const cards = useCards();
  const cardsDispatch = useCardsDispatch();
  const [loading, setLoading] = useState(true);
  const [cardCount, setCardCount] = useState(0);

  // eslint-disable-next-line react-hooks/exhaustive-deps
  const filterDeps = [
    filtersActive, filters.name, filters.setCode, filters.rarity, filters.artist,
    filters.text, filters.provider, filters.sortBy, filters.sortOrder,
    JSON.stringify(filters.colorIdentities),
    JSON.stringify(filters.domains),
    JSON.stringify(filters.energyTypes),
  ];

  useEffect(() => {
    setLoading(true);

    if (filtersActive) {
      const body = buildSearchBody(filters);
      const searchUrl = buildSearchUrl(collection, filters, pageNumber, systems);
      const countUrl = buildSearchUrl(collection, filters, pageNumber, systems, true);

      ops
        .fetch("Filtering collection", [], searchUrl, {
          method: "post",
          headers: { Accept: "application/json", "Content-Type": "application/json" },
          body: JSON.stringify(body),
        })
        .then((data) => {
          cardsDispatch({ type: "overwrite", cards: data });
          setLoading(false);
          setRefresh(false);
          selectedDispatch({ type: "empty" });
        });

      ops
        .fetch("Getting filtered count", 0, countUrl, {
          method: "post",
          headers: { Accept: "application/json", "Content-Type": "application/json" },
          body: JSON.stringify(body),
        })
        .then((data) => setCardCount(data));
    } else {
      const listUrl = buildListUrl(collection, filters, pageNumber, systems);

      ops
        .fetch("Listing items in " + collection, [], listUrl)
        .then((data) => {
          cardsDispatch({ type: "overwrite", cards: data });
          setLoading(false);
          setRefresh(false);
          selectedDispatch({ type: "empty" });
        });

      const countParams = new URLSearchParams();
      if (filters.provider) {
        countParams.set("provider", filters.provider);
      } else if (systems.length > 0) {
        countParams.set("providers", systems.join(","));
      }
      ops
        .fetch("Getting card count in " + collection, 0, `/collection/cards/${collection}/count?${countParams.toString()}`)
        .then((data) => setCardCount(data));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [collection, pageNumber, refresh, ...filterDeps]);

  const handlePageChange = (event) => {
    navigate("/c/" + collection + "/" + (parseInt(event.selected) + 1) + location.search);
  };

  const viewMode = filters.viewMode;

  return (
    <>
      <div className={viewMode === "list" ? "card-list" : "card-grid list"}>
        {(loading || refresh) && cards.length === 0 ? (
          <p>Loading...</p>
        ) : (
          <React.Fragment>
            {cards.map((card) => (
              <CardComponent
                viewMode={viewMode}
                systemType={systemType}
                id={card.id}
                details={card}
                key={card.collectionId + "-" + card.id}
              />
            ))}
          </React.Fragment>
        )}
      </div>
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
        pageCount={Math.ceil(parseInt(cardCount) / pageSize)}
        marginPagesDisplayed={2}
        pageRangeDisplayed={5}
        onPageChange={handlePageChange}
        forcePage={cardCount > 0 ? Math.max(0, pageNumber - 1) : -1}
      />
    </>
  );
}
