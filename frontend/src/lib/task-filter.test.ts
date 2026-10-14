import { describe, expect, it } from 'vitest';
import type { Task, TaskStatus } from './types';
import { filterTasks } from './task-filter';

function task(id: string, title: string, module: string, taskType: string, status: TaskStatus = 'scheduled'): Task {
  const date = '2027-01-31T10:00:00.000Z';
  return {
    id,
    idempotencyKey: id,
    module,
    taskType,
    title,
    payload: {},
    scheduledAt: date,
    recurrenceFrequency: 'once',
    cronExpression: null,
    cronAnchorAt: date,
    maxAttempts: 3,
    status,
    createdAt: date,
    updatedAt: date,
  };
}

const tasks = [
  task('1', 'Vérifier échéance', 'locatif', 'verifier_bail'),
  task('2', 'Rappel facture', 'documents', 'reminder', 'paused'),
  task('3', 'Importer relevé', 'import_bancaire', 'import_csv', 'cancelled'),
];

describe('task list filtering', () => {
  it('searches titles without case or accent sensitivity', () => {
    expect(filterTasks(tasks, 'VERIFIER ECHEANCE').map(item => item.id)).toEqual(['1']);
  });

  it('searches module and technical type', () => {
    expect(filterTasks(tasks, 'locatif').map(item => item.id)).toEqual(['1']);
    expect(filterTasks(tasks, 'reminder').map(item => item.id)).toEqual(['2']);
    expect(filterTasks(tasks, 'import_bancaire').map(item => item.id)).toEqual(['3']);
  });

  it('filters by each lifecycle status', () => {
    expect(filterTasks(tasks, '', 'scheduled').map(item => item.id)).toEqual(['1']);
    expect(filterTasks(tasks, '', 'paused').map(item => item.id)).toEqual(['2']);
    expect(filterTasks(tasks, '', 'cancelled').map(item => item.id)).toEqual(['3']);
  });

  it('combines the search and status filters without changing original order', () => {
    const rows = [
      task('a', 'Rappel paiement', 'general', 'reminder'),
      task('b', 'Rappel bail', 'locatif', 'reminder', 'paused'),
      task('c', 'Rappel assurance', 'general', 'reminder'),
    ];
    expect(filterTasks(rows, 'rappel', 'scheduled').map(item => item.id)).toEqual(['a', 'c']);
  });

  it('ignores surrounding whitespace and returns no rows for no match', () => {
    expect(filterTasks(tasks, '  facture  ').map(item => item.id)).toEqual(['2']);
    expect(filterTasks(tasks, 'non existent')).toEqual([]);
  });

  it('returns all tasks for an empty query and the all-status option', () => {
    expect(filterTasks(tasks, '   ', 'all')).toEqual(tasks);
  });
});
