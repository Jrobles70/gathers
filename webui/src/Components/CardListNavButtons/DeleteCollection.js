import React from "react";
import { confirm } from "./ConfirmCollectionDelete";
import { useNavigate } from "react-router-dom";
import { useOperations } from "../../OperationsContext";
import {
  isAllCollections,
  useCollection,
  useCollections,
  useCollectionsDispatch,
} from "../CollectionContext";

export default function DeleteCollection() {
  const navigate = useNavigate();
  const ops = useOperations();
  const collection = useCollection();
  const collections = useCollections();
  const collectionsDispatch = useCollectionsDispatch();

  if (isAllCollections(collection)) return null;

  const deleteCollection = () => {
    let moveToCollections = collections.filter((s) => s.id !== collection);
    moveToCollections.push({ id: "" });
    confirm({
      confirmType: "collection",
      collection: collection,
      collections: moveToCollections,
    }).then(
      ({ input }) => {
        ops
          .fetch(
            "Deleting collection " + collection,
            {},
            "/collection/remove/" +
              encodeURIComponent(collection) +
              "?keepCardsInCollection=" +
              encodeURIComponent(input ?? ""),
            {
              method: "post",
              headers: {
                Accept: "application/json",
                "Content-Type": "application/json",
              },
            },
          )
          .then((data) => {
            collectionsDispatch({ type: "deleted", id: collection });
            navigate(input ? "/c/" + encodeURIComponent(input) + "/1" : "/collections/1");
          });
      },
      () => {},
    );
  };

  return (
    <button
      onClick={deleteCollection}
      type="button"
      className="btn btn-outline-danger btn-sm w-100"
    >
      Delete collection
    </button>
  );
}
