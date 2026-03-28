import React, { useState, useEffect } from "react";
import CardDetails from "./CardDetails";
import { useSelectedCardsDispatch } from "./CardListContexts/SelectedCardsContext";
import { useCardLoader } from "./CardListContexts/CardLoaderContext";

export default function RiftboundCard({ id, card = null, details = null, provider = null }) {
  const [_card, setCard] = useState(card);
  const [selected, setSelected] = useState(false);

  const selectedDispatch = useSelectedCardsDispatch();
  const loader = useCardLoader();

  const toggleSelected = () => {
    if (details != null) {
      selectedDispatch({
        type: !selected ? "added" : "deleted",
        card: details,
      });
      setSelected((s) => !s);
    }
  };

  useEffect(() => {
    if (_card == null) {
      loader(id, provider).then(setCard).catch(() => {});
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id, _card, details, provider]);

  let imagePath =
    _card != null && _card.image != null
      ? _card.image
      : "";

  return (
    <>
      {_card == null ? (
        <p>Loading...</p>
      ) : (
        <div className={"card" + (selected ? " border border-primary" : "")}>
          <img
            src={imagePath}
            alt={_card.name}
            loading="lazy"
          />
          <CardDetails
            id={id}
            details={details}
            toggleSelected={toggleSelected}
          />
          <div className="card-info">
            <div className="row align-items-center">
              <span className="col-sm-8">
                {_card.name}
                {details != null ? (
                  <span className="badge bg-secondary">
                    {details.collectionId}
                  </span>
                ) : (
                  ""
                )}
              </span>
              <span className="col-sm-11">{_card.setCode}</span>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
