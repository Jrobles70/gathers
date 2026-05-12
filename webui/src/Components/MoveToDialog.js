import { useEffect, useMemo, useState } from "react";
import Modal from "react-bootstrap/Modal";
import Button from "react-bootstrap/Button";
import { useOperations } from "../OperationsContext";
import {
  isAllCollections,
  useCollection,
  useCollections,
  useCollectionsDispatch,
} from "./CollectionContext";
import {
  useSelectedCards,
  useSelectedCardsDispatch,
} from "./CardListContexts/SelectedCardsContext";
import { useRefreshCardList } from "./CardListContexts/RefreshCardListContext";

export default function MoveToDialog({ onClose }) {
  const ops = useOperations();
  const collection = useCollection();
  const collections = useCollections();
  const collectionsDispatch = useCollectionsDispatch();
  const selected = useSelectedCards();
  const selectedDispatch = useSelectedCardsDispatch();
  const triggerRefresh = useRefreshCardList();

  const [activeTab, setActiveTab] = useState("existing");
  const [destinationId, setDestinationId] = useState("");
  const [newName, setNewName] = useState("");
  const [newParent, setNewParent] = useState("");
  const [error, setError] = useState(null);
  const [submitting, setSubmitting] = useState(false);

  const destinationOptions = useMemo(
    () =>
      collections.filter(
        (c) => c.id !== collection && !isAllCollections(c.id)
      ),
    [collections, collection]
  );

  const parentOptions = useMemo(
    () => collections.filter((c) => !isAllCollections(c.id)),
    [collections]
  );

  // Initialize destinationId with first available option
  useEffect(() => {
    setDestinationId((current) => {
      if (current && destinationOptions.some((c) => c.id === current)) {
        return current;
      }
      return destinationOptions[0]?.id ?? "";
    });
  }, [destinationOptions]);

  // Initialize newParent based on current collection's parent
  useEffect(() => {
    const currentCollectionObj = collections.find((c) => c.id === collection);
    setNewParent(currentCollectionObj?.parent ?? "");
  }, [collections, collection]);

  const moveToExisting = () => {
    if (!destinationId) return;
    setSubmitting(true);
    setError(null);
    ops
      .fetch(
        "Moving cards to " + destinationId,
        [],
        "/collection/move/" + encodeURIComponent(destinationId),
        {
          method: "post",
          headers: { Accept: "application/json", "Content-Type": "application/json" },
          body: JSON.stringify(selected),
        }
      )
      .then(() => {
        triggerRefresh(true);
        selectedDispatch({ type: "empty" });
        onClose();
      })
      .catch((e) => {
        setError(e.message);
        setSubmitting(false);
      });
  };

  const moveToNew = () => {
    if (!newName.trim()) return;
    setSubmitting(true);
    setError(null);
    const id = newName.trim();
    ops
      .fetch(
        "Creating collection " + id,
        {},
        "/collection/add",
        {
          method: "post",
          headers: { Accept: "application/json", "Content-Type": "application/json" },
          body: JSON.stringify({ id, parent: newParent || null }),
        }
      )
      .then(() => {
        collectionsDispatch({
          type: "added",
          item: { id, parent: newParent || null, canRemove: true },
        });
        return ops.fetch(
          "Moving cards to " + id,
          [],
          "/collection/move/" + encodeURIComponent(id),
          {
            method: "post",
            headers: { Accept: "application/json", "Content-Type": "application/json" },
            body: JSON.stringify(selected),
          }
        );
      })
      .then(() => {
        triggerRefresh(true);
        selectedDispatch({ type: "empty" });
        onClose();
      })
      .catch((e) => {
        setError(e.message);
        setSubmitting(false);
      });
  };

  return (
    <Modal show onHide={onClose} animation={false}>
      <Modal.Header closeButton>
        <Modal.Title>
          Move {selected.length} card{selected.length !== 1 ? "s" : ""} to…
        </Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <div className="mb-3">
          <button
            type="button"
            className={
              "btn btn-sm me-2" +
              (activeTab === "existing" ? " btn-primary" : " btn-outline-secondary")
            }
            onClick={() => {
              setActiveTab("existing");
              setError(null);
            }}
          >
            Existing Collection
          </button>
          <button
            type="button"
            className={
              "btn btn-sm" +
              (activeTab === "new" ? " btn-primary" : " btn-outline-secondary")
            }
            onClick={() => {
              setActiveTab("new");
              setError(null);
            }}
          >
            New Collection
          </button>
        </div>

        {activeTab === "existing" && (
          <div>
            {destinationOptions.length === 0 ? (
              <p className="text-muted">No other collections available.</p>
            ) : (
              <select
                className="form-select"
                value={destinationId}
                onChange={(e) => setDestinationId(e.target.value)}
              >
                <option value="">-- Select a collection --</option>
                {destinationOptions.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.id}
                  </option>
                ))}
              </select>
            )}
          </div>
        )}

        {activeTab === "new" && (
          <div>
            <div className="mb-3">
              <label className="form-label">Collection name</label>
              <input
                type="text"
                className="form-control"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                placeholder="Name"
                autoFocus
              />
            </div>
            <div className="mb-3">
              <label className="form-label">Parent collection (optional)</label>
              <select
                className="form-select"
                value={newParent}
                onChange={(e) => setNewParent(e.target.value)}
              >
                <option value="">No parent (top-level)</option>
                {parentOptions.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.id}
                  </option>
                ))}
              </select>
            </div>
          </div>
        )}

        {error && <p className="text-danger mt-2">{error}</p>}
      </Modal.Body>
      <Modal.Footer>
        {activeTab === "existing" && (
          <Button onClick={moveToExisting} disabled={!destinationId || submitting}>
            Move
          </Button>
        )}
        {activeTab === "new" && (
          <Button onClick={moveToNew} disabled={!newName.trim() || submitting}>
            Create & Move
          </Button>
        )}
        <Button variant="secondary" onClick={onClose}>
          Cancel
        </Button>
      </Modal.Footer>
    </Modal>
  );
}
