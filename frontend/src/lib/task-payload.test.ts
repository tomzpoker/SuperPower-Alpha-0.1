import { describe, expect, it } from 'vitest';
import { readReminderMessage, withReminderMessage } from './task-payload';

describe('reminder task payload helpers', () => {
  it('reads and trims an optional message', () => {
    expect(readReminderMessage({ message: '  Relancer le locataire  ' })).toBe('Relancer le locataire');
    expect(readReminderMessage({ message: 42 })).toBe('');
    expect(readReminderMessage({})).toBe('');
    expect(readReminderMessage(null)).toBe('');
  });

  it('merges the message without losing other payload fields', () => {
    expect(withReminderMessage({ ref: 'BAIL-01', flags: ['urgent'] }, '  Appeler le locataire  '))
      .toEqual({ ref: 'BAIL-01', flags: ['urgent'], message: 'Appeler le locataire' });
  });

  it('removes the optional message when the field is blank', () => {
    expect(withReminderMessage({ message: 'Ancien texte', ref: 'BAIL-01' }, '   '))
      .toEqual({ ref: 'BAIL-01' });
  });

  it.each([null, [], 'texte', 42])('rejects a non-object reminder payload (%s)', (payload) => {
    expect(() => withReminderMessage(payload, 'Message')).toThrow('doivent être un objet');
  });
});
