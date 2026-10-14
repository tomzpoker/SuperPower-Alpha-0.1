export type ReminderPayload = Record<string, unknown>;

function isJsonObject(value: unknown): value is ReminderPayload {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/** Read the optional message used by the local general/reminder handler. */
export function readReminderMessage(payload: unknown): string {
  if (!isJsonObject(payload) || typeof payload.message !== 'string') return '';
  return payload.message.trim();
}

/**
 * Merge the form's message into an existing JSON object without dropping other
 * task payload fields. A blank message removes the optional property.
 */
export function withReminderMessage(payload: unknown, message: string): ReminderPayload {
  if (!isJsonObject(payload)) {
    throw new Error('Les données JSON d’un rappel doivent être un objet.');
  }

  const next = { ...payload };
  const cleaned = message.trim();
  if (cleaned) next.message = cleaned;
  else delete next.message;
  return next;
}
