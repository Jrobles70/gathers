import React, { useState, useEffect } from "react";
import { Link, useLocation } from "react-router-dom";
import CardDetails from "./CardDetails";
import { useSelectedCardsDispatch } from "./CardListContexts/SelectedCardsContext";
import { useCardLoader } from "./CardListContexts/CardLoaderContext";

export default function CardShell({ id, card = null, details = null, provider = null, detailPath, getImagePath, showCollectionSelect = false, listMode = false, targetCollection = null }) {
  const [_card, setCard] = useState(card);
  const [loadFailed, setLoadFailed] = useState(false);
  const [selected, setSelected] = useState(false);

  const selectedDispatch = useSelectedCardsDispatch();
  const loader = useCardLoader();
  const location = useLocation();
  const detailState = { returnTo: `${location.pathname}${location.search}` };

  const toggleSelected = () => {
    if (details != null) {
      selectedDispatch({ type: !selected ? "added" : "deleted", card: details });
      setSelected((s) => !s);
    }
  };

  useEffect(() => {
    if (_card == null) {
      loader(id, provider).then(setCard).catch(() => setLoadFailed(true));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id, _card, details, provider]);

  if (loadFailed) return null;

  const imagePath = _card != null ? getImagePath(_card) : "";

  if (listMode) {
    return (
      <div
        className={"card-list-item" + (selected ? " selected" : "")}
        onClick={toggleSelected}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => e.key === "Enter" && toggleSelected()}
      >
        {_card == null ? (
          <span className="text-muted">Loading…</span>
        ) : (
          <>
            <span className="card-list-name">
              <Link to={detailPath} state={detailState} onClick={(e) => e.stopPropagation()}>{_card.name}</Link>
            </span>
            <span className="card-list-set text-muted">{_card.setCode}</span>
            <span className="card-list-rarity text-muted">{_card.rarity ?? ""}</span>
            {details != null && (
              <>
                <span className="card-list-qty badge bg-secondary">×{details.quantity}</span>
                {details.foilQuantity > 0 && (
                  <span className="card-list-foil badge bg-info text-dark ms-1">✦×{details.foilQuantity}</span>
                )}
                <span className="card-list-provider badge bg-dark ms-1">{details.provider}</span>
              </>
            )}
          </>
        )}
      </div>
    );
  }

  return (
    <>
      {_card == null ? (
        <p>Loading...</p>
      ) : (
        <div className={"card search-card" + (selected ? " border border-primary" : "")}>
          <div className="search-card-art">
            <Link
              to={detailPath}
              state={detailState}
              className="search-card-image-link"
              aria-label={`Open details for ${_card.name}`}
            >
              <img src={imagePath} alt={_card.name} loading="lazy" />
            </Link>
            <CardDetails
              id={id}
              details={details}
              toggleSelected={toggleSelected}
              showCollectionSelect={showCollectionSelect}
              targetCollection={targetCollection}
            />
          </div>
          <div className="search-card-footer">
            <span className="search-card-price">$-</span>
            <span className="search-card-set">{_card.setCode}</span>
          </div>
        </div>
      )}
    </>
  );
}
