import { buildImportFormData } from "./ImportCards";

describe("buildImportFormData", () => {
  it("uses pasted text without requiring a file", () => {
    const formData = buildImportFormData({
      importMode: "text",
      text: "Set,CollectorNumber,Quantity,FoilQuantity\nM13,39,2,1\n",
      file: undefined,
      collection: "Pasted Collection",
    });

    expect(formData.get("text")).toBe(
      "Set,CollectorNumber,Quantity,FoilQuantity\nM13,39,2,1\n",
    );
    expect(formData.get("collection")).toBe("Pasted Collection");
    expect(formData.has("file")).toBe(false);
  });
});
