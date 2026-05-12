import React from "react";
import CardShell from "./CardShell";

export default function RiftboundCard({
  id,
  card = null,
  details = null,
  provider = null,
  listMode = false,
  targetCollection = null,
  detailReturnPath = null,
  onLoad = null,
  fetchCycle = 0,
}) {
  return (
    <CardShell
      id={id}
      card={card}
      details={details}
      provider={provider}
      listMode={listMode}
      targetCollection={targetCollection}
      detailReturnPath={detailReturnPath}
      onLoad={onLoad}
      fetchCycle={fetchCycle}
      detailPath={`/card/riftbound/${encodeURIComponent(id)}`}
      getImagePath={(_card) => _card.image ?? ""}
    />
  );
}
