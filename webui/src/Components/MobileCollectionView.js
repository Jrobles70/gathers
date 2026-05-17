import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import CardList from "./CardList";
import CollectionFilterBar from "./CollectionFilterBar";
import {
  collectionDisplayName,
  isAllCollections,
  useCollection,
  useCollections,
  useCollectionsDispatch,
  usePageNumber,
} from "./CollectionContext";
import { buildNewCollectionImportFormData } from "./AddCollectionForm";
import { useOperations } from "../OperationsContext";
import { useQuickSearch } from "./QuickSearchContext";
import { formatCents, formatPercent, parseCents, priceTrend, unitPriceCents } from "./priceUtils";
import { useCards, useCardsDispatch } from "./CardListContexts/CardsContext";
import { useCardLoader } from "./CardListContexts/CardLoaderContext";
import { useRefreshCardList } from "./CardListContexts/RefreshCardListContext";

const ACCENT_COLORS = ["#ec1f66", "#8cc84b", "#ffbe1b", "#ff4133", "#ff7b22"];

function formatMobileCents(cents) {
  if (cents == null || Number.isNaN(Number(cents))) return "$-";
  const amount = Number(cents) / 100;
  if (Math.abs(amount) >= 1000) return `$${(amount / 1000).toFixed(amount >= 10000 ? 0 : 2)}K`;
  return formatCents(cents).replace(/\.00$/, "");
}

function changeText(stats) {
  if (stats?.changeCents == null || stats?.changePercent == null) return "No baseline";
  const sign = stats.changeCents > 0 ? "+" : "";
  return `${sign}${formatCents(stats.changeCents).replace(/\.00$/, "")} (${formatPercent(stats.changePercent)})`;
}

function trendClass(stats) {
  if ((stats?.changeCents ?? 0) > 0) return "price-up";
  if ((stats?.changeCents ?? 0) < 0) return "price-down";
  return "";
}

function getCardImagePath(card) {
  if (!card) return "";
  // MTG / Scryfall
  if (card.cardIdentifiers?.scryfallId) {
    return `https://api.scryfall.com/cards/${card.cardIdentifiers.scryfallId}?format=image`;
  }
  // Riftbound / Pokemon — use .image if present
  if (card.image) return card.image;
  return "";
}

function MobileTopBar({ title, backTo = null }) {
  return (
    <header className="mobile-collection-topbar">
      {backTo ? (
        <Link className="mobile-icon-button" to={backTo} aria-label="Back to collections">
          ‹
        </Link>
      ) : (
        <span className="mobile-topbar-spacer" aria-hidden="true" />
      )}
      <h1>{title}</h1>
      <button type="button" className="mobile-icon-button" aria-label="More options">
        ...
      </button>
    </header>
  );
}

export function MobileBottomNav({ activeTab = "collection" }) {
  return (
    <nav className="mobile-bottom-nav" aria-label="Primary">
      <Link to="/collections/1" className={"mobile-bottom-nav-item" + (activeTab === "home" ? " active" : "")}>
        <span aria-hidden="true">⌂</span>
        Home
      </Link>
      <Link to="/search" className={"mobile-bottom-nav-item" + (activeTab === "search" ? " active" : "")}>
        <span aria-hidden="true">⌕</span>
        Search
      </Link>
      <Link to="/collections/1" className={"mobile-bottom-nav-item" + (activeTab === "collection" ? " active" : "")}>
        <span aria-hidden="true">▣</span>
        Collection
      </Link>
      <button type="button" className="mobile-bottom-nav-item">
        <span aria-hidden="true">▱</span>
        Decks
      </button>
      <button type="button" className="mobile-bottom-nav-item">
        <span aria-hidden="true">▢</span>
        Scan
      </button>
    </nav>
  );
}

// B1: Stats Header with scroll-fade (replaces MobileCollectionSummary)
function MobileStatsHeader({ stats }) {
  const wrapperRef = useRef(null);

  useEffect(() => {
    const el = wrapperRef.current;
    if (!el) return;

    // Scroll handler: fade out over ~80px as stats block leaves viewport top
    function handleScroll() {
      const rect = el.getBoundingClientRect();
      // rect.bottom is how far the bottom of the stats block is from the top of viewport
      // When rect.bottom goes from ~height to 0, we fade out
      const fadeRange = 80;
      const opacity = Math.max(0, Math.min(1, rect.bottom / fadeRange));
      el.style.setProperty("--stats-opacity", opacity);
    }

    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const nonProxyValue = (stats?.totalValueCents ?? 0) - (stats?.proxyTotalValueCents ?? 0);
  const nonProxyCount = (stats?.copyCount ?? 0) - (stats?.proxyCopyCount ?? 0);

  return (
    <section
      ref={wrapperRef}
      className="mobile-stats-header"
      aria-label="Collection value"
    >
      <div className="mobile-stats-value">{formatMobileCents(nonProxyValue)}</div>
      <div className={"mobile-stats-change " + trendClass(stats)}>{changeText(stats)}</div>
      <div className="mobile-stats-count">{nonProxyCount} cards</div>
      {(stats?.proxyCopyCount ?? 0) > 0 && (
        <div className="mobile-stats-saved">
          Saved {formatMobileCents(stats?.proxyTotalValueCents)}
        </div>
      )}
    </section>
  );
}

function centsToInput(cents) {
  if (cents == null) return "";
  return (Number(cents) / 100).toFixed(2);
}

// B2: Card bottom sheet component
function MobileCardSheet({ cards, initialIndex, onClose }) {
  const [activeIndex, setActiveIndex] = useState(initialIndex);
  const carouselRef = useRef(null);
  const touchStartY = useRef(null);
  const touchStartX = useRef(null);
  const carouselTouchStartY = useRef(null);
  const carouselTouchStartX = useRef(null);
  const scrollTimeout = useRef(null);

  // Inline action state
  const ops = useOperations();
  const cardsDispatch = useCardsDispatch();
  const triggerRefresh = useRefreshCardList();
  const [foilMode, setFoilMode] = useState(false);
  const [quantitiesByPrinting, setQuantitiesByPrinting] = useState({});
  const [purchasePriceInput, setPurchasePriceInput] = useState("");

  // Fix 1: Prevent background scroll while sheet is open
  useEffect(() => {
    const prev = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    return () => { document.body.style.overflow = prev; };
  }, []);

  // Load card metadata for a window of cards around the active index
  const loader = useCardLoader();
  const cardDataCache = useRef(new Map()); // index → card data
  const [cacheVersion, setCacheVersion] = useState(0); // bumped when active card loads
  const activeDetails = cards[activeIndex]; // flat details object IS the item

  // Reset per-card state when the active card changes
  useEffect(() => {
    setFoilMode(false);
    if (activeDetails) {
      setQuantitiesByPrinting((prev) => ({
        ...prev,
        [activeDetails.id]: {
          quantity: activeDetails.quantity ?? 0,
          foilQuantity: activeDetails.foilQuantity ?? 0,
        },
      }));
      setPurchasePriceInput(centsToInput(activeDetails.purchasePrice?.usdCents));
    }
  }, [activeIndex]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    if (!loader || cards.length === 0) return;

    const WINDOW = 4;
    const lo = Math.max(0, activeIndex - WINDOW);
    const hi = Math.min(cards.length - 1, activeIndex + WINDOW);

    const cancellers = [];
    for (let i = lo; i <= hi; i++) {
      if (cardDataCache.current.has(i)) continue;
      const entry = cards[i];
      if (!entry) continue;
      let cancelled = false;
      cancellers.push(() => { cancelled = true; });
      const capturedIndex = i;
      loader(entry.id, entry.provider).then((card) => {
        if (cancelled) return;
        cardDataCache.current.set(capturedIndex, card);
        // Re-render whenever any card in the window loads so its image appears
        setCacheVersion((v) => v + 1);
      }).catch(() => {});
    }

    return () => cancellers.forEach((cancel) => cancel());
  }, [activeIndex, cards, loader]);

  const activeCardData = cardDataCache.current.get(activeIndex) ?? null;

  // Scroll carousel to the initial card on mount without smooth scroll
  useEffect(() => {
    const carousel = carouselRef.current;
    if (!carousel) return;
    const slideWidth = window.innerWidth * 0.70;
    carousel.style.scrollBehavior = "auto";
    carousel.scrollLeft = initialIndex * slideWidth;
    carousel.style.scrollBehavior = "";
  }, [initialIndex]);

  // Update activeIndex as user swipes — slide width is 70vw, not full carousel width
  const handleCarouselScroll = useCallback(() => {
    if (scrollTimeout.current) clearTimeout(scrollTimeout.current);
    scrollTimeout.current = setTimeout(() => {
      const carousel = carouselRef.current;
      if (!carousel) return;
      const slideWidth = window.innerWidth * 0.70;
      const newIndex = Math.round(carousel.scrollLeft / slideWidth);
      setActiveIndex((prev) =>
        newIndex >= 0 && newIndex < cards.length ? newIndex : prev
      );
    }, 50);
  }, [cards.length]);

  useEffect(() => {
    return () => {
      if (scrollTimeout.current) clearTimeout(scrollTimeout.current);
    };
  }, []);

  // Swipe-to-dismiss: track both axes so horizontal carousel scroll doesn't trigger dismiss
  function handleTouchStart(e) {
    if (e.target.closest(".mobile-sheet-detail")) return;
    touchStartY.current = e.touches[0].clientY;
    touchStartX.current = e.touches[0].clientX;
  }

  function handleTouchEnd(e) {
    if (touchStartY.current == null) return;
    const deltaY = e.changedTouches[0].clientY - touchStartY.current;
    const deltaX = e.changedTouches[0].clientX - touchStartX.current;
    touchStartY.current = null;
    touchStartX.current = null;
    if (deltaY > 80 && deltaY > Math.abs(deltaX)) {
      onClose();
    }
  }

  // Swipe-down on carousel dismisses the sheet with visual drag feedback
  function handleCarouselTouchStart(e) {
    carouselTouchStartY.current = e.touches[0].clientY;
    carouselTouchStartX.current = e.touches[0].clientX;
    const carousel = carouselRef.current;
    if (carousel) carousel.style.transition = "none";
  }

  function handleCarouselTouchMove(e) {
    if (carouselTouchStartY.current == null) return;
    const deltaY = e.touches[0].clientY - carouselTouchStartY.current;
    const deltaX = e.touches[0].clientX - carouselTouchStartX.current;
    if (deltaY > 0 && deltaY > Math.abs(deltaX)) {
      const carousel = carouselRef.current;
      if (carousel) {
        const translateY = deltaY * 0.65;
        carousel.style.transform = `translateY(${translateY}px)`;
        carousel.style.opacity = String(Math.max(0.2, 1 - deltaY / 250));
      }
    }
  }

  function handleCarouselTouchEnd(e) {
    if (carouselTouchStartY.current == null) return;
    const deltaY = e.changedTouches[0].clientY - carouselTouchStartY.current;
    const deltaX = e.changedTouches[0].clientX - carouselTouchStartX.current;
    carouselTouchStartY.current = null;
    carouselTouchStartX.current = null;
    const carousel = carouselRef.current;
    if (deltaY > 80 && deltaY > Math.abs(deltaX) * 1.5) {
      onClose();
    } else if (carousel) {
      carousel.style.transition = "transform 0.3s ease, opacity 0.3s ease";
      carousel.style.transform = "";
      carousel.style.opacity = "";
    }
  }

  // Inline action handlers
  const updateQuantity = useCallback((delta, deltaFoil) => {
    if (!activeDetails) return;
    const collectionId = activeDetails.collectionId;
    const id = activeDetails.id;
    const add = parseInt(delta) >= 0 && parseInt(deltaFoil) >= 0;
    const url = `/collection/cards/${encodeURIComponent(collectionId)}/${add ? "add" : "delete"}`;
    const body = {
      id,
      collectionId,
      quantity: Math.abs(parseInt(delta)),
      foilQuantity: Math.abs(parseInt(deltaFoil)),
    };

    // Optimistic update
    setQuantitiesByPrinting((prev) => {
      const current = prev[id] ?? { quantity: activeDetails.quantity ?? 0, foilQuantity: activeDetails.foilQuantity ?? 0 };
      return {
        ...prev,
        [id]: {
          quantity: Math.max(0, current.quantity + parseInt(delta)),
          foilQuantity: Math.max(0, current.foilQuantity + parseInt(deltaFoil)),
        },
      };
    });

    ops.fetch("Updating quantities for card " + id, {}, url, {
      method: "post",
      headers: { Accept: "application/json", "Content-Type": "application/json" },
      body: JSON.stringify(body),
    }).then((data) => {
      const updatedCard = Array.isArray(data) ? data[0] : data;
      if (cardsDispatch && updatedCard != null) {
        cardsDispatch({ type: "added", card: updatedCard });
      }
      if (triggerRefresh && updatedCard != null && updatedCard.quantity === 0 && updatedCard.foilQuantity === 0) {
        triggerRefresh(true);
      }
    });
  }, [activeDetails, ops, cardsDispatch, triggerRefresh]);

  const savePurchasePrice = useCallback(() => {
    if (!activeDetails) return;
    const id = activeDetails.id;
    const collectionId = activeDetails.collectionId;
    const purchasePriceCents = purchasePriceInput.trim() === "" ? null : parseCents(purchasePriceInput);

    ops.fetch("Updating purchase price for card " + id, {}, `/collection/cards/${encodeURIComponent(collectionId)}/purchase-price`, {
      method: "post",
      headers: { Accept: "application/json", "Content-Type": "application/json" },
      body: JSON.stringify({ id, purchasePriceCents }),
    }).then((updatedCard) => {
      if (cardsDispatch && updatedCard != null) {
        cardsDispatch({ type: "added", card: updatedCard });
      }
    });
  }, [activeDetails, purchasePriceInput, ops, cardsDispatch]);

  const setCardProxy = useCallback((isProxy) => {
    if (!activeDetails) return;
    const id = activeDetails.id;
    const collectionId = activeDetails.collectionId;

    ops.fetch("Updating proxy status for card " + id, {}, `/collection/cards/${encodeURIComponent(collectionId)}/proxy`, {
      method: "post",
      headers: { Accept: "application/json", "Content-Type": "application/json" },
      body: JSON.stringify({ id, isProxy }),
    }).then((updatedCard) => {
      if (cardsDispatch && updatedCard != null) {
        cardsDispatch({ type: "added", card: updatedCard });
      }
      if (triggerRefresh) {
        triggerRefresh(true);
      }
    });
  }, [activeDetails, ops, cardsDispatch, triggerRefresh]);

  // Bug 1: Price / trend info now comes from loaded card data and flat details
  const trend = activeCardData?.price != null
    ? priceTrend(activeCardData.price, activeDetails)
    : null;
  const unitPrice = activeCardData?.price != null
    ? unitPriceCents(activeCardData.price, activeDetails)
    : null;

  const activeQuantities = activeDetails
    ? (quantitiesByPrinting[activeDetails.id] ?? { quantity: activeDetails.quantity ?? 0, foilQuantity: activeDetails.foilQuantity ?? 0 })
    : { quantity: 0, foilQuantity: 0 };
  const activeQuantity = foilMode ? activeQuantities.foilQuantity : activeQuantities.quantity;
  const qty = activeQuantities.quantity + activeQuantities.foilQuantity;

  return (
    <>
      {/* Backdrop — dim but not opaque so grid shows through */}
      <div className="mobile-card-sheet-backdrop" onClick={onClose} />

      {/* Carousel — floats between top bar and bottom panel, outside the sheet */}
      <div
        ref={carouselRef}
        className="mobile-sheet-carousel"
        onScroll={handleCarouselScroll}
        onTouchStart={handleCarouselTouchStart}
        onTouchMove={handleCarouselTouchMove}
        onTouchEnd={handleCarouselTouchEnd}
      >
        {cards.map((card, index) => {
          const cachedData = cardDataCache.current.get(index);
          const imgSrc = cachedData ? getCardImagePath(cachedData) : "";
          const isActive = index === activeIndex;
          return (
            <div key={card.id ?? index} className={"mobile-sheet-carousel-slide" + (isActive ? " active" : "")}>
              {imgSrc ? (
                <img
                  src={imgSrc}
                  alt={cachedData?.name ?? "Card"}
                  loading="eager"
                />
              ) : (
                <div className="mobile-sheet-carousel-placeholder" aria-hidden="true" />
              )}
            </div>
          );
        })}
      </div>

      {/* Bottom panel — info + actions */}
      <div
        className="mobile-card-sheet"
        onTouchStart={handleTouchStart}
        onTouchEnd={handleTouchEnd}
        role="dialog"
        aria-label="Card detail"
        aria-modal="true"
      >
        <div className="mobile-sheet-handle" aria-hidden="true" />
        <div className="mobile-sheet-detail">
          <div className="mobile-sheet-detail-name">
            <span className="mobile-sheet-qty">×{qty}</span>
            <strong>{activeCardData?.name ?? "Loading…"}</strong>
          </div>
          <div className="mobile-sheet-detail-meta">
            {activeCardData?.setCode && (
              <span className="search-card-set">{activeCardData.setCode}</span>
            )}
            {activeDetails?.collectionId && (
              <span className="collection-pill" title={activeDetails.collectionId}>
                {activeDetails.collectionId}
              </span>
            )}
          </div>
          {unitPrice != null && (
            <div className="mobile-sheet-price-row">
              <span className="mobile-sheet-price">{formatCents(unitPrice)}</span>
              {!activeDetails?.isProxy && trend && trend.direction !== "flat" && (
                <span className={[
                  "mobile-sheet-price-delta",
                  trend.direction === "up" ? "price-up" : "price-down",
                ].join(" ")}>
                  {(trend.changeCents >= 0 ? "+" : "") + formatCents(trend.changeCents)}
                  {" "}({formatPercent(trend.changePercent)})
                </span>
              )}
            </div>
          )}
          <div className="mobile-sheet-actions">
            {/* Quantity row */}
            <div className="mobile-sheet-qty-row">
              <button
                type="button"
                onClick={() => updateQuantity(foilMode ? 0 : -1, foilMode ? -1 : 0)}
                disabled={activeQuantity <= 0}
                aria-label={foilMode ? "Decrease foil quantity" : "Decrease quantity"}
              >
                −
              </button>
              <span aria-label={foilMode ? "Foil quantity" : "Quantity"}>{activeQuantity}</span>
              <button
                type="button"
                onClick={() => updateQuantity(foilMode ? 0 : 1, foilMode ? 1 : 0)}
                aria-label={foilMode ? "Increase foil quantity" : "Increase quantity"}
              >
                +
              </button>
            </div>
            {/* Foil toggle */}
            <label className="mobile-sheet-toggle">
              <input
                type="checkbox"
                checked={foilMode}
                onChange={(e) => setFoilMode(e.target.checked)}
              />
              Foil
            </label>
            {/* Proxy toggle */}
            {activeDetails && (
              <label className="mobile-sheet-toggle">
                <input
                  type="checkbox"
                  checked={Boolean(activeDetails.isProxy)}
                  onChange={(e) => setCardProxy(e.target.checked)}
                />
                Proxy
              </label>
            )}
            {/* Purchase price */}
            {activeDetails && (
              <div>
                <div className="mobile-sheet-field-label">Purchase price</div>
                <div className="mobile-sheet-price-edit">
                  <input
                    type="number"
                    min="0"
                    step="0.01"
                    inputMode="decimal"
                    value={purchasePriceInput}
                    onChange={(e) => setPurchasePriceInput(e.target.value)}
                    placeholder="Purchase price"
                  />
                  <button type="button" onClick={savePurchasePrice}>Save</button>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  );
}

function MobileNewCollectionSheet({ onClose }) {
  const [name, setName] = useState("");
  const [importText, setImportText] = useState("");
  const [isProxy, setIsProxy] = useState(false);
  const [error, setError] = useState(null);
  const [submitting, setSubmitting] = useState(false);
  const touchStartY = useRef(null);
  const collectionsDispatch = useCollectionsDispatch();
  const ops = useOperations();

  useEffect(() => {
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => { document.body.style.overflow = prev; };
  }, []);

  function handleTouchStart(e) {
    touchStartY.current = e.touches[0].clientY;
  }

  function handleTouchEnd(e) {
    if (touchStartY.current == null) return;
    const deltaY = e.changedTouches[0].clientY - touchStartY.current;
    touchStartY.current = null;
    if (deltaY > 80) onClose();
  }

  function handleSubmit(e) {
    e.preventDefault();
    const trimmedName = name.trim();
    const trimmedText = importText.trim();
    if (!trimmedName) return;
    setError(null);
    setSubmitting(true);

    const save = trimmedText.length > 0
      ? ops.fetch("Importing new collection", {}, "/collection/import", {
          method: "POST",
          body: buildNewCollectionImportFormData({ name: trimmedName, text: importText }),
        })
      : ops.fetch("Adding new collection", {}, "/collection/add", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ id: trimmedName, isProxy }),
        });

    save
      .then(() =>
        trimmedText.length > 0 && isProxy
          ? ops.fetch("Marking collection as proxy", {}, `/collection/proxy/${encodeURIComponent(trimmedName)}`, {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ isProxy: true }),
            })
          : null
      )
      .then(() => {
        collectionsDispatch({ type: "added", item: { id: trimmedName, isProxy, canRemove: true } });
        onClose();
      })
      .catch((err) => {
        setError(err.message);
        setSubmitting(false);
      });
  }

  return (
    <>
      <div className="mobile-card-sheet-backdrop" onClick={onClose} />
      <div
        className="mobile-new-collection-sheet"
        onTouchStart={handleTouchStart}
        onTouchEnd={handleTouchEnd}
        role="dialog"
        aria-label="New collection"
        aria-modal="true"
      >
        <div className="mobile-sheet-handle" aria-hidden="true" />
        <form className="mobile-new-collection-form" onSubmit={handleSubmit}>
          <h2>New Collection</h2>
          <input
            type="text"
            className="mobile-new-collection-name"
            placeholder="Collection name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            autoFocus
          />
          {error && <div className="mobile-new-collection-error">{error}</div>}
          <div>
            <div className="mobile-sheet-field-label">Import cards (optional)</div>
            <textarea
              className="mobile-new-collection-import"
              placeholder="Name,Set code,Set name,Collector number,Foil,Rarity,Quantity,..."
              value={importText}
              onChange={(e) => setImportText(e.target.value)}
              rows={4}
            />
          </div>
          <label className="mobile-sheet-toggle">
            <input
              type="checkbox"
              checked={isProxy}
              onChange={(e) => setIsProxy(e.target.checked)}
            />
            Proxy collection
          </label>
          <div className="mobile-new-collection-actions">
            <button
              type="submit"
              disabled={!name.trim() || submitting}
              className="mobile-new-collection-submit"
            >
              {submitting ? "Creating…" : "Create"}
            </button>
            <button type="button" onClick={onClose} className="mobile-new-collection-cancel">
              Cancel
            </button>
          </div>
        </form>
      </div>
    </>
  );
}

function MobileCollectionOverview() {
  const { fetch: opsFetch, quietFetch } = useOperations();
  const collections = useCollections();
  const [query, setQuery] = useState("");
  const [newCollectionOpen, setNewCollectionOpen] = useState(false);
  const [allStats, setAllStats] = useState(null);
  const [collectionStats, setCollectionStats] = useState({});

  const { childrenByParent, topLevel } = useMemo(() => {
    const childrenByParent = {};
    collections.forEach((c) => {
      if (c.parent) {
        if (!childrenByParent[c.parent]) childrenByParent[c.parent] = [];
        childrenByParent[c.parent].push(c);
      }
    });
    return { childrenByParent, topLevel: collections.filter((c) => !c.parent) };
  }, [collections]);

  const filteredCollections = useMemo(
    () => topLevel.filter((c) => c.id.toLowerCase().includes(query.trim().toLowerCase())),
    [topLevel, query],
  );
  const visibleCollectionIds = filteredCollections.slice(0, 20).map((c) => c.id).join("\n");

  useEffect(() => {
    let cancelled = false;
    opsFetch("Getting all collection value", null, "/collection/stats").then((stats) => {
      if (!cancelled) setAllStats(stats);
    }).catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [opsFetch]);

  useEffect(() => {
    let cancelled = false;
    const visible = filteredCollections.slice(0, 20);
    Promise.all(
      visible.map((collection) =>
        (quietFetch
          ? quietFetch(null, `/collection/cards/${encodeURIComponent(collection.id)}/stats`)
          : opsFetch("Getting collection value", null, `/collection/cards/${encodeURIComponent(collection.id)}/stats`))
          .then((stats) => [collection.id, stats])
          .catch(() => [collection.id, null]),
      ),
    ).then((entries) => {
      if (!cancelled) setCollectionStats(Object.fromEntries(entries));
    });
    return () => {
      cancelled = true;
    };
  }, [opsFetch, quietFetch, visibleCollectionIds, filteredCollections]);

  return (
    <>
      <MobileTopBar title="Collection" />
      <MobileStatsHeader stats={allStats} />
      <div className="mobile-section-tabs" role="tablist" aria-label="Collection sections">
        <button type="button" className="active">
          <span aria-hidden="true">▣</span>
          Collection
        </button>
        <button type="button">
          <span aria-hidden="true">▱</span>
          Lists
        </button>
      </div>
      <main className="mobile-overview-content">
        <Link className="mobile-large-option" to="/collections/1">
          <span aria-hidden="true">▣</span>
          All collections
        </Link>
        <label className="mobile-search-field">
          <span aria-hidden="true">⌕</span>
          <input
            type="search"
            placeholder="Search binders"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            aria-label="Search binders"
          />
        </label>
        <div className="mobile-binder-list">
          {filteredCollections.map((collection, index) => {
            const stats = collectionStats[collection.id];
            const children = childrenByParent[collection.id] ?? [];
            return (
              <Link
                className="mobile-binder-card"
                to={"/c/" + encodeURIComponent(collection.id) + "/1"}
                key={collection.id}
                style={{ "--binder-accent": ACCENT_COLORS[index % ACCENT_COLORS.length] }}
              >
                <span className="mobile-binder-copy">
                  <strong>{collection.id}</strong>
                  {children.length > 0 ? (
                    <span>{children.length} sub-collection{children.length !== 1 ? "s" : ""}</span>
                  ) : (
                    <span>{stats?.copyCount ?? 0} cards</span>
                  )}
                </span>
                <span className="mobile-binder-value">
                  <strong>{formatMobileCents((stats?.totalValueCents ?? 0) - (stats?.proxyTotalValueCents ?? 0))}</strong>
                  <span className={trendClass(stats)}>{changeText(stats)}</span>
                </span>
              </Link>
            );
          })}
        </div>
      </main>
      <div className="mobile-overview-fab">
        <button type="button" onClick={() => setNewCollectionOpen(true)} aria-label="New collection">
          +
        </button>
      </div>
      {newCollectionOpen && (
        <MobileNewCollectionSheet onClose={() => setNewCollectionOpen(false)} />
      )}
    </>
  );
}

function MobileCollectionDetail() {
  const collection = useCollection();
  const allCollections = useCollections();
  const pageNumber = usePageNumber();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const [filtersOpen, setFiltersOpen] = useState(false);
  const { openQuickSearch } = useQuickSearch();
  const searchValue = searchParams.get("cf_name") ?? "";
  const cards = useCards();

  const chipData = useMemo(() => {
    const currentColl = allCollections.find((c) => c.id === collection);
    const parentId = currentColl?.parent ?? null;
    const children = allCollections.filter((c) => c.parent === collection);

    if (children.length > 0) {
      // This is a parent: show All (active) + each child
      return {
        allLink: `/c/${encodeURIComponent(collection)}/1`,
        allActive: true,
        parentLink: null,
        chips: children.map((c) => ({ id: c.id, active: false })),
      };
    } else if (parentId) {
      // This is a child: show All (→ parent) + siblings
      const siblings = allCollections.filter((c) => c.parent === parentId);
      return {
        allLink: `/c/${encodeURIComponent(parentId)}/1`,
        allActive: false,
        parentLink: parentId,
        chips: siblings.map((c) => ({ id: c.id, active: c.id === collection })),
      };
    }
    return null;
  }, [allCollections, collection]);

  // B2: Bottom sheet state
  const [sheetCardIndex, setSheetCardIndex] = useState(null);

  const setSearchValue = (value) => {
    const next = new URLSearchParams(searchParams);
    if (value.trim() === "") next.delete("cf_name");
    else next.set("cf_name", value);
    setSearchParams(next);
    if (String(pageNumber) !== "1") {
      navigate(`/c/${encodeURIComponent(collection)}/1?${next.toString()}`, { replace: true });
    }
  };

  // Capture clicks on card images and open bottom sheet instead of navigating
  const handleGridClick = useCallback((e) => {
    const link = e.target.closest(".search-card-image-link, .search-card-art");
    if (!link) return;
    const cardEl = link.closest(".search-card");
    if (!cardEl) return;

    // Find index by matching DOM order within the grid against the cards array
    const gridEl = e.currentTarget;
    const allCardEls = Array.from(gridEl.querySelectorAll(".search-card"));
    const domIndex = allCardEls.indexOf(cardEl);
    if (domIndex < 0 || domIndex >= cards.length) return;

    e.preventDefault();
    e.stopPropagation();
    setSheetCardIndex(domIndex);
  }, [cards]);

  const backTo = chipData?.parentLink
    ? `/c/${encodeURIComponent(chipData.parentLink)}/1`
    : "/collections/1";

  return (
    <>
      <MobileTopBar title={collectionDisplayName(collection)} backTo={backTo} />
      {chipData && (
        <div className="mobile-child-filter-chips" role="tablist" aria-label="Filter by sub-collection">
          <Link
            to={chipData.allLink}
            className={"mobile-chip" + (chipData.allActive ? " active" : "")}
            replace
          >
            All
          </Link>
          {chipData.chips.map((chip) => (
            <Link
              key={chip.id}
              to={`/c/${encodeURIComponent(chip.id)}/1`}
              className={"mobile-chip" + (chip.active ? " active" : "")}
              replace
            >
              {chip.id}
            </Link>
          ))}
        </div>
      )}
      <main className="mobile-detail-content">
        <div className="mobile-detail-search-row">
          <label className="mobile-search-field mobile-card-search">
            <span aria-hidden="true">⌕</span>
            <input
              type="search"
              placeholder="Search"
              value={searchValue}
              onChange={(event) => setSearchValue(event.target.value)}
              aria-label={`Search ${collectionDisplayName(collection)}`}
            />
          </label>
          <button
            type="button"
            className="mobile-square-button"
            onClick={() => setFiltersOpen((open) => !open)}
            aria-expanded={filtersOpen}
            aria-label="Filters and sort"
          >
            ≡
          </button>
        </div>
        {filtersOpen && (
          <div className="mobile-filter-sheet">
            <CollectionFilterBar />
          </div>
        )}
        {/* Capture phase click handler to intercept card image link taps */}
        <div
          className="mobile-card-grid"
          onClickCapture={handleGridClick}
        >
          <CardList />
        </div>
      </main>
      <div className="mobile-floating-actions" aria-label="Collection actions">
        <button type="button" onClick={() => setFiltersOpen((open) => !open)} aria-label="Filters">
          ≡
        </button>
        <button type="button" onClick={openQuickSearch} aria-label="Quick add">
          +
        </button>
      </div>

      {/* B2: Bottom sheet */}
      {sheetCardIndex !== null && cards.length > 0 && (
        <MobileCardSheet
          cards={cards}
          initialIndex={sheetCardIndex}
          onClose={() => setSheetCardIndex(null)}
        />
      )}
    </>
  );
}

export default function MobileCollectionView() {
  const collection = useCollection();

  return (
    <div className="mobile-collection-app">
      {isAllCollections(collection) ? <MobileCollectionOverview /> : <MobileCollectionDetail />}
      <MobileBottomNav />
    </div>
  );
}
