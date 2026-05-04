import React, { useState, useEffect, createContext, useContext } from "react";
import { useOperations } from "../../OperationsContext";

const CardSetsContext = createContext([]);
export function useCardSets() {
  return useContext(CardSetsContext);
}

export function CardSetsProvider({ children }) {
  const { fetch: opsFetch } = useOperations();

  const [sets, setSets] = useState([]);

  useEffect(() => {
    opsFetch("Getting all available sets", [], "/mtg/sets").then((data) => {
      setSets([{ code: "", name: "" }, ...data]);
    });
  }, [opsFetch]);

  return (
    <CardSetsContext.Provider value={sets}>{children}</CardSetsContext.Provider>
  );
}
