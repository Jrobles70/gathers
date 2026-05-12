import React from "react";
import { isAllCollections, useCollection } from "../CollectionContext";

export default function ExportCollection() {
  const collection = useCollection();
  if (isAllCollections(collection)) return null;

  return (
    <a className="btn btn-outline-info btn-sm w-100" href={"/collection/export/" + encodeURIComponent(collection)}>
      Export collection
    </a>
  );
}
