export function getMagicSearchCard(searchResult, collectionsEnabled) {
  return collectionsEnabled ? searchResult.mtGCard : searchResult;
}

export function normalizeMagicSearchResult(searchResult, collectionsEnabled) {
  const card = getMagicSearchCard(searchResult, collectionsEnabled);
  return {
    id: card.id,
    card,
    details: card.details ?? null,
  };
}

export function magicPrintingGroupKey(card) {
  return (card.name ?? card.id ?? "").trim().toLowerCase();
}

export function groupMagicSearchResults(cards, collectionsEnabled) {
  const groups = [];
  const groupsByKey = new Map();

  cards.forEach((searchResult) => {
    const printing = normalizeMagicSearchResult(searchResult, collectionsEnabled);
    const key = magicPrintingGroupKey(printing.card);

    if (!groupsByKey.has(key)) {
      const group = {
        key,
        primary: printing,
        printings: [],
      };
      groupsByKey.set(key, group);
      groups.push(group);
    }

    groupsByKey.get(key).printings.push(printing);
  });

  return groups;
}

export function listMagicSearchResultsByPrinting(cards, collectionsEnabled) {
  return cards.map((searchResult) => {
    const printing = normalizeMagicSearchResult(searchResult, collectionsEnabled);
    return {
      key: printing.id,
      primary: printing,
      printings: [printing],
    };
  });
}
