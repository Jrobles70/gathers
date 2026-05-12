import React, { useState } from "react";
import { Button, Form } from "react-bootstrap";
import { useOperations } from "../OperationsContext";
import { useCollectionsDispatch } from "./CollectionContext";

export function buildNewCollectionImportFormData({ name, text }) {
  const formData = new FormData();
  formData.append("collection", name);
  formData.append("text", text);
  return formData;
}

function AddCollectionForm() {
  const [showForm, setShowForm] = useState(false);
  const [newItem, setNewItem] = useState("");
  const [importText, setImportText] = useState("");
  const [isProxy, setIsProxy] = useState(false);
  const [error, setError] = useState(null);

  const collectionsDispatch = useCollectionsDispatch();
  const ops = useOperations();

  const handleToggleForm = () => {
    setShowForm(!showForm);
    setError(null);
  };
  const handleHideForm = () => {
    setShowForm(false);
    setError(null);
  };

  const handleSubmit = (event) => {
    event.preventDefault();
    setError(null);

    const name = newItem.trim();
    const text = importText.trim();
    const saveCollection = text.length > 0
      ? ops.fetch("Importing new collection", {}, "/collection/import", {
        method: "POST",
        body: buildNewCollectionImportFormData({ name, text: importText }),
      })
      : ops.fetch("Adding new collection", {}, "/collection/add", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: name, isProxy }),
      });

    saveCollection
      .then(() => (
        text.length > 0 && isProxy
          ? ops.fetch("Marking collection as proxy", {}, "/collection/proxy/" + encodeURIComponent(name), {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ isProxy: true }),
          })
          : null
      ))
      .then(() => {
        collectionsDispatch({
          type: "added",
          item: { id: name, isProxy, canRemove: true },
        });
        setNewItem("");
        setImportText("");
        setIsProxy(false);
        setShowForm(false);
      })
      .catch((e) => {
        setError(e.message);
      });
  };

  return (
    <div className="add-collection-form">
      <Button className="add-collection-toggle" onClick={handleToggleForm} variant="primary">
        Add Collection
      </Button>
      {showForm && (
        <Form className="add-collection-fields" onSubmit={handleSubmit}>
          <Form.Group controlId="newItem">
            <Form.Control
              type="text"
              value={newItem}
              placeholder="Collection name"
              onChange={(event) => setNewItem(event.target.value)}
              isInvalid={!!error}
            />
            <Form.Control.Feedback type="invalid">{error}</Form.Control.Feedback>
          </Form.Group>
          <Form.Group controlId="newCollectionImportText">
            <Form.Label>Text import</Form.Label>
            <Form.Control
              as="textarea"
              rows={5}
              value={importText}
              placeholder="Name,Set code,Set name,Collector number,Foil,Rarity,Quantity,..."
              onChange={(event) => setImportText(event.target.value)}
            />
          </Form.Group>
          <label className="add-collection-proxy-row">
            <input
              type="checkbox"
              checked={isProxy}
              onChange={(event) => setIsProxy(event.target.checked)}
            />
            <span>Proxy collection</span>
          </label>
          <div className="add-collection-actions">
            <Button disabled={newItem.trim() === ""} variant="primary" type="submit">
              Save
            </Button>
            <Button variant="secondary" onClick={handleHideForm}>
              Cancel
            </Button>
          </div>
        </Form>
      )}
    </div>
  );
}

export default AddCollectionForm;
