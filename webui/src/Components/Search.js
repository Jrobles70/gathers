import React from "react";
import { useSearchParams } from "react-router-dom";
import SearchMagic from "./SearchMagic";
import SearchRiftbound from "./SearchRiftbound";
import SearchPokemon from "./SearchPokemon";
import { useSystemType } from "./SystemTypeContext";

function Search({ startSearch = false, dedicatedPage = false, sidePanel = false }) {
  const systemType = useSystemType();
  const [searchParams] = useSearchParams();

  if (systemType === "RiftboundSql" || searchParams.get("riftbound") === "true") {
    return (
      <SearchRiftbound
        startSearch={startSearch}
        dedicatedPage={dedicatedPage}
        sidePanel={sidePanel}
      />
    );
  }

  if (systemType === "PokemonSql") {
    return (
      <SearchPokemon
        startSearch={startSearch}
        dedicatedPage={dedicatedPage}
        sidePanel={sidePanel}
      />
    );
  }

  return (
    <SearchMagic startSearch={startSearch} dedicatedPage={dedicatedPage} sidePanel={sidePanel} />
  );
}

export default Search;
