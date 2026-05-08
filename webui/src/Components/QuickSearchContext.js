import { createContext, useContext, useState } from "react";

const QuickSearchContext = createContext({
  quickSearchOpen: false,
  openQuickSearch: () => {},
  closeQuickSearch: () => {},
});

export function QuickSearchProvider({ children }) {
  const [quickSearchOpen, setQuickSearchOpen] = useState(false);

  return (
    <QuickSearchContext.Provider
      value={{
        quickSearchOpen,
        openQuickSearch: () => setQuickSearchOpen(true),
        closeQuickSearch: () => setQuickSearchOpen(false),
      }}
    >
      {children}
    </QuickSearchContext.Provider>
  );
}

export function useQuickSearch() {
  return useContext(QuickSearchContext);
}
