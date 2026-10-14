import type { Task, TaskRecurrenceFrequency } from './types';

const recurrenceIntervalMonths: Partial<Record<TaskRecurrenceFrequency, number>> = {
  monthly: 1,
  quarterly: 3,
  yearly: 12,
};

/**
 * Return the task occurrences whose local-calendar dates intersect the visible
 * grid. Business recurrences always derive from their original UTC anchor, not
 * from the clipped previous occurrence (so Jan 31 -> Feb 28 -> Mar 31).
 */
export function taskCalendarOccurrenceDates(
  task: Task,
  rangeStart: Date,
  rangeEndExclusive: Date,
): string[] {
  const scheduledAt = Date.parse(task.scheduledAt);
  const rangeStartAt = rangeStart.getTime();
  const rangeEndAt = rangeEndExclusive.getTime();
  if (
    !Number.isFinite(scheduledAt)
    || !Number.isFinite(rangeStartAt)
    || !Number.isFinite(rangeEndAt)
    || rangeStartAt >= rangeEndAt
  ) {
    return [];
  }

  const singleOccurrence = (): string[] =>
    scheduledAt >= rangeStartAt && scheduledAt < rangeEndAt
      ? [new Date(scheduledAt).toISOString()]
      : [];

  const frequency = task.recurrenceFrequency
    ?? (task.cronExpression ? 'custom_cron' : 'once');
  const intervalMonths = recurrenceIntervalMonths[frequency];

  // A paused or cancelled definition must not paint future hypothetical dates.
  // Cron is parsed by the Rust engine and is intentionally not reimplemented in JS.
  if (task.status !== 'scheduled' || !intervalMonths) return singleOccurrence();
  if (scheduledAt >= rangeEndAt) return [];

  const anchorAt = Date.parse(task.cronAnchorAt ?? task.scheduledAt);
  if (!Number.isFinite(anchorAt)) return singleOccurrence();
  const anchor = new Date(anchorAt);

  const monthsSinceGridStart =
    (rangeStart.getFullYear() - anchor.getUTCFullYear()) * 12
    + (rangeStart.getMonth() - anchor.getUTCMonth());
  let occurrenceIndex = Math.max(0, Math.floor(monthsSinceGridStart / intervalMonths) - 1);
  const occurrences: string[] = [];

  // The index starts near the visible range, so this guard is only a safety net
  // against malformed or out-of-range timestamps, not a recurrence horizon.
  for (let generated = 0; generated < 120; generated += 1) {
    const occurrence = anchoredMonthOccurrence(anchor, occurrenceIndex * intervalMonths);
    if (!occurrence || occurrence.getTime() >= rangeEndAt) break;

    const timestamp = occurrence.getTime();
    if (timestamp >= scheduledAt && timestamp >= rangeStartAt) {
      occurrences.push(occurrence.toISOString());
    }
    occurrenceIndex += 1;
  }

  return occurrences;
}

function anchoredMonthOccurrence(anchor: Date, monthOffset: number): Date | null {
  const firstOfTargetMonth = new Date(Date.UTC(
    anchor.getUTCFullYear(),
    anchor.getUTCMonth() + monthOffset,
    1,
  ));
  const year = firstOfTargetMonth.getUTCFullYear();
  const month = firstOfTargetMonth.getUTCMonth();
  const lastDayOfMonth = new Date(Date.UTC(year, month + 1, 0)).getUTCDate();
  const day = Math.min(anchor.getUTCDate(), lastDayOfMonth);
  const occurrence = new Date(Date.UTC(
    year,
    month,
    day,
    anchor.getUTCHours(),
    anchor.getUTCMinutes(),
    anchor.getUTCSeconds(),
    anchor.getUTCMilliseconds(),
  ));
  return Number.isFinite(occurrence.getTime()) ? occurrence : null;
}
