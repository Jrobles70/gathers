import { buildNewCollectionImportFormData } from "./AddCollectionForm";

describe("buildNewCollectionImportFormData", () => {
  it("builds pasted text import data for a new collection", () => {
    const formData = buildNewCollectionImportFormData({
      name: "Proxy Box",
      text: "Set,CollectorNumber,Quantity,FoilQuantity\nM13,39,2,0\n",
    });

    expect(formData.get("collection")).toBe("Proxy Box");
    expect(formData.get("text")).toBe("Set,CollectorNumber,Quantity,FoilQuantity\nM13,39,2,0\n");
    expect(formData.has("file")).toBe(false);
  });
});
