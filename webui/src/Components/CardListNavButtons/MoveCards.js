import React, { useEffect, useMemo, useState } from "react";
import { useOperations } from "../../OperationsContext";
import { isAllCollections, useCollection, useCollections } from "../CollectionContext";
import {
  useSelectedCards,
  useSelectedCardsDispatch,
} from "../CardListContexts/SelectedCardsContext";
import { useRefreshCardList } from "../CardListContexts/RefreshCardListContext";

export default function MoveCards() {
  const ops = useOperations();
  const collection = useCollection();
  const collections = useCollections();
  const selected = useSelectedCards();
  const selectedDispatch = useSelectedCardsDispatch();
  const triggerRefresh = useRefreshCardList();
  const destinationOptions = useMemo(
    () => collections.filter((item) => !isAllCollections(item.id)),
    [collections],
  );

  const [destinationCollection, setDestinationCollection] =
    useState(isAllCollections(collection) ? "" : collection);
  useEffect(() => {
    if (isAllCollections(collection)) {
      setDestinationCollection((current) =>
        destinationOptions.some((item) => item.id === current)
          ? current
          : destinationOptions[0]?.id ?? "",
      );
      return;
    }
    setDestinationCollection(collection);
  }, [collection, destinationOptions]);
  const canMove = selected.length > 0 && Boolean(destinationCollection);

  const moveCards = () => {
    ops
      .fetch(
        "Moving items between " + collection + " and " + destinationCollection,
        [],
        "/collection/move/" + encodeURIComponent(destinationCollection),
        {
          method: "post",
          headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
          },
          body: JSON.stringify(selected),
        },
      )
      .then((data) => {
        triggerRefresh(true);
        selectedDispatch({ type: "empty" });
      });
  };

  return (
    <form className="move-cards-form">
      <button
        disabled={!canMove}
        onClick={moveCards}
        type="button"
        className="btn btn-outline-info btn-sm"
      >
        Move
      </button>
      <select
        value={destinationCollection}
        onChange={(e) => setDestinationCollection(e.target.value)}
        className="form-select form-select-sm"
        id="exampleFormControlSelect1"
      >
        {destinationOptions.map((c) => (
          <option key={"cardlistcol-" + c.id} value={c.id}>
            {c.id}
          </option>
        ))}
      </select>
    </form>
  );
}
