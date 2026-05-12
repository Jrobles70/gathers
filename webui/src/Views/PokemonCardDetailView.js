import React from "react";
import { useParams } from "react-router-dom";
import CardDetailLayout from "./CardDetailLayout";

function renderImage(card) {
  return card.image ? (
    <img src={card.image} alt={card.name} className="img-fluid rounded" />
  ) : (
    <div className="p-3 bg-secondary text-white rounded">No image available</div>
  );
}

function renderRows(card) {
  const energyTypes = Array.isArray(card.energyTypes) ? card.energyTypes.join(", ") : "—";
  return (
    <>
      <tr><th>Set</th><td>{card.setCode}</td></tr>
      <tr><th>Collector Number</th><td>{card.collectorNumber}</td></tr>
      <tr><th>Rarity</th><td>{card.rarity}</td></tr>
      <tr><th>Card Type</th><td>{card.cardType}</td></tr>
      <tr><th>Energy Types</th><td>{energyTypes}</td></tr>
      {card.pokedex != null && <tr><th>Pokédex #</th><td>{card.pokedex}</td></tr>}
    </>
  );
}

export default function PokemonCardDetailView() {
  const { id } = useParams();
  return (
    <CardDetailLayout
      fetchUrl={`/pokemon/cards?ids=${encodeURIComponent(id)}`}
      cardId={id}
      renderImage={renderImage}
      renderRows={renderRows}
    />
  );
}
