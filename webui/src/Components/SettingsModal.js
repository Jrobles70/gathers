import React, { useState } from "react";
import { useOperations } from "../OperationsContext";

export default function SettingsModal({ open, onClose }) {
  const ops = useOperations();
  const debugEnabled = ops.debugEnabled ?? false;
  const setDebugEnabled = ops.setDebugEnabled ?? (() => {});
  const [updateMessage, setUpdateMessage] = useState("");

  const checkMtgUpdates = () => {
    setUpdateMessage("");
    ops
      .fetch("Checking MTG card updates", "", "/mtg/update")
      .then((message) => setUpdateMessage(message || "MTG card update check complete"))
      .catch((error) => setUpdateMessage(error.message || "MTG card update check failed"));
  };

  if (!open) return null;

  return (
    <div className="settings-backdrop" onMouseDown={onClose}>
      <section
        className="settings-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="settings-header">
          <h2 id="settings-title">Settings</h2>
          <button
            type="button"
            className="btn btn-outline-secondary"
            onClick={onClose}
            aria-label="Close settings"
          >
            Close
          </button>
        </header>

        <div className="settings-body">
          <label className="settings-toggle">
            <input
              type="checkbox"
              className="form-check-input"
              checked={debugEnabled}
              onChange={(event) => setDebugEnabled(event.target.checked)}
            />
            <span>
              <strong>Show debug operation logs</strong>
              <small>Display the last 20 operation entries in the sidebar.</small>
            </span>
          </label>

          <div className="settings-section">
            <div>
              <h3>Card Data</h3>
              <p>Check MTGJSON for new MTG cards. If the local database already matches the remote SHA, nothing is downloaded.</p>
            </div>
            <button type="button" className="btn btn-primary" onClick={checkMtgUpdates}>
              Check MTG Updates
            </button>
            {updateMessage && (
              <div className="settings-status" role="status">
                {updateMessage}
              </div>
            )}
          </div>
        </div>
      </section>
    </div>
  );
}
