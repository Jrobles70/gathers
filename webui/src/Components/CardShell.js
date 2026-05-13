import React, { useEffect, useMemo, useRef, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import CardDetails from "./CardDetails";
import { useSelectedCards, useSelectedCardsDispatch } from "./CardListContexts/SelectedCardsContext";
import { useCardLoader } from "./CardListContexts/CardLoaderContext";
import { formatCents, formatPercent, priceTrend, unitPriceCents } from "./priceUtils";

const PROVIDER_LABELS = {
  MagicSQLite: "Magic",
  PokemonSQLite: "Pokémon",
  RiftboundSQLite: "Riftbound",
  Scryfall: "Scryfall",
};

function providerLabel(provider) {
  if (!provider) return "";
  return PROVIDER_LABELS[provider] ?? provider.replace(/SQLite$/, "");
}

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
  priceMode = "search",
  onLoad = null,
  fetchCycle = 0,
}) {
  const printingOptions = useMemo(
    () => (printings.length > 0 ? printings : [{ id, card, details }]),
    [card, details, id, printings],
  );
  const [selectedPrintingId, setSelectedPrintingId] = useState(id);
  const [loadFailed, setLoadFailed] = useState(false);
  const [printingPickerOpen, setPrintingPickerOpen] = useState(false);
  const [printingPickerAlign, setPrintingPickerAlign] = useState("left");
  const [hovered, setHovered] = useState(false);
  const cardRef = useRef(null);
  const cardDetailsRef = useRef(null);

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

  const selectedCards = useSelectedCards();
  const selectedDispatch = useSelectedCardsDispatch();
  const selected = selectedCards.some(
    (c) => c.id === activeDetails?.id && c.collectionId === activeDetails?.collectionId
  );
  const loader = useCardLoader();
  const location = useLocation();
  const onLoadFiredRef = useRef(false);

  // When a new fetch cycle starts, reset the fired flag and immediately report
  // if this card's data is already available (e.g. cached or re-render).
  useEffect(() => {
    onLoadFiredRef.current = false;
    if ((_card != null || loadFailed) && onLoad) {
      onLoadFiredRef.current = true;
      onLoad();
    }
    // Intentionally omit _card / loadFailed — only re-run when fetchCycle changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fetchCycle, onLoad]);

  useEffect(() => {
    if (_card != null && !onLoadFiredRef.current) {
      onLoadFiredRef.current = true;
      onLoad?.();
    }
  }, [_card, onLoad]);

  useEffect(() => {
    if (loadFailed && !onLoadFiredRef.current) {
      onLoadFiredRef.current = true;
      onLoad?.();
    }
  }, [loadFailed, onLoad]);

  const detailState = { returnTo: detailReturnPath ?? `${location.pathname}${location.search}` };

  const toggleSelected = () => {
    if (activeDetails != null) {
      selectedDispatch({ type: !selected ? "added" : "deleted", card: activeDetails });
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

  useEffect(() => {
    if (!hovered) return;
    const handleKeyDown = (e) => {
      if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA" || e.target.isContentEditable) return;
      if (e.key === "f" || e.key === "F") {
        e.preventDefault();
        cardDetailsRef.current?.toggleFoil();
      } else if ((e.key === "p" || e.key === "P") && printingOptions.length > 1) {
        e.preventDefault();
        if (cardRef.current) {
          const rect = cardRef.current.getBoundingClientRect();
          const pickerWidth = Math.min(544, window.innerWidth - 32);
          setPrintingPickerAlign(rect.left + pickerWidth > window.innerWidth - 8 ? "right" : "left");
        }
        setPrintingPickerOpen((open) => !open);
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [hovered, printingOptions.length]);

  if (loadFailed) return null;

  const imagePath = _card != null ? getImagePath(_card) : "";
  const activeProviderLabel = providerLabel(activeDetails?.provider);
  const activeUnitPrice = unitPriceCents(_card?.price, activeDetails);
  const activeTrend = priceMode === "collection" ? priceTrend(_card?.price, activeDetails) : null;
  const priceClass = [
    "search-card-price",
    activeTrend?.direction === "up" ? "price-up" : "",
    activeTrend?.direction === "down" ? "price-down" : "",
  ].filter(Boolean).join(" ");
  const priceText = [
    formatCents(activeUnitPrice),
    activeTrend && activeTrend.direction !== "flat"
      ? `(${formatPercent(activeTrend.changePercent)})`
      : null,
  ].filter(Boolean).join(" ");

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
                {activeProviderLabel && (
                  <span className="card-list-provider badge bg-dark ms-1">{activeProviderLabel}</span>
                )}
                {activeDetails.isProxy && (
                  <span className="card-list-proxy badge bg-warning text-dark ms-1">Proxy</span>
                )}
              </>
            )}
            <span className={activeDetails?.isProxy ? "search-card-price" : priceClass}>
              {activeDetails?.isProxy ? formatCents(activeUnitPrice) : priceText}
            </span>
          </>
        )}
      </div>
    );
  }

  if (_card == null) return null;

  return (
    <div
      ref={cardRef}
          className={"card search-card" + (selected ? " card-selected" : "")}
          onMouseEnter={() => setHovered(true)}
          onMouseLeave={() => setHovered(false)}
        >
          <div className="search-card-art">
            <button
              type="button"
              className={"card-checkbox" + (hovered || selected ? " visible" : "")}
              onClick={(e) => { e.preventDefault(); e.stopPropagation(); toggleSelected(); }}
              aria-label={selected ? "Deselect card" : "Select card"}
            >
              {selected ? "✓" : "○"}
            </button>
            <Link
              to={activeDetailPath}
              state={detailState}
              className="search-card-image-link"
              aria-label={`Open details for ${_card.name}`}
            >
              <img src={imagePath} alt={_card.name} loading="lazy" />
            </Link>
            <CardDetails
              ref={cardDetailsRef}
              id={activeId}
              details={activeDetails}
              showCollectionSelect={showCollectionSelect}
              targetCollection={targetCollection}
              hasPrintings={printingOptions.length > 1}
              onOpenPrintings={() => {
                if (cardRef.current) {
                  const rect = cardRef.current.getBoundingClientRect();
                  const pickerWidth = Math.min(544, window.innerWidth - 32);
                  setPrintingPickerAlign(rect.left + pickerWidth > window.innerWidth - 8 ? "right" : "left");
                }
                setPrintingPickerOpen(true);
              }}
            />
            {printingPickerOpen && (
              <div
                className={"search-card-printing-picker" + (printingPickerAlign === "right" ? " align-right" : "")}
                role="dialog"
                aria-label="Select a printing"
              >
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
                      <span className="search-card-printing-price">
                        {formatCents(unitPriceCents(printing.card?.price, printing.details))}
                      </span>
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>
          <div className="search-card-footer">
            <span className="search-card-price">{formatCents(activeUnitPrice)}</span>
            {!activeDetails?.isProxy && activeTrend && activeTrend.direction !== "flat" && (
              <span className={["search-card-price-delta", activeTrend.direction === "up" ? "price-up" : "price-down"].join(" ")}>
                {(activeTrend.changeCents >= 0 ? "+" : "") + formatCents(activeTrend.changeCents)} ({formatPercent(activeTrend.changePercent)})
              </span>
            )}
            <span className="search-card-footer-meta">
              {activeDetails?.collectionId && (
                <span className="collection-pill" title={activeDetails.collectionId}>
                  {activeDetails.collectionId}
                </span>
              )}
              {activeDetails?.isProxy && <span className="proxy-pill">Proxy</span>}
              <span className="search-card-set">{_card.setCode}</span>
            </span>
          </div>
    </div>
  );
}
