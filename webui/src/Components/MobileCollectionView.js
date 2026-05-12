import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import CardList from "./CardList";
import CardDetails from "./CardDetails";
import CollectionFilterBar from "./CollectionFilterBar";
import {
  collectionDisplayName,
  isAllCollections,
  useCollection,
  useCollections,
  usePageNumber,
} from "./CollectionContext";
import { useOperations } from "../OperationsContext";
import { useQuickSearch } from "./QuickSearchContext";
import { formatCents, formatPercent, priceTrend, unitPriceCents } from "./priceUtils";
import { useCards } from "./CardListContexts/CardsContext";
import { useCardLoader } from "./CardListContexts/CardLoaderContext";

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

function MobileBottomNav() {
  return (
    <nav className="mobile-bottom-nav" aria-label="Primary">
      <Link to="/collections/1" className="mobile-bottom-nav-item">
        <span aria-hidden="true">⌂</span>
        Home
      </Link>
      <Link to="/search" className="mobile-bottom-nav-item">
        <span aria-hidden="true">⌕</span>
        Search
      </Link>
      <Link to="/collections/1" className="mobile-bottom-nav-item active">
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
    </section>
  );
}

// B2: Card bottom sheet component
function MobileCardSheet({ cards, initialIndex, onClose }) {
  const [activeIndex, setActiveIndex] = useState(initialIndex);
  const carouselRef = useRef(null);
  const touchStartY = useRef(null);
  const scrollTimeout = useRef(null);

  // Bug 1: Load card metadata for the active card using useCardLoader
  const loader = useCardLoader();
  const [activeCardData, setActiveCardData] = useState(null);
  const activeDetails = cards[activeIndex]; // flat details object IS the item

  useEffect(() => {
    if (!activeDetails || !loader) return;
    let cancelled = false;
    setActiveCardData(null);
    loader(activeDetails.id, activeDetails.provider).then((card) => {
      if (!cancelled) setActiveCardData(card);
    }).catch(() => {});
    return () => { cancelled = true; };
  }, [activeIndex, activeDetails, loader]);

  // Bug 5: Scroll carousel to the initial card on mount without smooth scroll
  useEffect(() => {
    const carousel = carouselRef.current;
    if (!carousel) return;
    const itemWidth = carousel.clientWidth;
    carousel.style.scrollBehavior = "none";
    carousel.scrollLeft = initialIndex * itemWidth;
    carousel.style.scrollBehavior = "";
  }, [initialIndex]);

  // Bug 2: Update activeIndex as user swipes carousel — use functional updater to avoid stale closure
  const handleCarouselScroll = useCallback(() => {
    if (scrollTimeout.current) clearTimeout(scrollTimeout.current);
    scrollTimeout.current = setTimeout(() => {
      const carousel = carouselRef.current;
      if (!carousel) return;
      const itemWidth = carousel.clientWidth;
      const newIndex = Math.round(carousel.scrollLeft / itemWidth);
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

  // Bug 3: Swipe-to-dismiss touch handlers — bail if touch starts inside detail panel
  function handleTouchStart(e) {
    if (e.target.closest(".mobile-sheet-detail")) return;
    touchStartY.current = e.touches[0].clientY;
  }

  function handleTouchEnd(e) {
    if (touchStartY.current == null) return;
    const deltaY = e.changedTouches[0].clientY - touchStartY.current;
    touchStartY.current = null;
    if (deltaY > 80) {
      onClose();
    }
  }

  // Bug 1: Price / trend info now comes from loaded card data and flat details
  const trend = activeCardData?.price != null
    ? priceTrend(activeCardData.price, activeDetails)
    : null;
  const unitPrice = activeCardData?.price != null
    ? unitPriceCents(activeCardData.price, activeDetails)
    : null;

  const qty = (activeDetails?.quantity ?? 0) + (activeDetails?.foilQuantity ?? 0);

  return (
    <>
      {/* Backdrop */}
      <div
        className="mobile-card-sheet-backdrop"
        onClick={onClose}
        aria-label="Close card detail"
        role="button"
        tabIndex={-1}
      />

      {/* Sheet panel */}
      <div
        className="mobile-card-sheet"
        onTouchStart={handleTouchStart}
        onTouchEnd={handleTouchEnd}
        role="dialog"
        aria-label="Card detail"
        aria-modal="true"
      >
        {/* Swipe handle */}
        <div className="mobile-sheet-handle" aria-hidden="true" />

        {/* Carousel — show loaded image for active slide, placeholder for others */}
        <div
          ref={carouselRef}
          className="mobile-sheet-carousel"
          onScroll={handleCarouselScroll}
        >
          {cards.map((card, index) => {
            const isActive = index === activeIndex;
            const imgSrc = isActive ? getCardImagePath(activeCardData) : "";
            return (
              <div key={card.id ?? index} className="mobile-sheet-carousel-slide">
                {imgSrc ? (
                  <img
                    src={imgSrc}
                    alt={activeCardData?.name ?? "Card"}
                    loading="eager"
                  />
                ) : (
                  <div className="mobile-sheet-carousel-placeholder" aria-hidden="true" />
                )}
              </div>
            );
          })}
        </div>

        {/* Detail panel */}
        <div className="mobile-sheet-detail">
          <div className="mobile-sheet-detail-name">
            <span className="mobile-sheet-qty">×{qty}</span>
            <strong>{activeCardData?.name ?? activeDetails?.id}</strong>
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
              {trend && trend.direction !== "flat" && (
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

          {/* Action row */}
          <div className="mobile-sheet-actions">
            {activeDetails && (
              <CardDetails
                id={activeDetails.id}
                details={activeDetails}
                showCollectionSelect={false}
                targetCollection={activeDetails?.collectionId ?? null}
              />
            )}
          </div>
        </div>
      </div>
    </>
  );
}

function MobileCollectionOverview() {
  const { fetch: opsFetch, quietFetch } = useOperations();
  const collections = useCollections();
  const [query, setQuery] = useState("");
  const [allStats, setAllStats] = useState(null);
  const [collectionStats, setCollectionStats] = useState({});
  const filteredCollections = useMemo(
    () => collections.filter((c) => c.id.toLowerCase().includes(query.trim().toLowerCase())),
    [collections, query],
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
          All collection
        </Link>
        <button type="button" className="mobile-large-option">
          <span aria-hidden="true">◩</span>
          Decks
        </button>
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
            return (
              <Link
                className="mobile-binder-card"
                to={"/c/" + encodeURIComponent(collection.id) + "/1"}
                key={collection.id}
                style={{ "--binder-accent": ACCENT_COLORS[index % ACCENT_COLORS.length] }}
              >
                <span className="mobile-binder-copy">
                  <strong>{collection.id}</strong>
                  <span>{stats?.copyCount ?? 0} cards</span>
                </span>
                <span className="mobile-binder-value">
                  <strong>{formatMobileCents(stats?.totalValueCents)}</strong>
                  <span className={trendClass(stats)}>{changeText(stats)}</span>
                </span>
              </Link>
            );
          })}
        </div>
      </main>
    </>
  );
}

function MobileCollectionDetail() {
  const collection = useCollection();
  const pageNumber = usePageNumber();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const [filtersOpen, setFiltersOpen] = useState(false);
  const { openQuickSearch } = useQuickSearch();
  const searchValue = searchParams.get("cf_name") ?? "";
  const cards = useCards();

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

  return (
    <>
      <MobileTopBar title={collectionDisplayName(collection)} backTo="/collections/1" />
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
