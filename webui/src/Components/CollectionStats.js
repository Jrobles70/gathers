import React, { useEffect, useState } from "react";
import { useOperations } from "../OperationsContext";
import { useCollection } from "./CollectionContext";
import { useRefresh } from "./CardListContexts/RefreshCardListContext";
import { useCards } from "./CardListContexts/CardsContext";
import { formatCents, formatPercent } from "./priceUtils";

function statTone(stats) {
  if ((stats?.changeCents ?? 0) > 0) return "price-up";
  if ((stats?.changeCents ?? 0) < 0) return "price-down";
  return "";
}

function changeText(stats) {
  if (stats?.changeCents == null || stats?.changePercent == null) return "No baseline";
  const sign = stats.changeCents > 0 ? "+" : "";
  return `${sign}${formatCents(stats.changeCents)} (${formatPercent(stats.changePercent)})`;
}

function StatsBlock({ title, stats }) {
  return (
    <div className="collection-stats-block">
      <div className="collection-stats-title">{title}</div>
      <div className="collection-stats-value">{formatCents(stats?.totalValueCents)}</div>
      <div className={"collection-stats-change " + statTone(stats)}>
        {changeText(stats)}
      </div>
      <div className="collection-stats-meta">
        {stats?.copyCount ?? 0} copies - {stats?.pricedCopyCount ?? 0} priced
      </div>
    </div>
  );
}

export default function CollectionStats() {
  const ops = useOperations();
  const collection = useCollection();
  const refresh = useRefresh();
  const cards = useCards();
  const [statsOpen, setStatsOpen] = useState(false);
  const [currentStats, setCurrentStats] = useState(null);
  const [allStats, setAllStats] = useState(null);

  useEffect(() => {
    if (!collection || !statsOpen) return;
    ops
      .fetch("Getting collection value", null, `/collection/cards/${encodeURIComponent(collection)}/stats`)
      .then(setCurrentStats);
    ops
      .fetch("Getting all collection value", null, "/collection/stats")
      .then(setAllStats);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [collection, refresh, cards, statsOpen]);

  return (
    <section className="collection-panel-section collection-stats-panel">
      <button
        type="button"
        className="collection-panel-toggle"
        aria-expanded={statsOpen}
        onClick={() => setStatsOpen((open) => !open)}
      >
        <span>Stats</span>
        <span aria-hidden="true">{statsOpen ? "^" : "v"}</span>
      </button>
      {statsOpen && (
        <div className="collection-panel-dropdown collection-stats-dropdown">
          <StatsBlock title={collection} stats={currentStats} />
          <StatsBlock title="All collections" stats={allStats} />
        </div>
      )}
    </section>
  );
}
