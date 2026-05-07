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

  const [file, setFile] = useState();
  const [text, setText] = useState("");
  const [importMode, setImportMode] = useState("file");

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
    <form className="import-cards-form d-flex" onSubmit={handleUploadClick}>
      <div className="input-group input-group-sm import-cards-controls">
        <div className="btn-group" role="group" aria-label="Import source">
          <button
            aria-pressed={importMode === "file"}
            className={`btn btn-outline-secondary ${
              importMode === "file" ? "active" : ""
            }`}
            onClick={() => setImportMode("file")}
            type="button"
          >
            File
          </button>
          <button
            aria-pressed={importMode === "text"}
            className={`btn btn-outline-secondary ${
              importMode === "text" ? "active" : ""
            }`}
            onClick={() => setImportMode("text")}
            type="button"
          >
            Text
          </button>
        </div>
        {importMode === "file" && (
          <input
            onChange={handleFileChange}
            type="file"
            className="form-control"
            id="inputGroupFile02"
          />
        )}
        <button
          className="btn btn-outline-secondary"
          disabled={!canImport}
          type="submit"
          id="inputGroupFileAddon04"
        >
          Import
        </button>
      </div>
      {importMode === "text" && (
        <textarea
          aria-label="Paste CSV text"
          className="form-control form-control-sm import-cards-text"
          onChange={(e) => setText(e.target.value)}
          placeholder="Set,CollectorNumber,Quantity,FoilQuantity"
          rows="3"
          value={text}
        />
      )}
    </form>
  );
}
