/** Format the elapsed time for a completed execution or attempt. */
export function formatElapsedDuration(
  startedAt: string | null | undefined,
  finishedAt: string | null | undefined,
): string | null {
  if (!startedAt || !finishedAt) return null;

  const start = Date.parse(startedAt);
  const finish = Date.parse(finishedAt);
  if (!Number.isFinite(start) || !Number.isFinite(finish) || finish < start) return null;

  const totalSeconds = Math.floor((finish - start) / 1_000);
  if (totalSeconds < 60) return `${totalSeconds} s`;

  const totalMinutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  if (totalMinutes < 60) return seconds ? `${totalMinutes} min ${seconds} s` : `${totalMinutes} min`;

  const totalHours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (totalHours < 24) return minutes ? `${totalHours} h ${minutes} min` : `${totalHours} h`;

  const days = Math.floor(totalHours / 24);
  const hours = totalHours % 24;
  return hours ? `${days} j ${hours} h` : `${days} j`;
}
