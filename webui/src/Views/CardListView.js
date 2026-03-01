import React, { useState } from "react";
import Search from "../Components/Search";
import CardListNav from "../Components/CardListNav";
import CardList from "../Components/CardList";
import ViewProviders from "./ViewProviders";
import { useMode } from "../OperationsContext";

export default function CardListView({ showSearch = false }) {
  const { mode } = useMode();
  const isSearchOnly = mode === "search-only";
  const [searchOpen, setSearchOpen] = useState(false);

  return (
    <ViewProviders>
      {isSearchOnly ? (
        <Search dedicatedPage={showSearch} />
      ) : searchOpen ? (
        <div className="collection-split-view">
          <div className="collection-search-panel">
            <Search sidePanel={true} />
          </div>
          <div className="collection-cards-panel">
            <CardListNav onToggleSearch={() => setSearchOpen(false)} searchOpen={searchOpen} />
            <CardList />
          </div>
        </div>
      ) : (
        <>
          <CardListNav onToggleSearch={() => setSearchOpen(true)} searchOpen={searchOpen} />
          <CardList />
        </>
      )}
    </ViewProviders>
  );
}
