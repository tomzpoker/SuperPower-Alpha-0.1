import { describe, expect, it } from 'vitest';
import type { Task, TaskRecurrenceFrequency } from './types';
import { taskCalendarOccurrenceDates } from './task-calendar';

function task(
  frequency: TaskRecurrenceFrequency,
  changes: Partial<Task> = {},
): Task {
  const anchor = '2027-01-31T10:15:00.000Z';
  return {
    id: 'calendar-task',
    idempotencyKey: 'calendar-task-key',
    module: 'general',
    taskType: 'reminder',
    title: 'Rappel mensuel',
    payload: {},
    scheduledAt: anchor,
    recurrenceFrequency: frequency,
    cronExpression: frequency === 'custom_cron' ? '0 10 * * 1-5' : null,
    cronAnchorAt: anchor,
    maxAttempts: 3,
    status: 'scheduled',
    createdAt: anchor,
    updatedAt: anchor,
    ...changes,
  };
}

function range(start: string, endExclusive: string): [Date, Date] {
  return [new Date(start), new Date(endExclusive)];
}

describe('task calendar recurrence projection', () => {
  it('projects monthly dates from the original month-end anchor without drift', () => {
    const [start, end] = range('2027-02-01T00:00:00.000Z', '2027-04-01T00:00:00.000Z');

    expect(taskCalendarOccurrenceDates(task('monthly'), start, end)).toEqual([
      '2027-02-28T10:15:00.000Z',
      '2027-03-31T10:15:00.000Z',
    ]);
  });

  it('projects quarterly occurrences while preserving the anchor day', () => {
    const [start, end] = range('2027-04-01T00:00:00.000Z', '2027-07-01T00:00:00.000Z');

    expect(taskCalendarOccurrenceDates(task('quarterly'), start, end)).toEqual([
      '2027-04-30T10:15:00.000Z',
    ]);
  });

  it('keeps a leap-day yearly anchor across non-leap years', () => {
    const anchor = '2024-02-29T10:15:00.000Z';
    const [start, end] = range('2025-01-01T00:00:00.000Z', '2029-01-01T00:00:00.000Z');
    const yearly = task('yearly', { scheduledAt: anchor, cronAnchorAt: anchor });

    expect(taskCalendarOccurrenceDates(yearly, start, end)).toEqual([
      '2025-02-28T10:15:00.000Z',
      '2026-02-28T10:15:00.000Z',
      '2027-02-28T10:15:00.000Z',
      '2028-02-29T10:15:00.000Z',
    ]);
  });

  it('does not show old occurrences before the current schedule cursor', () => {
    const [start, end] = range('2027-01-01T00:00:00.000Z', '2027-04-01T00:00:00.000Z');
    const recurring = task('monthly', { scheduledAt: '2027-03-31T10:15:00.000Z' });

    expect(taskCalendarOccurrenceDates(recurring, start, end)).toEqual([
      '2027-03-31T10:15:00.000Z',
    ]);
  });

  it('does not project future instances for paused or cancelled tasks', () => {
    const [start, end] = range('2027-02-01T00:00:00.000Z', '2027-04-01T00:00:00.000Z');

    expect(taskCalendarOccurrenceDates(task('monthly', {
      scheduledAt: '2027-02-28T10:15:00.000Z',
      status: 'paused',
    }), start, end)).toEqual(['2027-02-28T10:15:00.000Z']);

    expect(taskCalendarOccurrenceDates(task('monthly', {
      scheduledAt: '2027-02-28T10:15:00.000Z',
      status: 'cancelled',
    }), start, end)).toEqual(['2027-02-28T10:15:00.000Z']);
  });

  it('shows only the current due date for custom cron and one-off tasks', () => {
    const [start, end] = range('2027-02-01T00:00:00.000Z', '2027-04-01T00:00:00.000Z');
    const cron = task('custom_cron', {
      scheduledAt: '2027-02-01T10:15:00.000Z',
      cronAnchorAt: '2027-01-31T10:15:00.000Z',
    });
    const once = task('once', { scheduledAt: '2027-03-20T10:15:00.000Z' });

    expect(taskCalendarOccurrenceDates(cron, start, end)).toEqual(['2027-02-01T10:15:00.000Z']);
    expect(taskCalendarOccurrenceDates(once, start, end)).toEqual(['2027-03-20T10:15:00.000Z']);
  });
});
