import React from "react";
import SearchMagic from "./SearchMagic";
import SearchRiftbound from "./SearchRiftbound";
import SearchPokemon from "./SearchPokemon";
import { useSystems, useSelectedSearchSystem } from "./SystemTypeContext";

function systemLabel(system) {
  if (system === "RiftboundSQLite") return "Riftbound";
  if (system === "PokemonSQLite") return "Pokémon";
  if (system === "Scryfall") return "MTG (Scryfall)";
  return "MTG";
}

function Search({ startSearch = false, dedicatedPage = false, sidePanel = false }) {
  const systems = useSystems();
  const [selectedSystem, setSelectedSystem] = useSelectedSearchSystem();

  const renderSearch = () => {
    if (selectedSystem === "RiftboundSQLite") {
      return (
        <SearchRiftbound
          startSearch={startSearch}
          dedicatedPage={dedicatedPage}
          sidePanel={sidePanel}
        />
      );
    }
    if (selectedSystem === "PokemonSQLite") {
      return (
        <SearchPokemon
          startSearch={startSearch}
          dedicatedPage={dedicatedPage}
          sidePanel={sidePanel}
        />
      );
    }
    return (
      <SearchMagic
        startSearch={startSearch}
        dedicatedPage={dedicatedPage}
        sidePanel={sidePanel}
      />
    );
  };

  return (
    <>
      {systems.length > 1 && (
        <div className="system-switcher btn-group mb-2" role="group">
          {systems.map((sys) => (
            <button
              key={sys}
              type="button"
              className={`btn btn-sm ${
                selectedSystem === sys ? "btn-primary" : "btn-outline-secondary"
              }`}
              onClick={() => setSelectedSystem(sys)}
            >
              {systemLabel(sys)}
            </button>
          ))}
        </div>
      )}
      {renderSearch()}
    </>
  );
}

export default Search;
