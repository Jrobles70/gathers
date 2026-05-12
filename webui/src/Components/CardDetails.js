import React, { useEffect, useState } from "react";
import { useCollection, useCollections } from "./CollectionContext";
import { useOperations, useMode } from "../OperationsContext";
import { useCardsDispatch } from "../Components/CardListContexts/CardsContext";
import { useRefreshCardList } from "./CardListContexts/RefreshCardListContext";
import { parseCents } from "./priceUtils";

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

function centsToInput(cents) {
  if (cents == null) return "";
  return (Number(cents) / 100).toFixed(2);
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
  const owningCollection = collections.find((item) => item.id === details?.collectionId);
  const proxyFromCollection = Boolean(owningCollection?.isProxy);
  const detailsQuantity = details?.quantity ?? 0;
  const detailsFoilQuantity = details?.foilQuantity ?? 0;
  const [selectedCollection, setSelectedCollection] = useState(null);
  const [foilMode, setFoilMode] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [purchasePriceInput, setPurchasePriceInput] = useState(() =>
    centsToInput(details?.purchasePrice?.usdCents),
  );
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

  useEffect(() => {
    setPurchasePriceInput(centsToInput(details?.purchasePrice?.usdCents));
  }, [details?.purchasePrice?.usdCents, id]);

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
      "/collection/cards/" + encodeURIComponent(collection) + "/" + (add ? "add" : "delete");
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
        const updatedCard = Array.isArray(data) ? data[0] : data;
        setQuantitiesByPrinting((previous) => ({
          ...previous,
          [id]: applyQuantityDelta(
            previous[id] ?? detailsToQuantities(details),
            parseInt(delta),
            parseInt(deltaFoil),
          ),
        }));
        if (cardsDispatch && updatedCard != null) {
          cardsDispatch({ type: "added", card: updatedCard });
        }
        if (
          triggerRefresh &&
          updatedCard != null &&
          updatedCard.quantity === 0 &&
          updatedCard.foilQuantity === 0
        ) {
          triggerRefresh(true);
        }
      });
  };

  const handleAction = (event, action) => {
    event.preventDefault();
    event.stopPropagation();
    action();
  };

  const savePurchasePrice = () => {
    if (!hasDetails) return;
    const purchasePriceCents = purchasePriceInput.trim() === ""
      ? null
      : parseCents(purchasePriceInput);

    ops
      .fetch("Updating purchase price for card " + id, {}, `/collection/cards/${encodeURIComponent(details.collectionId)}/purchase-price`, {
        method: "post",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          id,
          purchasePriceCents,
        }),
      })
      .then((updatedCard) => {
        if (cardsDispatch && updatedCard != null) {
          cardsDispatch({ type: "added", card: updatedCard });
        }
      });
  };

  const setCardProxy = (isProxy) => {
    if (!hasDetails) return;

    ops
      .fetch("Updating proxy status for card " + id, {}, `/collection/cards/${encodeURIComponent(details.collectionId)}/proxy`, {
        method: "post",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          id,
          isProxy,
        }),
      })
      .then((updatedCard) => {
        if (cardsDispatch && updatedCard != null) {
          cardsDispatch({ type: "added", card: updatedCard });
        }
        if (triggerRefresh) {
          triggerRefresh(true);
        }
      });
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
            {hasDetails && (
              <label className="search-card-menu-row" title={proxyFromCollection ? "Proxy is inherited from this collection" : "Track this card as proxy"}>
                <input
                  type="checkbox"
                  checked={Boolean(details.isProxy)}
                  disabled={proxyFromCollection}
                  onChange={(event) => setCardProxy(event.target.checked)}
                />
                <span>{proxyFromCollection ? "Proxy via collection" : "Proxy card"}</span>
              </label>
            )}
            {hasDetails && (
              <label className="search-card-menu-field">
                <span>Purchase price</span>
                <div className="search-card-price-edit">
                  <input
                    type="number"
                    min="0"
                    step="0.01"
                    inputMode="decimal"
                    value={purchasePriceInput}
                    onChange={(event) => setPurchasePriceInput(event.target.value)}
                    onClick={(event) => event.stopPropagation()}
                    className="form-control form-control-sm"
                  />
                  <button
                    type="button"
                    className="search-card-menu-button search-card-price-save"
                    onClick={(event) => handleAction(event, savePurchasePrice)}
                  >
                    Save
                  </button>
                </div>
              </label>
            )}
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
