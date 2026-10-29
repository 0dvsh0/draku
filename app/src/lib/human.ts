export function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

/** Real age from the store, never a filename dressed up as a date. */
export function relativeAge(item: { ageDays: number; name?: string }): string {
  const days = Math.max(0, Math.floor(item.ageDays));
  if (days === 0) return "untouched since today";
  if (days === 1) return "untouched for a day";
  if (days < 30) return `untouched for ${days} days`;
  if (days < 365) {
    const months = Math.floor(days / 30);
    return `untouched for ${months} ${months === 1 ? "month" : "months"}`;
  }
  const years = (days / 365).toFixed(1);
  return `untouched for ${years} ${years === "1.0" ? "year" : "years"}`;
}

export function dayPhrase(days: number): string {
  if (days === 0) return "today";
  if (days === 1) return "tomorrow";
  if (days < 0) return `${-days} ${-days === 1 ? "day" : "days"} ago`;
  return `in ${days} days`;
}
