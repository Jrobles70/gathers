import { groupDuplicateCards, totalCopies } from "./DuplicateCollectionsView";

function card(id, collectionId, quantity = 1, foilQuantity = 0, provider = "MagicSQLite") {
  return {
    id,
    collectionId,
    quantity,
    foilQuantity,
    provider,
    timeAdded: `${collectionId}-${id}`,
  };
}

describe("duplicate collection grouping", () => {
  it("detects the same card across two collections", () => {
    const groups = groupDuplicateCards([
      card("card-a", "A1"),
      card("card-a", "A2", 2),
      card("card-b", "A1"),
    ]);

    expect(groups).toHaveLength(1);
    expect(groups[0]).toMatchObject({
      id: "card-a",
      provider: "MagicSQLite",
      collectionIds: ["A1", "A2"],
      totalCopies: 3,
    });
  });

  it("ignores repeated rows that only appear in one collection", () => {
    const groups = groupDuplicateCards([
      card("card-a", "A1"),
      card("card-a", "A1", 0, 1),
    ]);

    expect(groups).toEqual([]);
  });

  it("keeps providers separated when ids overlap", () => {
    const groups = groupDuplicateCards([
      card("shared-id", "A1", 1, 0, "MagicSQLite"),
      card("shared-id", "A2", 1, 0, "PokemonSQLite"),
    ]);

    expect(groups).toEqual([]);
  });

  it("adds regular and foil quantities into the copy total", () => {
    expect(totalCopies([card("card-a", "A1", 2, 3)])).toBe(5);
  });
});
