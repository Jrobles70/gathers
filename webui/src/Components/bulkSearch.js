export function normalizeBulkCardName(name) {
  return name.trim().replace(/\s+/g, " ").toLowerCase();
}

export function parseBulkSearchInput(input) {
  const byName = new Map();

  input.split(/\r?\n/).forEach((line) => {
    const trimmed = line.trim();
    if (!trimmed) return;

    const match = trimmed.match(/^(\d+)\s*x?\s+(.+)$/i);
    const quantity = match ? parseInt(match[1], 10) : 1;
    const name = (match ? match[2] : trimmed)
      .replace(/\s+#.*$/, "")
      .trim();
    const key = normalizeBulkCardName(name);

    if (!key || quantity <= 0) return;

    const existing = byName.get(key);
    if (existing) {
      existing.quantity += quantity;
    } else {
      byName.set(key, { name, quantity });
    }
  });

  return Array.from(byName.values());
}

export function flattenBulkMatches(results) {
  return results.flatMap((result) => result.matches ?? []);
}

export function bulkSearchTotals(results) {
  return results.reduce(
    (totals, result) => ({
      requested: totals.requested + (result.requestedQuantity ?? 0),
      owned: totals.owned + Math.min(result.ownedQuantity ?? 0, result.requestedQuantity ?? 0),
      needed: totals.needed + (result.neededQuantity ?? 0),
    }),
    { requested: 0, owned: 0, needed: 0 },
  );
}
