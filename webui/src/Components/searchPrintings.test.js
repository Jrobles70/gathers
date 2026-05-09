import {
  groupMagicSearchResults,
  magicPrintingGroupKey,
} from "./searchPrintings";

describe("search printings helpers", () => {
  it("groups magic search results by card name", () => {
    const groups = groupMagicSearchResults([
      { id: "one", name: "Lightning Bolt", setCode: "lea" },
      { id: "two", name: "Lightning Bolt", setCode: "4ed" },
      { id: "three", name: "Counterspell", setCode: "lea" },
    ], false);

    expect(groups).toHaveLength(2);
    expect(groups[0].primary.id).toBe("one");
    expect(groups[0].printings.map((printing) => printing.id)).toEqual(["one", "two"]);
    expect(groups[1].primary.id).toBe("three");
  });

  it("preserves collection details when grouping collection search results", () => {
    const groups = groupMagicSearchResults([
      { mtGCard: { id: "one", name: "Island", details: { collectionId: "Main", quantity: 4 } } },
      { mtGCard: { id: "two", name: "Island", details: { collectionId: "Main", quantity: 1 } } },
    ], true);

    expect(groups).toHaveLength(1);
    expect(groups[0].printings[1].details.quantity).toBe(1);
  });

  it("normalizes group keys", () => {
    expect(magicPrintingGroupKey({ name: "  Deadpool, Trading Card " })).toBe("deadpool, trading card");
  });
});

