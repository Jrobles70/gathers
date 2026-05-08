import React from "react";
import Search from "../Components/Search";
import CardListNav from "../Components/CardListNav";
import CardList from "../Components/CardList";
import CollectionFilterBar from "../Components/CollectionFilterBar";
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
        <>
          <CardListNav />
          <CollectionFilterBar />
          <CardList />
        </>
      )}
    </ViewProviders>
  );
}
