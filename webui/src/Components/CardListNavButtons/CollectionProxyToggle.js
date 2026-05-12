import React from "react";
import { useOperations } from "../../OperationsContext";
import {
  isAllCollections,
  useCollection,
  useCollections,
  useCollectionsDispatch,
} from "../CollectionContext";
import { useRefreshCardList } from "../CardListContexts/RefreshCardListContext";

export default function CollectionProxyToggle() {
  const collection = useCollection();
  const collections = useCollections();
  const collectionsDispatch = useCollectionsDispatch();
  const triggerRefresh = useRefreshCardList();
  const ops = useOperations();

  if (isAllCollections(collection)) return null;

  const current = collections.find((item) => item.id === collection);
  const checked = Boolean(current?.isProxy);

  const setCollectionProxy = (event) => {
    const isProxy = event.target.checked;
    ops
      .fetch("Updating proxy status for " + collection, {}, "/collection/proxy/" + encodeURIComponent(collection), {
        method: "post",
        headers: { Accept: "application/json", "Content-Type": "application/json" },
        body: JSON.stringify({ isProxy }),
      })
      .then((updatedCollection) => {
        collectionsDispatch({ type: "updated", item: updatedCollection });
        triggerRefresh(true);
      });
  };

  return (
    <label className="collection-proxy-toggle">
      <input
        type="checkbox"
        checked={checked}
        onChange={setCollectionProxy}
      />
      <span>Proxy collection</span>
    </label>
  );
}
