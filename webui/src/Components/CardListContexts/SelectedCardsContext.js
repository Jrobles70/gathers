import { createContext, useContext, useReducer } from "react";

export function SelectedCardsProvider({ children }) {
  const [selected, selectedDispatch] = useReducer(selectedCardsReducer, []);

  return (
    <SelectedCardsContext.Provider value={selected}>
      <SelectedCardsDispatchContext.Provider value={selectedDispatch}>
        {children}
      </SelectedCardsDispatchContext.Provider>
    </SelectedCardsContext.Provider>
  );
}

const SelectedCardsContext = createContext([]);
export function useSelectedCards() {
  return useContext(SelectedCardsContext);
}

const SelectedCardsDispatchContext = createContext(null);
export function useSelectedCardsDispatch() {
  return useContext(SelectedCardsDispatchContext);
}

function selectedCardsReducer(selected, action) {
  const sameCard = (a, b) => a.id === b.id && a.collectionId === b.collectionId;
  switch (action.type) {
    case "added": {
      if (selected.some((card) => sameCard(card, action.card))) return selected;
      return [...selected, action.card];
    }
    case "deleted": {
      return selected.filter((t) => !sameCard(t, action.card));
    }
    case "empty": {
      return [];
    }
    default: {
      throw Error("Unknown action: " + action.type);
    }
  }
}
