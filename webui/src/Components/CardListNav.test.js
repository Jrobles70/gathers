import { ALL_COLLECTIONS_ID } from "./CollectionContext";
import { hasCollectionActions } from "./CardListNav";

describe("hasCollectionActions", () => {
  it("hides collection actions on the all collections view", () => {
    expect(hasCollectionActions(ALL_COLLECTIONS_ID)).toBe(false);
  });

  it("shows collection actions for a named collection", () => {
    expect(hasCollectionActions("Main")).toBe(true);
  });
});
