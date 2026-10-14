import { describe, expect, it } from 'vitest';
import { buildFluxReminderTaskInput } from './flux-reminder';

describe('buildFluxReminderTaskInput', () => {
  const flow = { id: 'flux-1', libelle: 'Loyer du local', datePrevue: '2027-01-05' };

  it('builds a one-off treasury task with a stable idempotency key and flux reference', () => {
    const input = buildFluxReminderTaskInput(flow);

    expect(input).toMatchObject({
      idempotencyKey: 'flux-reminder-flux-1',
      module: 'tresorerie',
      taskType: 'rappel_flux',
      title: 'Échéance : Loyer du local',
      payload: { fluxId: 'flux-1' },
      recurrenceFrequency: 'once',
      cronExpression: null,
      maxAttempts: 3,
    });
    expect(Number.isNaN(new Date(input.scheduledAt).getTime())).toBe(false);
    expect(new Date(input.scheduledAt).getHours()).toBe(9);
    expect(buildFluxReminderTaskInput(flow).idempotencyKey).toBe(input.idempotencyKey);
  });

  it('rejects an invalid forecast date instead of scheduling an unintended task', () => {
    expect(() => buildFluxReminderTaskInput({ ...flow, datePrevue: 'not-a-date' }))
      .toThrow('La date prévue du flux est invalide.');
  });
});
