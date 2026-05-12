import {
  bulkSearchTotals,
  flattenBulkMatches,
  parseBulkSearchInput,
} from "./bulkSearch";

describe("bulk search helpers", () => {
  it("parses deck-list style quantities", () => {
    expect(parseBulkSearchInput(`
      5 Swamp
      1x Tainted Peak
      Twinflame
    `)).toEqual([
      { name: "Swamp", quantity: 5 },
      { name: "Tainted Peak", quantity: 1 },
      { name: "Twinflame", quantity: 1 },
    ]);
  });

  it("combines duplicate card names", () => {
    expect(parseBulkSearchInput("2 Swamp\n3 swamp")).toEqual([
      { name: "Swamp", quantity: 5 },
    ]);
  });

  it("flattens owned matches for card rendering", () => {
    expect(flattenBulkMatches([
      { matches: [{ mtGCard: { id: "one" } }] },
      { matches: [{ mtGCard: { id: "two" } }] },
    ])).toHaveLength(2);
  });

  it("summarizes requested, owned, and needed quantities", () => {
    expect(bulkSearchTotals([
      { requestedQuantity: 5, ownedQuantity: 3, neededQuantity: 2 },
      { requestedQuantity: 1, ownedQuantity: 2, neededQuantity: 0 },
    ])).toEqual({ requested: 6, owned: 4, needed: 2 });
  });
});
