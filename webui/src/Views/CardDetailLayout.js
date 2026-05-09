import React, { useState, useEffect } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import ViewProviders from "./ViewProviders";

export function resolveDetailReturnPath(locationState) {
  return locationState?.returnTo ?? null;
}

function CardDetailContent({ fetchUrl, cardId, renderImage, renderRows }) {
  const [card, setCard] = useState(null);
  const [error, setError] = useState(null);
  const navigate = useNavigate();
  const location = useLocation();
  const returnPath = resolveDetailReturnPath(location.state);

  useEffect(() => {
    fetch(fetchUrl)
      .then((res) => res.json())
      .then((data) => {
        const found = data[cardId];
        if (found) setCard(found);
        else setError("Card not found.");
      })
      .catch(() => setError("Failed to load card."));
  }, [fetchUrl, cardId]);

  if (error) return <p className="p-3 text-danger">{error}</p>;
  if (!card) return <p className="p-3">Loading...</p>;

  return (
    <div className="container mt-4">
      <button
        className="btn btn-link p-0"
        onClick={() => {
          if (returnPath) {
            navigate(returnPath);
          } else {
            navigate(-1);
          }
        }}
      >
        ← Back
      </button>
      <div className="row mt-3 g-4">
        <div className="col-md-4">{renderImage(card)}</div>
        <div className="col-md-8">
          <h2>{card.name}</h2>
          <table className="table table-sm table-bordered">
            <tbody>{renderRows(card)}</tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

export default function CardDetailLayout({ fetchUrl, cardId, renderImage, renderRows }) {
  return (
    <ViewProviders>
      <CardDetailContent
        fetchUrl={fetchUrl}
        cardId={cardId}
        renderImage={renderImage}
        renderRows={renderRows}
      />
    </ViewProviders>
  );
}
