import React, { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import ViewProviders from "./ViewProviders";

function RiftboundCardDetail({ id }) {
  const [card, setCard] = useState(null);
  const [error, setError] = useState(null);
  const navigate = useNavigate();

  useEffect(() => {
    fetch(`/riftbound/cards?ids=${encodeURIComponent(id)}`)
      .then((res) => res.json())
      .then((data) => {
        const found = data[id];
        if (found) setCard(found);
        else setError("Card not found.");
      })
      .catch(() => setError("Failed to load card."));
  }, [id]);

  if (error) return <p className="p-3 text-danger">{error}</p>;
  if (!card) return <p className="p-3">Loading...</p>;

  const artists = Array.isArray(card.artists) ? card.artists.join(", ") : (card.artist ?? "—");
  const domains = Array.isArray(card.domains) ? card.domains.join(", ") : "—";

  return (
    <div className="container mt-4">
      <button className="btn btn-link p-0" onClick={() => navigate(-1)}>← Back</button>
      <div className="row mt-3 g-4">
        <div className="col-md-4">
          {card.image ? (
            <img
              src={card.image}
              alt={card.name}
              className="img-fluid rounded zoomable"
              onMouseMove={(e) => {
                const r = e.currentTarget.getBoundingClientRect();
                e.currentTarget.style.transformOrigin = `${((e.clientX - r.left) / r.width) * 100}% ${((e.clientY - r.top) / r.height) * 100}%`;
              }}
            />
          ) : (
            <div className="p-3 bg-secondary text-white rounded">No image available</div>
          )}
        </div>
        <div className="col-md-8">
          <h2>{card.name}</h2>
          <table className="table table-sm table-bordered">
            <tbody>
              <tr>
                <th>Set</th>
                <td>{card.setCode}</td>
              </tr>
              <tr>
                <th>Collector Number</th>
                <td>{card.collectorNumber}</td>
              </tr>
              <tr>
                <th>Rarity</th>
                <td>{card.rarity}</td>
              </tr>
              <tr>
                <th>Artist(s)</th>
                <td>{artists}</td>
              </tr>
              <tr>
                <th>Domains</th>
                <td>{domains}</td>
              </tr>
              {card.text && (
                <tr>
                  <th>Text</th>
                  <td style={{ whiteSpace: "pre-line" }}>{card.text}</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

export default function RiftboundCardDetailView() {
  const { id } = useParams();
  return (
    <ViewProviders>
      <RiftboundCardDetail id={id} />
    </ViewProviders>
  );
}
