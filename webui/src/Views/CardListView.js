import React from "react";
import Search from "../Components/Search";
import CardList from "../Components/CardList";
import ViewProviders from "./ViewProviders";
import { useMode } from "../OperationsContext";

export default function CardListView({ showSearch = false }) {
  const { mode } = useMode();
  const isSearchOnly = mode === "search-only";

  return (
    <ViewProviders>
      {isSearchOnly ? (
        <Search dedicatedPage={showSearch} />
      ) : (
        <section className="collection-results-panel">
          <CardList />
        </section>
      )}
    </ViewProviders>
  );
}
