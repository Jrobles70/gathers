import React, { useEffect, useState } from "react";
import Search from "../Components/Search";
import CardList from "../Components/CardList";
import ViewProviders from "./ViewProviders";
import { useMode } from "../OperationsContext";
import MobileCollectionView from "../Components/MobileCollectionView";

function useMobileCollectionLayout() {
  const [isMobile, setIsMobile] = useState(() =>
    typeof window !== "undefined" ? window.matchMedia("(max-width: 760px)").matches : false,
  );

  useEffect(() => {
    const media = window.matchMedia("(max-width: 760px)");
    const update = () => setIsMobile(media.matches);
    update();
    media.addEventListener("change", update);
    return () => media.removeEventListener("change", update);
  }, []);

  return isMobile;
}

export default function CardListView({ showSearch = false }) {
  const { mode } = useMode();
  const isSearchOnly = mode === "search-only";
  const useMobileLayout = useMobileCollectionLayout();

  return (
    <ViewProviders>
      {isSearchOnly ? (
        <Search dedicatedPage={showSearch} />
      ) : useMobileLayout ? (
        <MobileCollectionView />
      ) : (
        <section className="collection-results-panel">
          <CardList />
        </section>
      )}
    </ViewProviders>
  );
}
