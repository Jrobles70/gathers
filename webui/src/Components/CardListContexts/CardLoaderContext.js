import React, { createContext, useCallback, useContext, useMemo, useRef } from "react";
import DataLoader from "dataloader";
import { useOperations } from "../../OperationsContext";
import { useCardCache } from "./CardCacheContext";
import { useSystemType } from "../SystemTypeContext";

const CardLoaderContext = createContext(null);
export function useCardLoader() {
  return useContext(CardLoaderContext);
}


export function CardLoaderProvider({ children }) {
  const { fetch: trackedFetch, quietFetch } = useOperations();
  const [cache, dispatch] = useCardCache();
  const systemType = useSystemType();
  const systemTypeRef = useRef(systemType);
  const cacheRef = useRef(cache);
  systemTypeRef.current = systemType;
  cacheRef.current = cache;

  const fetchDetails = useCallback((endpoint, keys) => {
    const params = new URLSearchParams(keys.map((k) => ["ids", k]));
    const url = endpoint + "?" + params.toString();

    if (quietFetch) {
      return quietFetch({}, url);
    }

    return trackedFetch(
      "Bulk updating details for cards",
      {},
      url,
    );
  }, [quietFetch, trackedFetch]);

  const loaders = useMemo(() => {
    function makeLoader(endpoint) {
      return new DataLoader(async (keys) => {
        const results = await fetchDetails(endpoint, keys);
        return keys.map((key) => results[key] || new Error(`No card for ${key}`));
      });
    }

    return {
      mtg: makeLoader("/mtg/cards"),
      riftbound: makeLoader("/riftbound/cards"),
      pokemon: makeLoader("/pokemon/cards"),
    };
  }, [fetchDetails]);

  const loadCard = useCallback(async (id, provider) => {
    function getLoader(effectiveProvider) {
      if (effectiveProvider === "RiftboundSQLite") return loaders.riftbound;
      if (effectiveProvider === "PokemonSQLite") return loaders.pokemon;
      return loaders.mtg;
    }

    const effectiveProvider = provider || systemTypeRef.current || "MagicSQLite";
    const cacheKey = effectiveProvider + ":" + id;
    if (cacheRef.current[cacheKey]) {
      return cacheRef.current[cacheKey];
    }
    const card = await getLoader(effectiveProvider).load(id);
    dispatch({ type: "ADD-TO-CACHE", id: cacheKey, data: card });
    return card;
  }, [dispatch, loaders]);

  return (
    <CardLoaderContext.Provider value={loadCard}>
      {children}
    </CardLoaderContext.Provider>
  );
}
