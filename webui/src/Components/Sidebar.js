import React, { useState, useEffect } from "react";
import { Link, useLocation } from "react-router-dom";
import AddCollectionForm from "./AddCollectionForm";
import {
  ALL_COLLECTIONS_ID,
  useCollection,
  useCollections,
} from "./CollectionContext";
import OperationsTracker from "./CardListNavButtons/OperationsTracker";
import { useMode } from "../OperationsContext";
import { useQuickSearch } from "./QuickSearchContext";
import SettingsModal from "./SettingsModal";
import CardListNav from "./CardListNav";
import CollectionFilterBar from "./CollectionFilterBar";

function useServerStatus() {
  const [status, setStatus] = useState({ ready: true, downloading: {} });

  useEffect(() => {
    let timeout;
    const poll = () => {
      fetch("/system")
        .then((r) => (r.ok ? r.json() : null))
        .then((data) => {
          if (!data) {
            setStatus({ ready: false, downloading: {} });
            timeout = setTimeout(poll, 3000);
          } else if (data.downloading && Object.keys(data.downloading).length > 0) {
            setStatus({ ready: false, downloading: data.downloading });
            timeout = setTimeout(poll, 1000);
          } else {
            setStatus({ ready: true, downloading: {} });
          }
        })
        .catch(() => {
          setStatus({ ready: false, downloading: {} });
          timeout = setTimeout(poll, 3000);
        });
    };
    poll();
    return () => clearTimeout(timeout);
  }, []);

  return status;
}

export function getSidebarBrandPath({ isSearchOnly = false, collectionsEnabled = false } = {}) {
  return !isSearchOnly && collectionsEnabled ? "/collections/1" : "/search";
}

export default function Sidebar() {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [collectionsOpen, setCollectionsOpen] = useState(true);
  const [collectionQuery, setCollectionQuery] = useState("");
  const collection = useCollection();
  const collections = useCollections();
  const { mode, collectionsEnabled } = useMode();
  const isSearchOnly = mode === "search-only";
  const serverStatus = useServerStatus();
  const { openQuickSearch } = useQuickSearch();
  const location = useLocation();
  const showCollectionTools =
    !isSearchOnly &&
    collectionsEnabled &&
    (location.pathname.startsWith("/c/") || location.pathname.startsWith("/collections"));
  const filteredCollections = collections.filter((c) =>
    c.id.toLowerCase().includes(collectionQuery.trim().toLowerCase()),
  );
  const allCollectionsActive = collection === ALL_COLLECTIONS_ID;
  const brandPath = getSidebarBrandPath({ isSearchOnly, collectionsEnabled });

  return (
    <header>
      <nav id="sidebarMenu" className="d-lg-block sidebar bg-white">
        <nav className="navbar navbar-expand-lg navbar-light bg-light">
          <div className="container-fluid">
            <Link className="navbar-brand" to={brandPath}>
              GatheRs
            </Link>
          </div>
        </nav>
        <div className="position-sticky sidebar-content">
          <div
            className="nav flex-column nav-pills me-3 sidebar-top-actions"
            role="tablist"
            aria-orientation="vertical"
          >
            <Link to={"/search"} className="btn btn-secondary">
              Search
            </Link>
            {!isSearchOnly && collectionsEnabled && (
              <button type="button" className="btn btn-primary" onClick={openQuickSearch}>
                Quick Add
              </button>
            )}
          </div>
          {!serverStatus.ready && (
            <>
              <hr />
              <div className="px-3 py-2 text-muted small">
                {Object.keys(serverStatus.downloading).length === 0 ? (
                  <div className="d-flex align-items-center gap-2">
                    <div className="spinner-border spinner-border-sm" role="status" aria-hidden="true" />
                    Server starting up…
                  </div>
                ) : (
                  Object.entries(serverStatus.downloading).map(([system, p]) => {
                    const pct = p.total > 0 ? Math.round((p.downloaded / p.total) * 100) : null;
                    const label = p.phase === "checking"
                      ? `Checking ${system}…`
                      : p.phase === "verifying"
                      ? `Verifying ${system}…`
                      : pct !== null
                      ? `Downloading ${system}: ${pct}%`
                      : `Downloading ${system}…`;
                    return (
                      <div key={system} className="mb-1">
                        <div className="d-flex align-items-center gap-2 mb-1">
                          <div className="spinner-border spinner-border-sm" role="status" aria-hidden="true" />
                          {label}
                        </div>
                        {p.phase === "downloading" && pct !== null && (
                          <div className="progress" style={{ height: "4px" }}>
                            <div
                              className="progress-bar"
                              role="progressbar"
                              style={{ width: `${pct}%` }}
                              aria-valuenow={pct}
                              aria-valuemin={0}
                              aria-valuemax={100}
                            />
                          </div>
                        )}
                      </div>
                    );
                  })
                )}
              </div>
            </>
          )}
          <hr />
          <div
            className="nav flex-column nav-pills me-3 sidebar-nav-stack sidebar-main-nav"
            aria-label="Collection sidebar"
          >
            {showCollectionTools && <CollectionFilterBar />}
            {!isSearchOnly && collectionsEnabled && (
              <section className="collection-panel-section sidebar-collections-panel">
                <button
                  type="button"
                  className="collection-panel-toggle"
                  aria-expanded={collectionsOpen}
                  onClick={() => setCollectionsOpen((open) => !open)}
                >
                  <span>Collections</span>
                  <span aria-hidden="true">{collectionsOpen ? "^" : "v"}</span>
                </button>
                {collectionsOpen && (
                  <div className="collection-panel-dropdown">
                    <input
                      type="search"
                      className="form-control form-control-sm"
                      placeholder="Search collections"
                      aria-label="Search collections"
                      value={collectionQuery}
                      onChange={(event) => setCollectionQuery(event.target.value)}
                    />
                    <div className="sidebar-collection-list">
                      <Link
                        to="/collections/1"
                        className={"nav-link sidebar-collection-link" + (allCollectionsActive ? " active" : "")}
                      >
                        <span>All Collections</span>
                      </Link>
                      {filteredCollections.length > 0 ? (
                        filteredCollections.map((c) => (
                          <Link
                            to={"/c/" + encodeURIComponent(c.id) + "/1"}
                            key={c.id}
                            className={"nav-link sidebar-collection-link" + (c.id === collection ? " active" : "")}
                          >
                            <span>{c.id}</span>
                            {c.isProxy && <span className="proxy-pill">Proxy</span>}
                          </Link>
                        ))
                      ) : (
                        <div className="sidebar-empty-state">No collections found</div>
                      )}
                    </div>
                    <AddCollectionForm />
                  </div>
                )}
              </section>
            )}
            {showCollectionTools && (
              <div className="sidebar-collection-tools">
                <CardListNav />
              </div>
            )}
            <OperationsTracker />
          </div>
          <div className="sidebar-settings">
            <hr />
            <div className="nav flex-column nav-pills me-3">
              <button type="button" className="btn btn-outline-secondary" onClick={() => setSettingsOpen(true)}>
                Settings
              </button>
            </div>
          </div>
        </div>
      </nav>
      <SettingsModal open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </header>
  );
}
