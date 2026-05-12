import React, { useState, useEffect, useCallback, useRef } from "react";
import { createPortal } from "react-dom";
import { Link, useLocation, useNavigate } from "react-router-dom";
import Modal from "react-bootstrap/Modal";
import Button from "react-bootstrap/Button";
import AddCollectionForm from "./AddCollectionForm";
import {
  ALL_COLLECTIONS_ID,
  useCollection,
  useCollections,
  useCollectionsDispatch,
} from "./CollectionContext";
import OperationsTracker from "./CardListNavButtons/OperationsTracker";
import { useMode, useOperations } from "../OperationsContext";
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


function CollectionDotMenu({ collectionId, collectionType, onRename, onDelete, onMoveParent, onRemoveFromParent }) {
  const [open, setOpen] = useState(false);
  const [pos, setPos] = useState({ top: 0, right: 0 });
  const btnRef = useRef();
  const closeTimer = useRef(null);

  const handleMouseEnter = () => {
    clearTimeout(closeTimer.current);
    if (btnRef.current) {
      const rect = btnRef.current.getBoundingClientRect();
      setPos({ top: rect.bottom, right: window.innerWidth - rect.right });
    }
    setOpen(true);
  };

  const handleMouseLeave = () => {
    closeTimer.current = setTimeout(() => setOpen(false), 80);
  };

  return (
    <div
      className="collection-dot-menu"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      <button
        ref={btnRef}
        type="button"
        className="collection-dot-menu-btn"
        aria-label={"Options for " + collectionId}
      >
        ···
      </button>
      {open && createPortal(
        <div
          className="collection-dot-menu-dropdown"
          style={{ position: "fixed", top: pos.top, right: pos.right, zIndex: 9999 }}
          onMouseEnter={() => clearTimeout(closeTimer.current)}
          onMouseLeave={handleMouseLeave}
        >
          <button onClick={() => { setOpen(false); onRename(); }}>Rename</button>
          <button onClick={() => { setOpen(false); onDelete(); }}>Delete</button>
          {collectionType === "leaf" && (
            <button onClick={() => { setOpen(false); onMoveParent(); }}>Move into a parent</button>
          )}
          {collectionType === "child" && (
            <button onClick={() => { setOpen(false); onMoveParent(); }}>Move to another parent</button>
          )}
          {collectionType === "child" && (
            <button onClick={() => { setOpen(false); onRemoveFromParent(); }}>Remove from parent</button>
          )}
        </div>,
        document.body
      )}
    </div>
  );
}

function InlineRenameForm({ collectionId, onDone, onCancel }) {
  const [name, setName] = useState(collectionId);
  const [error, setError] = useState(null);
  const ops = useOperations();
  const collectionsDispatch = useCollectionsDispatch();
  const navigate = useNavigate();
  const collection = useCollection();

  const handleSubmit = (e) => {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed || trimmed === collectionId) { onCancel(); return; }
    setError(null);
    ops.fetch("Renaming collection " + collectionId, {}, "/collection/rename/" + encodeURIComponent(collectionId), {
      method: "post",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: trimmed }),
    }).then((updated) => {
      collectionsDispatch({ type: "renamed", from: collectionId, item: updated });
      if (collection === collectionId) {
        navigate("/c/" + encodeURIComponent(updated.id) + "/1");
      }
      onDone();
    }).catch((err) => setError(err.message));
  };

  return (
    <form className="sidebar-inline-rename" onSubmit={handleSubmit}>
      <input
        autoFocus
        className={"form-control form-control-sm" + (error ? " is-invalid" : "")}
        value={name}
        onChange={(e) => setName(e.target.value)}
      />
      {error && <div className="invalid-feedback d-block">{error}</div>}
      <div className="sidebar-inline-actions">
        <button type="submit" className="btn btn-primary btn-sm" disabled={!name.trim() || name.trim() === collectionId}>Save</button>
        <button type="button" className="btn btn-secondary btn-sm" onClick={onCancel}>Cancel</button>
      </div>
    </form>
  );
}

function AddChildCollectionForm({ parentId, onDone, onCancel }) {
  const [name, setName] = useState("");
  const [error, setError] = useState(null);
  const ops = useOperations();
  const collectionsDispatch = useCollectionsDispatch();

  const handleSubmit = (e) => {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;
    setError(null);
    ops.fetch("Adding child collection", {}, "/collection/add", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: trimmed, parent: parentId }),
    }).then((added) => {
      collectionsDispatch({ type: "added", item: { id: trimmed, isProxy: false, canRemove: true, parent: parentId } });
      onDone();
    }).catch((err) => setError(err.message));
  };

  return (
    <form className="sidebar-inline-rename sidebar-add-child" onSubmit={handleSubmit}>
      <input
        autoFocus
        className={"form-control form-control-sm" + (error ? " is-invalid" : "")}
        placeholder="Child collection name"
        value={name}
        onChange={(e) => setName(e.target.value)}
      />
      {error && <div className="invalid-feedback d-block">{error}</div>}
      <div className="sidebar-inline-actions">
        <button type="submit" className="btn btn-primary btn-sm" disabled={!name.trim()}>Add</button>
        <button type="button" className="btn btn-secondary btn-sm" onClick={onCancel}>Cancel</button>
      </div>
    </form>
  );
}

function DeleteParentDialog({ collectionId, children, allCollections, onConfirm, onCancel }) {
  const [mode, setMode] = useState("top-level");
  const [targetParent, setTargetParent] = useState("");
  const availableParents = allCollections.filter(
    (c) => !c.parent && c.id !== collectionId && !children.some((ch) => ch.id === c.id)
  );

  return (
    <Modal show animation={false} onHide={onCancel}>
      <Modal.Header>
        <Modal.Title>Delete "{collectionId}"</Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <p>
          <strong>{collectionId}</strong> has {children.length} child collection{children.length !== 1 ? "s" : ""}. What should happen to them?
        </p>
        <div className="mb-2">
          <label className="d-flex align-items-center gap-2">
            <input
              type="radio"
              name="reparent"
              value="top-level"
              checked={mode === "top-level"}
              onChange={() => setMode("top-level")}
            />
            Leave as top-level collections
          </label>
        </div>
        {availableParents.length > 0 && (
          <div className="mb-2">
            <label className="d-flex align-items-center gap-2">
              <input
                type="radio"
                name="reparent"
                value="move"
                checked={mode === "move"}
                onChange={() => setMode("move")}
              />
              Move to another parent:
            </label>
            {mode === "move" && (
              <select
                className="form-select form-select-sm mt-1"
                value={targetParent}
                onChange={(e) => setTargetParent(e.target.value)}
              >
                <option value="">— select parent —</option>
                {availableParents.map((c) => (
                  <option key={c.id} value={c.id}>{c.id}</option>
                ))}
              </select>
            )}
          </div>
        )}
      </Modal.Body>
      <Modal.Footer>
        <Button variant="secondary" onClick={onCancel}>Cancel</Button>
        <Button
          variant="danger"
          disabled={mode === "move" && !targetParent}
          onClick={() => onConfirm(mode === "move" ? targetParent : null)}
        >
          Delete
        </Button>
      </Modal.Footer>
    </Modal>
  );
}

function MoveParentModal({ collectionId, currentParent, allCollections, onConfirm, onCancel }) {
  const parentOptions = allCollections.filter(
    (c) => !c.parent && c.id !== collectionId
  );
  const [selected, setSelected] = useState(currentParent || "");

  return (
    <Modal show animation={false} onHide={onCancel}>
      <Modal.Header>
        <Modal.Title>Move "{collectionId}" to a parent</Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <select
          className="form-select"
          value={selected}
          onChange={(e) => setSelected(e.target.value)}
        >
          <option value="">— no parent (top-level) —</option>
          {parentOptions.map((c) => (
            <option key={c.id} value={c.id}>{c.id}</option>
          ))}
        </select>
      </Modal.Body>
      <Modal.Footer>
        <Button variant="secondary" onClick={onCancel}>Cancel</Button>
        <Button variant="primary" disabled={selected === (currentParent || "")} onClick={() => onConfirm(selected || null)}>
          Move
        </Button>
      </Modal.Footer>
    </Modal>
  );
}

function useCollectionActions({ allCollections, collection }) {
  const ops = useOperations();
  const collectionsDispatch = useCollectionsDispatch();
  const navigate = useNavigate();
  const [deleteParentState, setDeleteParentState] = useState(null);
  const [moveParentState, setMoveParentState] = useState(null);

  const childrenOf = useCallback(
    (id) => allCollections.filter((c) => c.parent === id),
    [allCollections]
  );

  const handleDelete = useCallback((collectionId) => {
    const children = childrenOf(collectionId);
    if (children.length > 0) {
      setDeleteParentState({ collectionId, children });
      return;
    }
    // Use simple confirm for non-parent collections
    if (!window.confirm(`Delete collection "${collectionId}"?`)) return;
    ops.fetch("Deleting collection " + collectionId, {}, "/collection/remove/" + encodeURIComponent(collectionId), {
      method: "post",
    }).then(() => {
      collectionsDispatch({ type: "deleted", id: collectionId });
      if (collection === collectionId) navigate("/collections/1");
    }).catch((e) => alert("Delete failed: " + e.message));
  }, [allCollections, childrenOf, ops, collectionsDispatch, navigate, collection]);

  const confirmDeleteParent = useCallback((collectionId, reparentChildrenTo) => {
    setDeleteParentState(null);
    const url = "/collection/remove/" + encodeURIComponent(collectionId) +
      (reparentChildrenTo != null ? "?reparentChildrenTo=" + encodeURIComponent(reparentChildrenTo) : "");
    ops.fetch("Deleting collection " + collectionId, {}, url, {
      method: "post",
    }).then(() => {
      const childIds = childrenOf(collectionId).map((c) => c.id);
      collectionsDispatch({ type: "deleted", id: collectionId });
      childIds.forEach((id) => {
        collectionsDispatch({ type: "updated", item: { ...allCollections.find((c) => c.id === id), parent: reparentChildrenTo || null } });
      });
      if (collection === collectionId) navigate("/collections/1");
    }).catch((e) => alert("Delete failed: " + e.message));
  }, [ops, collectionsDispatch, navigate, collection, childrenOf, allCollections]);

  const handleMoveParent = useCallback((collectionId, currentParent) => {
    setMoveParentState({ collectionId, currentParent: currentParent || null });
  }, []);

  const confirmMoveParent = useCallback((collectionId, newParent) => {
    setMoveParentState(null);
    ops.fetch("Setting parent for " + collectionId, {}, "/collection/set-parent/" + encodeURIComponent(collectionId), {
      method: "post",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ parent: newParent }),
    }).then((updated) => {
      collectionsDispatch({ type: "updated", item: updated });
    }).catch((e) => alert("Move failed: " + e.message));
  }, [ops, collectionsDispatch]);

  const handleRemoveFromParent = useCallback((collectionId) => {
    ops.fetch("Removing " + collectionId + " from parent", {}, "/collection/set-parent/" + encodeURIComponent(collectionId), {
      method: "post",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ parent: null }),
    }).then((updated) => {
      collectionsDispatch({ type: "updated", item: updated });
    }).catch((e) => alert("Failed: " + e.message));
  }, [ops, collectionsDispatch]);

  return {
    handleDelete,
    confirmDeleteParent,
    handleMoveParent,
    confirmMoveParent,
    handleRemoveFromParent,
    deleteParentState,
    setDeleteParentState,
    moveParentState,
    setMoveParentState,
  };
}


export default function Sidebar() {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [collectionsOpen, setCollectionsOpen] = useState(true);
  const [collectionQuery, setCollectionQuery] = useState("");
  const [expandedParents, setExpandedParents] = useState(new Set());
  const [renamingId, setRenamingId] = useState(null);
  const [addingChildTo, setAddingChildTo] = useState(null);
  const collection = useCollection();
  const allCollections = useCollections();
  const { mode, collectionsEnabled } = useMode();
  const isSearchOnly = mode === "search-only";
  const serverStatus = useServerStatus();
  const location = useLocation();
  const showCollectionTools =
    !isSearchOnly &&
    collectionsEnabled &&
    (location.pathname.startsWith("/c/") || location.pathname.startsWith("/collections"));
  const hideSidebarOnMobile = showCollectionTools || location.pathname.startsWith("/search");
  const allCollectionsActive = collection === ALL_COLLECTIONS_ID;

  const {
    handleDelete,
    confirmDeleteParent,
    handleMoveParent,
    confirmMoveParent,
    handleRemoveFromParent,
    deleteParentState,
    setDeleteParentState,
    moveParentState,
    setMoveParentState,
  } = useCollectionActions({ allCollections, collection });

  // Build parent/child map
  const childrenByParent = {};
  const childIds = new Set();
  allCollections.forEach((c) => {
    if (c.parent) {
      childIds.add(c.id);
      if (!childrenByParent[c.parent]) childrenByParent[c.parent] = [];
      childrenByParent[c.parent].push(c);
    }
  });

  const query = collectionQuery.trim().toLowerCase();
  const collectionMatchesQuery = (c) => !query || c.id.toLowerCase().includes(query);
  const topLevel = allCollections.filter((c) => !c.parent);

  const toggleExpand = (parentId) => {
    setExpandedParents((prev) => {
      const next = new Set(prev);
      if (next.has(parentId)) next.delete(parentId);
      else next.add(parentId);
      return next;
    });
  };

  const isExpanded = (parentId) => {
    if (expandedParents.has(parentId)) return true;
    // Auto-expand if a child matches the query
    if (query && childrenByParent[parentId]?.some(collectionMatchesQuery)) return true;
    return false;
  };

  const visibleTopLevel = topLevel.filter((c) => {
    if (collectionMatchesQuery(c)) return true;
    // Show parent if any child matches
    if (childrenByParent[c.id]?.some(collectionMatchesQuery)) return true;
    return false;
  });

  return (
    <header>
      <nav
        id="sidebarMenu"
        className={"d-lg-block sidebar bg-white" + (hideSidebarOnMobile ? " mobile-collection-sidebar-hidden" : "")}
      >
        <div className="position-sticky sidebar-content">
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
                      {visibleTopLevel.length > 0 ? (
                        visibleTopLevel.map((c) => {
                          const children = childrenByParent[c.id] || [];
                          const hasChildren = children.length > 0;
                          const expanded = isExpanded(c.id);
                          const collectionType = hasChildren ? "parent" : "leaf";

                          return (
                            <div key={c.id} className="sidebar-collection-group">
                              <div className={"sidebar-collection-row sidebar-collection-row--" + collectionType}>
                                {hasChildren && (
                                  <button
                                    type="button"
                                    className="sidebar-chevron-btn"
                                    aria-label={expanded ? "Collapse " + c.id : "Expand " + c.id}
                                    onClick={() => toggleExpand(c.id)}
                                  >
                                    {expanded ? "▼" : "▶"}
                                  </button>
                                )}
                                {renamingId === c.id ? (
                                  <InlineRenameForm
                                    collectionId={c.id}
                                    onDone={() => setRenamingId(null)}
                                    onCancel={() => setRenamingId(null)}
                                  />
                                ) : (
                                  <>
                                    <Link
                                      to={"/c/" + encodeURIComponent(c.id) + "/1"}
                                      className={"nav-link sidebar-collection-link" + (c.id === collection ? " active" : "")}
                                      onClick={hasChildren ? () => { if (!expanded) toggleExpand(c.id); } : undefined}
                                    >
                                      <span>{c.id}</span>
                                      {c.isProxy && <span className="proxy-pill">Proxy</span>}
                                    </Link>
                                    <div className="sidebar-collection-row-actions">
                                      {hasChildren && (
                                        <button
                                          type="button"
                                          className="sidebar-add-child-btn"
                                          title={"Add child to " + c.id}
                                          onClick={() => setAddingChildTo(c.id)}
                                        >
                                          +
                                        </button>
                                      )}
                                      <CollectionDotMenu
                                        collectionId={c.id}
                                        collectionType={collectionType}
                                        onRename={() => setRenamingId(c.id)}
                                        onDelete={() => handleDelete(c.id)}
                                        onMoveParent={() => handleMoveParent(c.id, null)}
                                        onRemoveFromParent={() => {}}
                                      />
                                    </div>
                                  </>
                                )}
                              </div>

                              {hasChildren && addingChildTo === c.id && (
                                <div className="sidebar-child-row sidebar-add-child-form">
                                  <AddChildCollectionForm
                                    parentId={c.id}
                                    onDone={() => { setAddingChildTo(null); if (!expanded) toggleExpand(c.id); }}
                                    onCancel={() => setAddingChildTo(null)}
                                  />
                                </div>
                              )}

                              {hasChildren && expanded && (
                                <div className="sidebar-children">
                                  {children
                                    .filter(collectionMatchesQuery)
                                    .map((child) => (
                                      <div key={child.id} className="sidebar-child-row">
                                        {renamingId === child.id ? (
                                          <InlineRenameForm
                                            collectionId={child.id}
                                            onDone={() => setRenamingId(null)}
                                            onCancel={() => setRenamingId(null)}
                                          />
                                        ) : (
                                          <>
                                            <Link
                                              to={"/c/" + encodeURIComponent(child.id) + "/1"}
                                              className={"nav-link sidebar-collection-link" + (child.id === collection ? " active" : "")}
                                            >
                                              <span>{child.id}</span>
                                              {child.isProxy && <span className="proxy-pill">Proxy</span>}
                                            </Link>
                                            <div className="sidebar-collection-row-actions">
                                              <CollectionDotMenu
                                                collectionId={child.id}
                                                collectionType="child"
                                                onRename={() => setRenamingId(child.id)}
                                                onDelete={() => handleDelete(child.id)}
                                                onMoveParent={() => handleMoveParent(child.id, child.parent)}
                                                onRemoveFromParent={() => handleRemoveFromParent(child.id)}
                                              />
                                            </div>
                                          </>
                                        )}
                                      </div>
                                    ))}
                                </div>
                              )}
                            </div>
                          );
                        })
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

      {deleteParentState && (
        <DeleteParentDialog
          collectionId={deleteParentState.collectionId}
          children={deleteParentState.children}
          allCollections={allCollections}
          onConfirm={(reparentTo) => confirmDeleteParent(deleteParentState.collectionId, reparentTo)}
          onCancel={() => setDeleteParentState(null)}
        />
      )}

      {moveParentState && (
        <MoveParentModal
          collectionId={moveParentState.collectionId}
          currentParent={moveParentState.currentParent}
          allCollections={allCollections}
          onConfirm={(newParent) => confirmMoveParent(moveParentState.collectionId, newParent)}
          onCancel={() => setMoveParentState(null)}
        />
      )}
    </header>
  );
}
