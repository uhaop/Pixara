export function parsePositiveDimension(raw: string): number | null {
  if (raw.trim() === "") {
    return null;
  }
  const value = Number.parseInt(raw, 10);
  if (!Number.isFinite(value) || value <= 0) {
    return null;
  }
  return value;
}
