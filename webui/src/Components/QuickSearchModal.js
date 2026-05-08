import React, { useEffect, useMemo, useState } from "react";
import Search from "./Search";
import { useCollection, useCollections } from "./CollectionContext";
import { useQuickSearch } from "./QuickSearchContext";

export default function QuickSearchModal() {
  const currentCollection = useCollection();
  const collections = useCollections();
  const { quickSearchOpen, closeQuickSearch } = useQuickSearch();

  const collectionOptions = useMemo(
    () => (collections.length > 0 ? collections : [{ id: currentCollection }]),
    [collections, currentCollection],
  );
  const preferredCollection = collectionOptions.some((c) => c.id === currentCollection)
    ? currentCollection
    : collectionOptions[0]?.id ?? currentCollection;
  const [targetCollection, setTargetCollection] = useState(preferredCollection);

  useEffect(() => {
    if (quickSearchOpen) {
      setTargetCollection(preferredCollection);
    }
  }, [quickSearchOpen, preferredCollection]);

  useEffect(() => {
    if (!quickSearchOpen) return undefined;
    const handleKeyDown = (event) => {
      if (event.key === "Escape") closeQuickSearch();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [quickSearchOpen, closeQuickSearch]);

  if (!quickSearchOpen) return null;

  return (
    <div className="quick-search-backdrop" onMouseDown={closeQuickSearch}>
      <section
        className="quick-search-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="quick-search-title"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="quick-search-header">
          <div className="quick-search-heading">
            <h2 id="quick-search-title">Quick Search</h2>
            <div className="quick-search-target-summary">
              Adding cards to <strong>{targetCollection}</strong>
            </div>
          </div>
          <button
            type="button"
            className="btn btn-outline-secondary"
            onClick={closeQuickSearch}
            aria-label="Close quick search"
          >
            Close
          </button>
        </header>
        <label className="quick-search-target-control">
          <span>Collection</span>
          <select
            className="form-select"
            value={targetCollection}
            onChange={(event) => setTargetCollection(event.target.value)}
          >
            {collectionOptions.map((c) => (
              <option key={c.id} value={c.id}>
                {c.id}
              </option>
            ))}
          </select>
        </label>
        <div className="quick-search-body">
          <Search sidePanel={true} showTitle={false} targetCollection={targetCollection} />
        </div>
      </section>
    </div>
  );
}
