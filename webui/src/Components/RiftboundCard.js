import React from "react";
import CardShell from "./CardShell";

export default function RiftboundCard({ id, card = null, details = null, provider = null, listMode = false }) {
  return (
    <CardShell
      id={id}
      card={card}
      details={details}
      provider={provider}
      listMode={listMode}
      detailPath={`/card/riftbound/${encodeURIComponent(id)}`}
      getImagePath={(_card) => _card.image ?? ""}
    />
  );
}
