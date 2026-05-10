export function formatCents(cents) {
  if (cents == null || Number.isNaN(Number(cents))) return "$-";
  return (Number(cents) / 100).toLocaleString(undefined, {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

export function parseCents(value) {
  const trimmed = String(value ?? "").trim();
  if (trimmed === "") return null;
  const normalized = trimmed.replace(/^\$/, "");
  const amount = Number(normalized);
  if (!Number.isFinite(amount) || amount < 0) return null;
  return Math.round(amount * 100);
}

export function formatPercent(value) {
  if (value == null || Number.isNaN(Number(value))) return null;
  const rounded = Math.round(Number(value));
  const sign = rounded > 0 ? "+" : "";
  return `${sign}${rounded}%`;
}

export function copyCount(details) {
  return Math.max(0, details?.quantity ?? 0) + Math.max(0, details?.foilQuantity ?? 0);
}

export function unitPriceCents(price, details = null) {
  if (!price) return null;
  if ((details?.quantity ?? 0) > 0) {
    return price.usdCents ?? price.usdFoilCents ?? price.usdEtchedCents ?? null;
  }
  if ((details?.foilQuantity ?? 0) > 0) {
    return price.usdFoilCents ?? price.usdCents ?? price.usdEtchedCents ?? null;
  }
  return price.usdCents ?? price.usdFoilCents ?? price.usdEtchedCents ?? null;
}

export function priceTrend(price, details) {
  const current = unitPriceCents(price, details);
  const baseline = details?.purchasePrice?.usdCents;
  if (current == null || baseline == null || baseline <= 0) return null;
  const changeCents = current - baseline;
  return {
    changeCents,
    changePercent: (changeCents / baseline) * 100,
    direction: changeCents > 0 ? "up" : changeCents < 0 ? "down" : "flat",
  };
}
