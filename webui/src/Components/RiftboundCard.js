import React from "react";
import CardShell from "./CardShell";

export default function RiftboundCard({ id, card = null, details = null, provider = null }) {
  return (
    <CardShell
      id={id}
      card={card}
      details={details}
      provider={provider}
      detailPath={`/card/riftbound/${encodeURIComponent(id)}`}
      getImagePath={(_card) => _card.image ?? ""}
    />
  );
}
