import React from "react";
import CardShell from "./CardShell";

export default function PokemonCard({ id, card = null, details = null, provider = null, listMode = false, targetCollection = null }) {
  return (
    <CardShell
      id={id}
      card={card}
      details={details}
      provider={provider}
      listMode={listMode}
      targetCollection={targetCollection}
      detailPath={`/card/pokemon/${encodeURIComponent(id)}`}
      getImagePath={(_card) => _card.image ?? ""}
    />
  );
}
