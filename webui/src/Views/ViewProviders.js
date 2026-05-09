import React from "react";
import { CardCacheProvider } from "../Components/CardListContexts/CardCacheContext";
import { CardLoaderProvider } from "../Components/CardListContexts/CardLoaderContext";
import { CardSetsProvider } from "../Components/ReusableConstants/CardSets";
import { SelectedCardsProvider } from "../Components/CardListContexts/SelectedCardsContext";
import { CardsProvider } from "../Components/CardListContexts/CardsContext";
import { CollectionsProvider } from "../Components/CollectionContext";
import Header from "../Components/Layout/Header";
import { RefreshCardListProvider } from "../Components/CardListContexts/RefreshCardListContext";
import { SystemTypeProvider } from "../Components/SystemTypeContext";
import { QuickSearchProvider } from "../Components/QuickSearchContext";
import QuickSearchModal from "../Components/QuickSearchModal";

export default function ViewProviders({ children }) {
  return (
    <CollectionsProvider>
      <SystemTypeProvider>
        <QuickSearchProvider>
          <CardsProvider>
            <SelectedCardsProvider>
              <CardCacheProvider>
                <CardLoaderProvider>
                  <RefreshCardListProvider>
                    <CardSetsProvider>
                      <Header />
                      <main>
                        {children}
                        <QuickSearchModal />
                      </main>
                    </CardSetsProvider>
                  </RefreshCardListProvider>
                </CardLoaderProvider>
              </CardCacheProvider>
            </SelectedCardsProvider>
          </CardsProvider>
        </QuickSearchProvider>
      </SystemTypeProvider>
    </CollectionsProvider>
  );
}
