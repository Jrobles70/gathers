import { getHomePath } from "./BaseApp";
import { getSidebarBrandPath } from "./Components/Sidebar";

describe("navigation destinations", () => {
  it("sends the home route and brand link to all collections in collection mode", () => {
    expect(getHomePath({ collectionsEnabled: true })).toBe("/collections/1");
    expect(getSidebarBrandPath({ collectionsEnabled: true })).toBe("/collections/1");
  });

  it("keeps home and brand navigation on search when collections are unavailable", () => {
    expect(getHomePath({ collectionsEnabled: false })).toBe("/search");
    expect(getSidebarBrandPath({ collectionsEnabled: false })).toBe("/search");
  });

  it("keeps search-only mode on the dedicated search page", () => {
    expect(getHomePath({ mode: "search-only", collectionsEnabled: true })).toBe("/search");
    expect(getSidebarBrandPath({ isSearchOnly: true, collectionsEnabled: true })).toBe("/search");
  });
});
