import React, { useEffect, useState } from "react";
import Search from "../Components/Search";
import ViewProviders from "./ViewProviders";
import MobileSearchView from "../Components/MobileSearchView";

function useIsMobile() {
  const [isMobile, setIsMobile] = useState(() => window.matchMedia("(max-width: 760px)").matches);
  useEffect(() => {
    const mq = window.matchMedia("(max-width: 760px)");
    const handler = (e) => setIsMobile(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);
  return isMobile;
}

export default function SearchView() {
  const isMobile = useIsMobile();
  return (
    <ViewProviders>
      {isMobile ? <MobileSearchView /> : <Search startSearch={false} dedicatedPage={true} />}
    </ViewProviders>
  );
}
