import React, { useEffect, useState } from "react";
import { useCollection, useCollections } from "./CollectionContext";
import { useOperations, useMode } from "../OperationsContext";
import { useCardsDispatch } from "../Components/CardListContexts/CardsContext";
import { useRefreshCardList } from "./CardListContexts/RefreshCardListContext";

export function resolveCardUpdateCollection({
  details,
  showCollectionSelect,
  selectedCollection,
  collections,
  currentCollection,
  targetCollection,
}) {
  if (details != null) return details.collectionId;
  if (showCollectionSelect) return selectedCollection ?? collections[0]?.id ?? currentCollection;
  return targetCollection ?? currentCollection;
}

export function applyQuantityDelta(quantities, delta, deltaFoil) {
  return {
    quantity: Math.max(0, quantities.quantity + delta),
    foilQuantity: Math.max(0, quantities.foilQuantity + deltaFoil),
  };
}

function detailsToQuantities(details) {
  return {
    quantity: details?.quantity ?? 0,
    foilQuantity: details?.foilQuantity ?? 0,
  };
}

export default function CardDetails({
  id,
  details = null,
  showCollectionSelect = false,
  targetCollection = null,
  hasPrintings = false,
  onOpenPrintings = null,
}) {
  const ops = useOperations();
  const { collectionsEnabled } = useMode();
  const currentCollection = useCollection();
  const collections = useCollections();
  const cardsDispatch = useCardsDispatch();
  const triggerRefresh = useRefreshCardList();
  const hasDetails = details != null;
  const detailsQuantity = details?.quantity ?? 0;
  const detailsFoilQuantity = details?.foilQuantity ?? 0;
  const [selectedCollection, setSelectedCollection] = useState(null);
  const [foilMode, setFoilMode] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [quantitiesByPrinting, setQuantitiesByPrinting] = useState(() => ({
    [id]: {
      quantity: detailsQuantity,
      foilQuantity: detailsFoilQuantity,
    },
  }));

  useEffect(() => {
    setQuantitiesByPrinting((previous) => ({
      ...previous,
      [id]: !hasDetails && previous[id] != null
        ? previous[id]
        : {
          quantity: detailsQuantity,
          foilQuantity: detailsFoilQuantity,
        },
    }));
  }, [id, hasDetails, detailsQuantity, detailsFoilQuantity]);

  const updateQuantity = (delta, deltaFoil) => {
    if (!collectionsEnabled && details == null) return;

    let collection = resolveCardUpdateCollection({
      details,
      showCollectionSelect,
      selectedCollection,
      collections,
      currentCollection,
      targetCollection,
    });
    let add = parseInt(delta) >= 0 && parseInt(deltaFoil) >= 0;
    let url =
      "/collection/cards/" + collection + "/" + (add ? "add" : "delete");
    let body = {
      id: id,
      collectionId: collection,
      quantity: Math.abs(parseInt(delta)),
      foilQuantity: Math.abs(parseInt(deltaFoil)),
    };

    ops
      .fetch("Updating quantities for card " + id, {}, url, {
        method: "post",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
      })
      .then((data) => {
        setQuantitiesByPrinting((previous) => ({
          ...previous,
          [id]: applyQuantityDelta(
            previous[id] ?? detailsToQuantities(details),
            parseInt(delta),
            parseInt(deltaFoil),
          ),
        }));
        if (cardsDispatch) {
          cardsDispatch({ type: "added", card: add ? data[0] : data });
        }
        if (triggerRefresh) {
          triggerRefresh(true);
        }
      });
  };

  const handleAction = (event, action) => {
    event.preventDefault();
    event.stopPropagation();
    action();
  };

  const quantities = quantitiesByPrinting[id] ?? detailsToQuantities(details);
  const activeQuantity = foilMode ? quantities.foilQuantity : quantities.quantity;
  const canAdjustQuantity = hasDetails || collectionsEnabled;

  if (!canAdjustQuantity) return null;

  return (
    <div className="card-img-overlay search-card-overlay">
      <div className="search-card-action-rail" aria-label="Card quantity controls">
        <button
          type="button"
          onClick={(event) => handleAction(event, () => updateQuantity(foilMode ? 0 : 1, foilMode ? 1 : 0))}
          className="search-card-action"
          aria-label={foilMode ? "Increase foil quantity" : "Increase quantity"}
        >
          +
        </button>
        <output className="search-card-quantity" aria-label={foilMode ? "Foil quantity" : "Quantity"}>
          {activeQuantity}
        </output>
        <button
          type="button"
          onClick={(event) => handleAction(event, () => updateQuantity(foilMode ? 0 : -1, foilMode ? -1 : 0))}
          className="search-card-action"
          disabled={activeQuantity <= 0}
          aria-label={foilMode ? "Decrease foil quantity" : "Decrease quantity"}
        >
          -
        </button>
        <button
          type="button"
          onClick={(event) => handleAction(event, () => setMenuOpen((open) => !open))}
          className="search-card-action"
          aria-haspopup="menu"
          aria-expanded={menuOpen}
          aria-label="More card actions"
        >
          ...
        </button>
        {menuOpen && (
          <div className="search-card-menu" role="menu">
            <div className="search-card-menu-title">
              {foilMode ? "Foil" : "Regular"} quantity
            </div>
            {showCollectionSelect && collections.length > 0 && (
              <label className="search-card-menu-field">
                <span>Collection</span>
                <select
                  value={selectedCollection ?? collections[0]?.id ?? ""}
                  onChange={(event) => setSelectedCollection(event.target.value)}
                  onClick={(event) => event.stopPropagation()}
                  className="form-select form-select-sm"
                >
                  {collections.map((c) => (
                    <option key={c.id} value={c.id}>{c.id}</option>
                  ))}
                </select>
              </label>
            )}
            <label className="search-card-menu-row">
              <input
                type="checkbox"
                checked={foilMode}
                onChange={(event) => setFoilMode(event.target.checked)}
              />
              <span>Foil</span>
            </label>
            <button
              type="button"
              className="search-card-menu-button"
              disabled={!hasPrintings || onOpenPrintings == null}
              onClick={(event) => handleAction(event, () => {
                setMenuOpen(false);
                onOpenPrintings();
              })}
            >
              Switch printing
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
