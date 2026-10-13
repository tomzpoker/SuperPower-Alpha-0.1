import { act, createElement } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { TaskCalendar } from './App';
import type { Task } from './lib/types';

const anchor = '2027-01-31T10:15:00.000Z';
const marchOccurrence = '2027-03-31T10:15:00.000Z';

const recurringTask: Task = {
  id: 'monthly-reminder',
  idempotencyKey: 'monthly-reminder-key',
  module: 'general',
  taskType: 'reminder',
  title: 'Rappel mensuel',
  payload: {},
  scheduledAt: anchor,
  recurrenceFrequency: 'monthly',
  cronExpression: null,
  cronAnchorAt: anchor,
  maxAttempts: 3,
  status: 'scheduled',
  createdAt: anchor,
  updatedAt: anchor,
};

describe('TaskCalendar occurrence selection', () => {
  let root: Root;
  let container: HTMLDivElement;

  beforeEach(() => {
    Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
    container = document.createElement('div');
    document.body.append(container);
    root = createRoot(container);
  });

  afterEach(() => {
    act(() => root.unmount());
    container.remove();
  });

  it('selects the exact projected occurrence and exposes its selected state accessibly', () => {
    const onSelectOccurrence = vi.fn();
    const props = {
      tasks: [recurringTask],
      month: new Date(2027, 2, 1),
      onMonthChange: vi.fn(),
      selectedOccurrence: null,
      onSelectOccurrence,
    };

    act(() => {
      root.render(createElement(TaskCalendar, props));
    });

    const marchButton = Array.from(
      container.querySelectorAll<HTMLButtonElement>('.tasks-calendar-event'),
    ).find(button => button.title.includes('31/03/2027'));
    expect(marchButton).toBeDefined();
    expect(marchButton?.tagName.toLowerCase()).toBe('button');
    expect(marchButton?.getAttribute('aria-pressed')).toBe('false');

    act(() => {
      if (!marchButton) throw new Error('L’occurrence du 31 mars devrait être affichée.');
      marchButton.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    });

    expect(onSelectOccurrence).toHaveBeenCalledTimes(1);
    expect(onSelectOccurrence).toHaveBeenCalledWith(recurringTask.id, marchOccurrence);

    act(() => {
      root.render(createElement(TaskCalendar, {
        ...props,
        selectedOccurrence: { taskId: recurringTask.id, scheduledAt: marchOccurrence },
      }));
    });

    const selectedButton = Array.from(
      container.querySelectorAll<HTMLButtonElement>('.tasks-calendar-event'),
    ).find(button => button.title.includes('31/03/2027'));
    expect(selectedButton?.getAttribute('aria-pressed')).toBe('true');
    expect(selectedButton?.getAttribute('aria-label')).toContain('Rappel mensuel');
  });
});
