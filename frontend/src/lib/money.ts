/** Parse a human-entered EUR amount into integer cents without float arithmetic. */
export function parseAmountToCents(input: string): number | null {
  const normalized = input.trim().replace(/\s/g, '').replace(',', '.');
  if (!/^(?:0|[1-9]\d*)(?:\.\d{1,2})?$/.test(normalized)) return null;
  const [eurosPart, centsPart = ''] = normalized.split('.');
  try {
    const cents = BigInt(eurosPart) * 100n + BigInt((centsPart + '00').slice(0, 2));
    if (cents <= 0n || cents > BigInt(Number.MAX_SAFE_INTEGER)) return null;
    return Number(cents);
  } catch {
    return null;
  }
}

export function formatMoney(cents: number, currency = 'EUR'): string {
  return new Intl.NumberFormat('fr-FR', { style: 'currency', currency }).format(cents / 100);
}

export function formatDate(date: string): string {
  const parsed = new Date(`${date}T12:00:00`);
  if (Number.isNaN(parsed.getTime())) return date;
  return new Intl.DateTimeFormat('fr-FR', { dateStyle: 'medium' }).format(parsed);
}
