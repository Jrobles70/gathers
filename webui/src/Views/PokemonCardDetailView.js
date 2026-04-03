import React, { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import ViewProviders from "./ViewProviders";

function PokemonCardDetail({ id }) {
  const [card, setCard] = useState(null);
  const [error, setError] = useState(null);
  const navigate = useNavigate();

  useEffect(() => {
    fetch(`/pokemon/cards?ids=${encodeURIComponent(id)}`)
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

  const energyTypes = Array.isArray(card.energyTypes) ? card.energyTypes.join(", ") : "—";

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
                <th>Card Type</th>
                <td>{card.cardType}</td>
              </tr>
              <tr>
                <th>Energy Types</th>
                <td>{energyTypes}</td>
              </tr>
              {card.pokedex != null && (
                <tr>
                  <th>Pokédex #</th>
                  <td>{card.pokedex}</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

export default function PokemonCardDetailView() {
  const { id } = useParams();
  return (
    <ViewProviders>
      <PokemonCardDetail id={id} />
    </ViewProviders>
  );
}
