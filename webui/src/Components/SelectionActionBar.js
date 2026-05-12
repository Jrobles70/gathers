import React, { useState } from "react";
import { useSelectedCards, useSelectedCardsDispatch } from "./CardListContexts/SelectedCardsContext";
import { useCards } from "./CardListContexts/CardsContext";
import { useRefreshCardList } from "./CardListContexts/RefreshCardListContext";
import { useOperations } from "../OperationsContext";
import { useCollection } from "./CollectionContext";
import { confirm } from "./CardListNavButtons/ConfirmCollectionDelete";
import MoveToDialog from "./MoveToDialog";

export default function SelectionActionBar() {
  const selected = useSelectedCards();
  const selectedDispatch = useSelectedCardsDispatch();
  const cards = useCards();
  const triggerRefresh = useRefreshCardList();
  const ops = useOperations();
  const collection = useCollection();
  const [moveToOpen, setMoveToOpen] = useState(false);

  const selectAll = () => {
    cards.forEach((card) => {
      selectedDispatch({ type: "added", card });
    });
  };

  const deselectAll = () => {
    selectedDispatch({ type: "empty" });
  };

  const removeCards = () => {
    confirm({ confirmType: "cards", selectedCount: selected.length }).then(
      ({ input }) => {
        Promise.all(
          selected.map((card) =>
            ops.fetch(
              "Removing " + card.id + " from " + (card.collectionId ?? collection),
              [],
              "/collection/cards/" + encodeURIComponent(card.collectionId ?? collection) + "/delete",
              {
                method: "post",
                headers: { Accept: "application/json", "Content-Type": "application/json" },
                body: JSON.stringify({
                  id: card.id,
                  quantity: card.quantity,
                  foilQuantity: card.foilQuantity,
                }),
              }
            )
          )
        ).then(() => {
          triggerRefresh(true);
          selectedDispatch({ type: "empty" });
        });
      },
      () => {}
    );
  };

  return (
    <div className={"selection-action-bar" + (selected.length > 0 ? " visible" : "")}>
      <span className="selection-action-bar-count">{selected.length} selected</span>
      <button type="button" className="btn btn-sm btn-outline-secondary" onClick={selectAll}>
        Select All
      </button>
      <button type="button" className="btn btn-sm btn-outline-secondary" onClick={deselectAll}>
        Deselect All
      </button>
      <button type="button" className="btn btn-sm btn-outline-info" onClick={() => setMoveToOpen(true)} disabled={selected.length === 0}>
        Move To
      </button>
      <button type="button" className="btn btn-sm btn-outline-danger" onClick={removeCards} disabled={selected.length === 0}>
        Remove
      </button>
      {moveToOpen && <MoveToDialog onClose={() => setMoveToOpen(false)} />}
    </div>
  );
}
