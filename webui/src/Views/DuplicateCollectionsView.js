import React, { useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import ViewProviders from "./ViewProviders";
import {
  ALL_COLLECTIONS_ID,
  collectionDisplayName,
  useCollections,
} from "../Components/CollectionContext";
import { useOperations } from "../OperationsContext";
import { useCardLoader } from "../Components/CardListContexts/CardLoaderContext";

const DUPLICATE_PAGE_SIZE = 1000;

function cardGroupKey(card) {
  return `${card.provider || "unknown"}:${card.id}`;
}

export function totalCopies(cards) {
  return cards.reduce(
    (total, card) => total + Number(card.quantity || 0) + Number(card.foilQuantity || 0),
    0,
  );
}

export function groupDuplicateCards(cards) {
  const groupsByKey = new Map();

  cards.forEach((card) => {
    const key = cardGroupKey(card);
    const group = groupsByKey.get(key) ?? {
      key,
      id: card.id,
      provider: card.provider,
      cards: [],
      collectionIds: new Set(),
    };
    group.cards.push(card);
    group.collectionIds.add(card.collectionId);
    groupsByKey.set(key, group);
  });

  return Array.from(groupsByKey.values())
    .filter((group) => group.collectionIds.size > 1)
    .map((group) => ({
      ...group,
      collectionIds: Array.from(group.collectionIds).sort((a, b) => a.localeCompare(b)),
      totalCopies: totalCopies(group.cards),
    }))
    .sort((a, b) => {
      const nameA = a.collectionIds.join(" ");
      const nameB = b.collectionIds.join(" ");
      return nameA.localeCompare(nameB) || a.id.localeCompare(b.id);
    });
}

function providerLabel(provider) {
  if (provider === "RiftboundSQLite") return "Riftbound";
  if (provider === "PokemonSQLite") return "Pokemon";
  if (provider === "MagicSQLite") return "Magic";
  return provider?.replace(/SQLite$/, "") ?? "Unknown";
}

function detailPathFor(provider, id) {
  if (provider === "RiftboundSQLite") return `/card/riftbound/${encodeURIComponent(id)}`;
  if (provider === "PokemonSQLite") return `/card/pokemon/${encodeURIComponent(id)}`;
  return `/card/mtg/${encodeURIComponent(id)}`;
}

function DuplicateCardName({ group }) {
  const loadCard = useCardLoader();
  const [card, setCard] = useState(null);

  useEffect(() => {
    let cancelled = false;
    setCard(null);
    loadCard(group.id, group.provider)
      .then((loadedCard) => {
        if (!cancelled) setCard(loadedCard);
      })
      .catch(() => {
        if (!cancelled) setCard(null);
      });
    return () => {
      cancelled = true;
    };
  }, [group.id, group.provider, loadCard]);

  const label = card?.name ?? group.id;
  return (
    <Link to={detailPathFor(group.provider, group.id)} className="duplicate-card-name">
      {label}
    </Link>
  );
}

function DuplicateGroup({ group, movingKey, onMove }) {
  const isMoving = movingKey === group.key;

  return (
    <article className="duplicate-card-row">
      <div className="duplicate-card-summary">
        <div>
          <DuplicateCardName group={group} />
          <div className="duplicate-card-meta">
            <span>{providerLabel(group.provider)}</span>
            <span>{group.totalCopies} copies</span>
            <span>{group.collectionIds.length} collections</span>
          </div>
        </div>
        <div className="duplicate-card-actions" aria-label="Consolidate duplicate card">
          {group.collectionIds.map((collectionId) => (
            <button
              key={`${group.key}-${collectionId}`}
              type="button"
              className="btn btn-outline-info btn-sm"
              disabled={isMoving}
              onClick={() => onMove(group, collectionId)}
            >
              {isMoving ? "Moving..." : `Move all to ${collectionDisplayName(collectionId)}`}
            </button>
          ))}
        </div>
      </div>
      <div className="duplicate-location-list">
        {group.cards.map((card) => (
          <div
            key={`${card.collectionId}-${card.id}-${card.timeAdded}`}
            className="duplicate-location-item"
          >
            <span className="collection-pill" title={card.collectionId}>
              {card.collectionId}
            </span>
            <span>Regular x{card.quantity}</span>
            <span>Foil x{card.foilQuantity}</span>
            {card.isProxy && <span className="proxy-pill">Proxy</span>}
          </div>
        ))}
      </div>
    </article>
  );
}

function DuplicateCollectionsPage() {
  const { fetch: opsFetch, quietFetch } = useOperations();
  const collections = useCollections();
  const [duplicates, setDuplicates] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [movingKey, setMovingKey] = useState("");
  const [status, setStatus] = useState("");

  const collectionCount = collections.length;

  const loadDuplicates = useCallback(async () => {
    setLoading(true);
    setError("");
    setStatus("");

    try {
      const count = await opsFetch(
        "Counting cards across collections",
        0,
        `/collection/cards/${encodeURIComponent(ALL_COLLECTIONS_ID)}/count`,
      );
      const pages = Math.ceil(Number(count) / DUPLICATE_PAGE_SIZE);
      const batches = await Promise.all(
        Array.from({ length: pages }, (_, index) => {
          const params = new URLSearchParams({
            offset: String(index * DUPLICATE_PAGE_SIZE),
            limit: String(DUPLICATE_PAGE_SIZE),
          });
          return quietFetch(
            [],
            `/collection/cards/${encodeURIComponent(ALL_COLLECTIONS_ID)}/list?${params.toString()}`,
          );
        }),
      );

      setDuplicates(groupDuplicateCards(batches.flat()));
    } catch (e) {
      setError(e.message || "Failed to find duplicate cards.");
    } finally {
      setLoading(false);
    }
  }, [opsFetch, quietFetch]);

  useEffect(() => {
    loadDuplicates();
  }, [loadDuplicates, collectionCount]);

  const moveToCollection = useCallback(
    async (group, targetCollectionId) => {
      const cardsToMove = group.cards.filter((card) => card.collectionId !== targetCollectionId);
      if (cardsToMove.length === 0) return;

      setMovingKey(group.key);
      setError("");
      setStatus("");
      try {
        await opsFetch(
          `Moving duplicates to ${targetCollectionId}`,
          [],
          `/collection/move/${encodeURIComponent(targetCollectionId)}`,
          {
            method: "post",
            headers: {
              Accept: "application/json",
              "Content-Type": "application/json",
            },
            body: JSON.stringify(cardsToMove),
          },
        );
        setStatus(`Moved ${group.totalCopies} copies to ${targetCollectionId}.`);
        await loadDuplicates();
      } catch (e) {
        setError(e.message || "Failed to move duplicate cards.");
      } finally {
        setMovingKey("");
      }
    },
    [loadDuplicates, opsFetch],
  );

  const summary = useMemo(() => {
    const copyCount = duplicates.reduce((total, group) => total + group.totalCopies, 0);
    return { groups: duplicates.length, copyCount };
  }, [duplicates]);

  return (
    <section className="collection-results-panel duplicate-page">
      <div className="duplicate-page-header">
        <div>
          <h1>Duplicate Cards</h1>
          <p>
            Find cards split across collections and consolidate each duplicate into one location.
          </p>
        </div>
        <button
          type="button"
          className="btn btn-outline-light"
          onClick={loadDuplicates}
          disabled={loading || Boolean(movingKey)}
        >
          Refresh
        </button>
      </div>

      <div className="duplicate-summary-bar" aria-live="polite">
        <span>{loading ? "Scanning collections..." : `${summary.groups} duplicate cards`}</span>
        {!loading && <span>{summary.copyCount} copies involved</span>}
        {status && <span className="duplicate-status-success">{status}</span>}
        {error && <span className="duplicate-status-error">{error}</span>}
      </div>

      {!loading && duplicates.length === 0 && !error && (
        <div className="duplicate-empty-state">
          No cross-collection duplicates found.
        </div>
      )}

      <div className="duplicate-list">
        {duplicates.map((group) => (
          <DuplicateGroup
            key={group.key}
            group={group}
            movingKey={movingKey}
            onMove={moveToCollection}
          />
        ))}
      </div>
    </section>
  );
}

export default function DuplicateCollectionsView() {
  return (
    <ViewProviders>
      <DuplicateCollectionsPage />
    </ViewProviders>
  );
}
