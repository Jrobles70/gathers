import { buildSearchReturnPath } from "./Search";

describe("Search helpers", () => {
  it("builds a search return path with current query params", () => {
    expect(buildSearchReturnPath(new URLSearchParams("name=bolt&page=3"))).toBe("/search?name=bolt&page=3");
  });

  it("builds a search return path without empty query params", () => {
    expect(buildSearchReturnPath(new URLSearchParams())).toBe("/search");
  });
});

