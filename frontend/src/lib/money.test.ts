import { describe, expect, it } from 'vitest';
import { formatMoney, parseAmountToCents } from './money';

describe('parseAmountToCents', () => {
  it('parses decimal commas and points without losing cents', () => {
    expect(parseAmountToCents('123,45')).toBe(12345);
    expect(parseAmountToCents('123.4')).toBe(12340);
  });
  it('rejects ambiguous, negative, zero and over-precise amounts', () => {
    expect(parseAmountToCents('-1.00')).toBeNull();
    expect(parseAmountToCents('0')).toBeNull();
    expect(parseAmountToCents('1.234')).toBeNull();
    expect(parseAmountToCents('1,2,3')).toBeNull();
  });
  it('formats a value in French currency style', () => {
    expect(formatMoney(12345)).toContain('123,45');
  });
});
