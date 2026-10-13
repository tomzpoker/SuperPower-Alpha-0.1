import { describe, expect, it } from 'vitest';
import { formatElapsedDuration } from './task-history';

describe('formatElapsedDuration', () => {
  it('formats seconds and floors sub-second durations', () => {
    expect(formatElapsedDuration('2027-01-01T10:00:00.000Z', '2027-01-01T10:00:05.900Z')).toBe('5 s');
    expect(formatElapsedDuration('2027-01-01T10:00:00.000Z', '2027-01-01T10:00:00.900Z')).toBe('0 s');
  });

  it('formats minutes and hours without redundant zero units', () => {
    expect(formatElapsedDuration('2027-01-01T10:00:00.000Z', '2027-01-01T10:01:00.000Z')).toBe('1 min');
    expect(formatElapsedDuration('2027-01-01T10:00:00.000Z', '2027-01-01T10:02:07.000Z')).toBe('2 min 7 s');
    expect(formatElapsedDuration('2027-01-01T10:00:00.000Z', '2027-01-01T12:00:00.000Z')).toBe('2 h');
    expect(formatElapsedDuration('2027-01-01T10:00:00.000Z', '2027-01-01T11:45:00.000Z')).toBe('1 h 45 min');
  });

  it('formats durations longer than one day', () => {
    expect(formatElapsedDuration('2027-01-01T00:00:00.000Z', '2027-01-02T00:00:00.000Z')).toBe('1 j');
    expect(formatElapsedDuration('2027-01-01T00:00:00.000Z', '2027-01-03T02:00:00.000Z')).toBe('2 j 2 h');
  });

  it('does not invent a duration for running, malformed or reversed timestamps', () => {
    expect(formatElapsedDuration('2027-01-01T10:00:00.000Z', null)).toBeNull();
    expect(formatElapsedDuration(null, '2027-01-01T10:00:00.000Z')).toBeNull();
    expect(formatElapsedDuration('bad-date', '2027-01-01T10:00:00.000Z')).toBeNull();
    expect(formatElapsedDuration('2027-01-01T10:00:05.000Z', '2027-01-01T10:00:00.000Z')).toBeNull();
  });
});
