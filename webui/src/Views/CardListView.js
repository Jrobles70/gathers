import React from "react";
import Search from "../Components/Search";
import CardListNav from "../Components/CardListNav";
import CardList from "../Components/CardList";
import ViewProviders from "./ViewProviders";
import { useMode } from "../OperationsContext";

export default function CardListView({ showSearch = false }) {
  const { mode } = useMode();
  const isSearchOnly = mode === "search-only";

  return (
    <ViewProviders>
      <Search dedicatedPage={showSearch} />
      {!isSearchOnly && <CardListNav />}
      {!isSearchOnly && <CardList />}
    </ViewProviders>
  );
}
