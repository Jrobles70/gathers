import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useOperations } from "../../OperationsContext";
import {
  collectionDisplayName,
  isAllCollections,
  useCollection,
  useCollectionsDispatch,
} from "../CollectionContext";

export default function RenameCollection() {
  const collection = useCollection();
  const collectionsDispatch = useCollectionsDispatch();
  const navigate = useNavigate();
  const ops = useOperations();
  const [name, setName] = useState(collection);
  const [error, setError] = useState(null);

  useEffect(() => {
    setName(collection);
    setError(null);
  }, [collection]);

  if (isAllCollections(collection)) return null;

  const trimmed = name.trim();
  const canRename = trimmed !== "" && trimmed !== collection;

  const renameCollection = (event) => {
    event.preventDefault();
    if (!canRename) return;
    setError(null);

    ops
      .fetch("Renaming collection " + collection, {}, "/collection/rename/" + encodeURIComponent(collection), {
        method: "post",
        headers: { Accept: "application/json", "Content-Type": "application/json" },
        body: JSON.stringify({ id: trimmed }),
      })
      .then((updatedCollection) => {
        collectionsDispatch({
          type: "renamed",
          from: collection,
          item: updatedCollection,
        });
        navigate("/c/" + encodeURIComponent(updatedCollection.id) + "/1");
      })
      .catch((e) => setError(e.message));
  };

  return (
    <form className="collection-rename-form" onSubmit={renameCollection}>
      <label htmlFor="collection-rename-input">Rename</label>
      <div className="collection-inline-form">
        <input
          id="collection-rename-input"
          className={"form-control form-control-sm" + (error ? " is-invalid" : "")}
          value={name}
          aria-label={"Rename " + collectionDisplayName(collection)}
          onChange={(event) => setName(event.target.value)}
        />
        <button className="btn btn-outline-info btn-sm" type="submit" disabled={!canRename}>
          Save
        </button>
      </div>
      {error && <div className="invalid-feedback d-block">{error}</div>}
    </form>
  );
}
