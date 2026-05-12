import { createContext, useContext, useReducer } from "react";

export const pageSize = 50;

export function CardsProvider({ children }) {
  const [cards, cardsDispatch] = useReducer(cardsReducer, []);

  return (
    <CardsContext.Provider value={cards}>
      <CardsDispatchContext.Provider value={cardsDispatch}>
        {children}
      </CardsDispatchContext.Provider>
    </CardsContext.Provider>
  );
}

const CardsContext = createContext([]);
export function useCards() {
  return useContext(CardsContext);
}

const CardsDispatchContext = createContext(null);
export function useCardsDispatch() {
  return useContext(CardsDispatchContext);
}

function cardsReducer(cards, action) {
  const sameCard = (a, b) => a.id === b.id && a.collectionId === b.collectionId;
  switch (action.type) {
    case "added": {
      let updated = false;
      const newCards = cards.map((c) => {
        if (sameCard(c, action.card)) {
          updated = true;
          return action.card;
        }
        return c;
      });
      if (updated) {
        return newCards;
      }
      return cards;
    }
    case "deleted": {
      return cards.filter((t) => !sameCard(t, action.card));
    }
    case "overwrite": {
      return action.cards;
    }
    default: {
      throw Error("Unknown action: " + action.type);
    }
  }
}
