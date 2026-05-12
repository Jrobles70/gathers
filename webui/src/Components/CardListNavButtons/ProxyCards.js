import React from "react";
import { useOperations } from "../../OperationsContext";
import {
  useSelectedCards,
  useSelectedCardsDispatch,
} from "../CardListContexts/SelectedCardsContext";
import { useRefreshCardList } from "../CardListContexts/RefreshCardListContext";

export default function ProxyCards() {
  const ops = useOperations();
  const selected = useSelectedCards();
  const selectedDispatch = useSelectedCardsDispatch();
  const triggerRefresh = useRefreshCardList();

  const setProxy = (isProxy) => {
    if (selected.length === 0) return;
    ops
      .fetch(isProxy ? "Marking cards as proxy" : "Marking cards as regular", [], "/collection/cards/proxy", {
        method: "post",
        headers: { Accept: "application/json", "Content-Type": "application/json" },
        body: JSON.stringify({ cards: selected, isProxy }),
      })
      .then(() => {
        triggerRefresh(true);
        selectedDispatch({ type: "empty" });
      });
  };

  return (
    <div className="proxy-cards-actions" aria-label="Proxy card actions">
      <button
        type="button"
        className="btn btn-outline-info btn-sm"
        disabled={selected.length === 0}
        onClick={() => setProxy(true)}
      >
        Mark proxy
      </button>
      <button
        type="button"
        className="btn btn-outline-secondary btn-sm"
        disabled={selected.length === 0}
        onClick={() => setProxy(false)}
      >
        Mark regular
      </button>
    </div>
  );
}
