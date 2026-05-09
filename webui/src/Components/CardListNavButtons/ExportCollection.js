import React from "react";
import { useCollection } from "../CollectionContext";

export default function ExportCollection() {
  const collection = useCollection();

  return (
    <a className="btn btn-outline-info btn-sm w-100" href={"/collection/export/" + collection}>
      Export collection
    </a>
  );
}
