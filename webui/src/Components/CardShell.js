import React, { useEffect, useMemo, useRef, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import CardDetails from "./CardDetails";
import { useSelectedCardsDispatch } from "./CardListContexts/SelectedCardsContext";
import { useCardLoader } from "./CardListContexts/CardLoaderContext";

export default function CardShell({
  id,
  card = null,
  details = null,
  provider = null,
  detailPath,
  makeDetailPath = null,
  getImagePath,
  showCollectionSelect = false,
  listMode = false,
  targetCollection = null,
  printings = [],
  detailReturnPath = null,
}) {
  const printingOptions = useMemo(
    () => (printings.length > 0 ? printings : [{ id, card, details }]),
    [card, details, id, printings],
  );
  const [selectedPrintingId, setSelectedPrintingId] = useState(id);
  const [loadFailed, setLoadFailed] = useState(false);
  const [selected, setSelected] = useState(false);
  const [printingPickerOpen, setPrintingPickerOpen] = useState(false);

  useEffect(() => {
    setSelectedPrintingId(id);
  }, [id]);

  const selectedPrinting = useMemo(
    () => printingOptions.find((printing) => printing.id === selectedPrintingId) ?? printingOptions[0],
    [printingOptions, selectedPrintingId],
  );
  const activeId = selectedPrinting?.id ?? id;
  const activeDetails = selectedPrinting?.details ?? details;
  const cardFromProps = selectedPrinting?.card ?? card;
  const activeDetailPath = makeDetailPath ? makeDetailPath(activeId) : detailPath;
  const activeCardKey = `${provider ?? "default"}:${activeId}`;
  const [_card, setCard] = useState(cardFromProps);
  const loadedCardKeyRef = useRef(cardFromProps != null ? activeCardKey : null);

  const selectedDispatch = useSelectedCardsDispatch();
  const loader = useCardLoader();
  const location = useLocation();
  const detailState = { returnTo: detailReturnPath ?? `${location.pathname}${location.search}` };

  const toggleSelected = () => {
    if (activeDetails != null) {
      selectedDispatch({ type: !selected ? "added" : "deleted", card: activeDetails });
      setSelected((s) => !s);
    }
  };

  useEffect(() => {
    let cancelled = false;
    setLoadFailed(false);

    if (cardFromProps != null) {
      loadedCardKeyRef.current = activeCardKey;
      setCard(cardFromProps);
      return undefined;
    }

    if (loadedCardKeyRef.current !== activeCardKey) {
      setCard(null);
    }
    if (loader != null) {
      loader(activeId, provider)
        .then((loadedCard) => {
          if (!cancelled) {
            loadedCardKeyRef.current = activeCardKey;
            setCard(loadedCard);
          }
        })
        .catch(() => {
          if (!cancelled) setLoadFailed(true);
        });
    }

    return () => {
      cancelled = true;
    };
  }, [activeCardKey, activeId, cardFromProps, loader, provider]);

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
              <Link to={activeDetailPath} state={detailState} onClick={(e) => e.stopPropagation()}>{_card.name}</Link>
            </span>
            <span className="card-list-set text-muted">{_card.setCode}</span>
            <span className="card-list-rarity text-muted">{_card.rarity ?? ""}</span>
            {activeDetails != null && (
              <>
                <span className="card-list-qty badge bg-secondary">×{activeDetails.quantity}</span>
                {activeDetails.foilQuantity > 0 && (
                  <span className="card-list-foil badge bg-info text-dark ms-1">✦×{activeDetails.foilQuantity}</span>
                )}
                <span className="card-list-provider badge bg-dark ms-1">{activeDetails.provider}</span>
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
              to={activeDetailPath}
              state={detailState}
              className="search-card-image-link"
              aria-label={`Open details for ${_card.name}`}
            >
              <img src={imagePath} alt={_card.name} loading="lazy" />
            </Link>
            <CardDetails
              id={activeId}
              details={activeDetails}
              showCollectionSelect={showCollectionSelect}
              targetCollection={targetCollection}
              hasPrintings={printingOptions.length > 1}
              onOpenPrintings={() => setPrintingPickerOpen(true)}
            />
            {printingPickerOpen && (
              <div className="search-card-printing-picker" role="dialog" aria-label="Select a printing">
                <div className="search-card-printing-header">
                  <strong>Select a printing</strong>
                  <button
                    type="button"
                    className="search-card-printing-close"
                    onClick={(event) => {
                      event.preventDefault();
                      event.stopPropagation();
                      setPrintingPickerOpen(false);
                    }}
                    aria-label="Close printing picker"
                  >
                    ×
                  </button>
                </div>
                <div className="search-card-printing-list">
                  {printingOptions.map((printing) => (
                    <button
                      type="button"
                      key={printing.id}
                      className={
                        "search-card-printing-option" +
                        (printing.id === activeId ? " selected" : "")
                      }
                      onClick={(event) => {
                        event.preventDefault();
                        event.stopPropagation();
                        setSelectedPrintingId(printing.id);
                        setPrintingPickerOpen(false);
                      }}
                    >
                      {printing.card != null && (
                        <img src={getImagePath(printing.card)} alt="" loading="lazy" />
                      )}
                      <span className="search-card-printing-meta">
                        <span>{printing.card?.name ?? printing.id}</span>
                        <span>
                          {printing.card?.setCode ?? ""}
                          {printing.card?.collectorNumber ? ` #${printing.card.collectorNumber}` : ""}
                        </span>
                      </span>
                      <span className="search-card-printing-price">$-</span>
                    </button>
                  ))}
                </div>
              </div>
            )}
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
