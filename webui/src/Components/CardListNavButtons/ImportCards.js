import React, { useState } from "react";
import { useOperations } from "../../OperationsContext";
import { useCollection } from "../CollectionContext";
import { useRefreshCardList } from "../CardListContexts/RefreshCardListContext";

export function buildImportFormData({ importMode, file, text, collection }) {
  const formData = new FormData();
  if (importMode === "text") {
    formData.append("text", text);
  } else {
    formData.append("file", file);
  }
  formData.append("collection", collection);
  return formData;
}

export default function ImportCards() {
  const ops = useOperations();
  const collection = useCollection();
  const triggerRefresh = useRefreshCardList();

  const [open, setOpen] = useState(false);
  const [file, setFile] = useState();
  const [text, setText] = useState("");
  const [importMode, setImportMode] = useState("text");

  const handleFileChange = (e) => {
    if (e.target.files) {
      setFile(e.target.files[0]);
    }
  };

  const canImport =
    importMode === "text" ? text.trim().length > 0 : Boolean(file);

  const handleUploadClick = (e) => {
    e.preventDefault();
    if (!canImport) {
      return;
    }

    const formData = buildImportFormData({
      importMode,
      file,
      text,
      collection,
    });

    ops
      .fetch("Importing into " + collection, [], "/collection/import", {
        method: "post",
        body: formData,
      })
      .then(() => {
        if (importMode === "text") {
          setText("");
        }
        triggerRefresh(true);
      });
  };

  return (
    <form className="import-cards-form" onSubmit={handleUploadClick}>
      <button
        type="button"
        className="collection-panel-toggle"
        aria-expanded={open}
        onClick={() => setOpen((isOpen) => !isOpen)}
      >
        <span>Import</span>
        <span aria-hidden="true">{open ? "^" : "v"}</span>
      </button>

      {open && (
        <div className="collection-panel-dropdown import-cards-dropdown">
          <label className="form-label" htmlFor="collection-import-source">
            Source
          </label>
          <select
            className="form-select form-select-sm"
            id="collection-import-source"
            onChange={(e) => setImportMode(e.target.value)}
            value={importMode}
          >
            <option value="file">File</option>
            <option value="text">Text</option>
          </select>

          {importMode === "file" ? (
            <input
              onChange={handleFileChange}
              type="file"
              className="form-control form-control-sm"
              id="inputGroupFile02"
            />
          ) : (
            <textarea
              aria-label="Paste CSV text"
              className="form-control form-control-sm import-cards-text"
              onChange={(e) => setText(e.target.value)}
              placeholder="Name,Set code,Set name,Collector number,Foil,Rarity,Quantity,..."
              rows="4"
              value={text}
            />
          )}

          <button
            className="btn btn-outline-info btn-sm"
            disabled={!canImport}
            type="submit"
            id="inputGroupFileAddon04"
          >
            Import
          </button>
        </div>
      )}
    </form>
  );
}
