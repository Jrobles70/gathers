import React from "react";
import { Link } from "react-router-dom";
import AddCollectionForm from "./AddCollectionForm";
import { useCollection, useCollections } from "./CollectionContext";
import OperationsTracker from "./CardListNavButtons/OperationsTracker";
import { useMode } from "../OperationsContext";

export default function Sidebar() {
  const collection = useCollection();
  const collections = useCollections();
  const { mode, collectionsEnabled } = useMode();
  const isSearchOnly = mode === "search-only";

  const renderCollections = () => {
    return collections.map((c) => (
      <Link
        to={"/c/" + c.id + "/1"}
        key={c.id}
        className={"nav-link" + (c.id === collection ? " active" : "")}
      >
        {c.id}
      </Link>
    ));
  };

  return (
    <header>
      <nav id="sidebarMenu" className="d-lg-block sidebar bg-white">
        <nav className="navbar navbar-expand-lg navbar-light bg-light">
          <div className="container-fluid">
            <a className="navbar-brand" href="/">
              GatheRs
            </a>
          </div>
        </nav>
        <div className="position-sticky">
          <div
            className="nav flex-column nav-pills me-3"
            role="tablist"
            aria-orientation="vertical"
          >
            <Link to={"/search"} className="btn btn-secondary">
              Search
            </Link>
          </div>
          <hr />
          <div
            className="nav flex-column nav-pills me-3"
            role="tablist"
            aria-orientation="vertical"
          >
            {!isSearchOnly && collectionsEnabled && (
              <React.Fragment>
                {renderCollections(collections)}
                <hr />
                <AddCollectionForm />
                <hr />
                <OperationsTracker />
              </React.Fragment>
            )}
          </div>
        </div>
      </nav>
    </header>
  );
}
