import {
  formatOperationTime,
  prependOperationLog,
  updateOperationLog,
} from "./OperationsContext";

describe("operation log helpers", () => {
  it("formats operation timestamps as HH:MM:SS", () => {
    expect(formatOperationTime(new Date(2026, 4, 8, 9, 5, 7))).toBe("09:05:07");
  });

  it("keeps only the latest 200 operation logs", () => {
    const logs = Array.from({ length: 200 }, (_, index) => ({
      id: String(index),
      message: `operation ${index}`,
    }));

    const next = prependOperationLog(logs, { id: "new", message: "new operation" });

    expect(next).toHaveLength(200);
    expect(next[0].id).toBe("new");
    expect(next.some((log) => log.id === "199")).toBe(false);
  });

  it("updates an existing operation log entry", () => {
    const logs = [{ id: "a", status: "active" }, { id: "b", status: "active" }];

    expect(updateOperationLog(logs, "b", { status: "done" })).toEqual([
      { id: "a", status: "active" },
      { id: "b", status: "done" },
    ]);
  });
});
