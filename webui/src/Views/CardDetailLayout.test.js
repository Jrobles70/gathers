import { resolveDetailReturnPath } from "./CardDetailLayout";

describe("CardDetailLayout helpers", () => {
  it("uses a route state return path when present", () => {
    expect(resolveDetailReturnPath({ returnTo: "/search?name=bolt&page=2" })).toBe("/search?name=bolt&page=2");
  });

  it("falls back to history navigation when no return path is present", () => {
    expect(resolveDetailReturnPath(null)).toBeNull();
  });
});

