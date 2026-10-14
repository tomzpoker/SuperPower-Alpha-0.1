import type { FluxView, NewTask } from './types';

/** Build an idempotent one-off reminder tied to one forecast occurrence. */
export function buildFluxReminderTaskInput(
  flow: Pick<FluxView, 'id' | 'libelle' | 'datePrevue'>,
): NewTask {
  const scheduledAt = new Date(`${flow.datePrevue}T09:00:00`);
  if (!Number.isFinite(scheduledAt.getTime())) {
    throw new Error('La date prévue du flux est invalide.');
  }

  return {
    idempotencyKey: `flux-reminder-${flow.id}`,
    module: 'tresorerie',
    taskType: 'rappel_flux',
    title: `Échéance : ${flow.libelle}`,
    payload: { fluxId: flow.id },
    scheduledAt: scheduledAt.toISOString(),
    recurrenceFrequency: 'once',
    cronExpression: null,
    maxAttempts: 3,
  };
}
