import type { Task, TaskStatus } from './types';

export type TaskStatusFilter = TaskStatus | 'all';

/** Filter by status and accent-insensitive title/module/type without changing sort order. */
export function filterTasks(
  tasks: Task[],
  query: string,
  status: TaskStatusFilter = 'all',
): Task[] {
  const normalizedQuery = normalizeSearchText(query.trim());
  return tasks.filter((task) => {
    if (status !== 'all' && task.status !== status) return false;
    if (!normalizedQuery) return true;
    const searchableText = normalizeSearchText(
      [task.title, task.module, task.taskType].join(' '),
    );
    return searchableText.includes(normalizedQuery);
  });
}

function normalizeSearchText(value: string): string {
  return value
    .normalize('NFD')
    .replace(/\p{Diacritic}/gu, '')
    .toLocaleLowerCase('fr-FR');
}
