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

function CardComponent({ systemType, id, details }) {
  const effectiveSystem = details?.provider || systemType;
  if (effectiveSystem === "RiftboundSQLite") {
    return <RiftboundCard id={id} details={details} provider={effectiveSystem} />;
  } else if (effectiveSystem === "PokemonSQLite") {
    return <PokemonCard id={id} details={details} provider={effectiveSystem} />;
  }
  return <Card id={id} details={details} provider={effectiveSystem} />;
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

  const cards = useCards();
  const cardsDispatch = useCardsDispatch();
  const [loading, setLoading] = useState(true);
  const [cardCount, setCardCount] = useState(0);

  useEffect(() => {
    ops
      .fetch(
        "Listing items in " + collection,
        [],
        "/collection/cards/" +
          collection +
          "/list?offset=" +
          (pageNumber - 1) * pageSize +
          "&pageSize=" +
          pageSize,
      )
      .then((data) => {
        cardsDispatch({ type: "overwrite", cards: data });
        setLoading(false);
        setRefresh(false);
        selectedDispatch({ type: "empty" });
      });
    ops
      .fetch(
        "Getting card count in " + collection,
        0,
        "/collection/cards/" + collection + "/count",
      )
      .then((data) => {
        setCardCount(data);
      });
  }, [collection, pageNumber, refresh]);

  const handlePageChange = (event) => {
    navigate("/c/" + collection + "/" + (parseInt(event.selected) + 1) + location.search);
  };

  return (
    <>
      <div className="card-grid list">
        {(loading || refresh) && cards.length === 0 ? (
          <p>Loading...</p>
        ) : (
          <React.Fragment>
            {cards
              .filter(
                (card) =>
                  systems.length === 0 ||
                  !card.provider ||
                  systems.includes(card.provider),
              )
              .map((card) => (
                <CardComponent
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
