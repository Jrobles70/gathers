import React from "react";
import CardShell from "./CardShell";

export default function MtGCard({
  id,
  card = null,
  details = null,
  provider = null,
  showCollectionSelect = false,
  listMode = false,
  targetCollection = null,
  printings = [],
  detailReturnPath = null,
}) {
  return (
    <CardShell
      id={id}
      card={card}
      details={details}
      provider={provider}
      showCollectionSelect={showCollectionSelect}
      listMode={listMode}
      targetCollection={targetCollection}
      printings={printings}
      detailReturnPath={detailReturnPath}
      detailPath={`/card/mtg/${encodeURIComponent(id)}`}
      makeDetailPath={(cardId) => `/card/mtg/${encodeURIComponent(cardId)}`}
      getImagePath={(_card) =>
        _card.cardIdentifiers?.scryfallId
          ? `https://api.scryfall.com/cards/${_card.cardIdentifiers.scryfallId}?format=image`
          : ""
      }
    />
  );
}
