// Numeric comparison for display only; keep in step with aiconfig/versions.rs.
function versionKey(v: string): number[] {
  return v.replace(/^v+/, "").split(/[.\-+]/).map((part) => Number(part.match(/^\d+/)?.[0] ?? 0));
}

export function isBehind(pinned: string, latest: string): boolean {
  if (!/^\d/.test(pinned)) return false;
  const a = versionKey(pinned);
  const b = versionKey(latest);
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const x = a[i] ?? 0;
    const y = b[i] ?? 0;
    if (x !== y) return y > x;
  }
  return false;
}
