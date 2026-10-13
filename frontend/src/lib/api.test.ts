import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { CreateSocieteInput, FluxInput, FluxView, NewTask, RealisationFluxInput, Societe, Task } from './types';

type Api = typeof import('./api')['api'];
let api: Api;
let idCounter = 0;

beforeEach(async () => {
  vi.unstubAllGlobals();
  vi.resetModules();
  idCounter = 0;
  vi.stubGlobal('crypto', { randomUUID: () => 'test-id-' + (++idCounter) });
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__');
  api = (await import('./api')).api;
});

async function createCompany(): Promise<Societe> {
  const input: CreateSocieteInput = {
    nom: 'SCI de test',
    formeJuridique: 'sci',
    regimeFiscal: 'ir',
    regimeTva: 'non_renseigne',
    exigibiliteTva: 'non_definie',
    periodiciteTva: 'non_definie',
    siret: null,
    finExercice: '2026-12-31',
  };
  return api<Societe>('create_societe', { input });
}

function flowInput(societeId: string, changes: Partial<FluxInput> = {}): FluxInput {
  return {
    societeId,
    libelle: 'Loyer test',
    sens: 'encaissement',
    categorie: 'loyer',
    datePrevue: '2026-12-01',
    montantPrevuCents: 10_000,
    tiers: null,
    reference: null,
    frequenceRecurrence: 'ponctuel',
    dateFinRecurrence: null,
    ...changes,
  };
}

describe('browser demo cashflow behaviour', () => {
  it('rejects impossible ISO dates for forecasts and realizations', async () => {
    const company = await createCompany();

    await expect(
      api<FluxView>('create_flux', { input: flowInput(company.id, { datePrevue: '2026-02-30' }) }),
    ).rejects.toThrow('date prévue');

    const flow = await api<FluxView>('create_flux', { input: flowInput(company.id) });
    const invalidActual: RealisationFluxInput = {
      dateRealisation: '2026-02-30',
      montantCents: 2_000,
      reference: null,
      note: null,
    };
    await expect(
      api<FluxView>('register_flux_realisation', { fluxId: flow.id, input: invalidActual }),
    ).rejects.toThrow('date de réalisation');
  });

  it('does not allow deleting a company that has a cancelled forecast', async () => {
    const company = await createCompany();
    const flow = await api<FluxView>('create_flux', { input: flowInput(company.id) });
    await api<FluxView>('cancel_flux', { id: flow.id });

    await expect(api<void>('delete_societe', { id: company.id })).rejects.toThrow('flux prévisionnels');

    const companies = await api<Societe[]>('list_societes');
    expect(companies.map((item) => item.id)).toContain(company.id);
  });

  it('normalizes partial realizations and preserves cancelled history without counting it', async () => {
    const company = await createCompany();
    const flow = await api<FluxView>('create_flux', {
      input: flowInput(company.id, { libelle: '  Loyer octobre  ', tiers: '  Locataire  ', reference: '  PREV-01  ' }),
    });
    expect(flow.libelle).toBe('Loyer octobre');
    expect(flow.tiers).toBe('Locataire');
    expect(flow.reference).toBe('PREV-01');

    const input: RealisationFluxInput = {
      dateRealisation: '2026-10-13',
      montantCents: 4_000,
      reference: '  VRT-01  ',
      note: '  Acompte  ',
    };
    const afterActual = await api<FluxView>('register_flux_realisation', { fluxId: flow.id, input });
    expect(afterActual.montantRealiseCents).toBe(4_000);
    expect(afterActual.realisations[0].reference).toBe('VRT-01');
    expect(afterActual.realisations[0].note).toBe('Acompte');

    const actualId = afterActual.realisations[0].id;
    const afterCancel = await api<FluxView>('cancel_flux_realisation', {
      id: actualId,
      motifAnnulation: 'Erreur de saisie',
    });
    expect(afterCancel.montantRealiseCents).toBe(0);
    expect(afterCancel.realisations[0].motifAnnulation).toBe('Erreur de saisie');

    const listed = await api<FluxView[]>('list_flux');
    expect(listed[0].montantRealiseCents).toBe(0);
    expect(listed[0].realisations[0].annuleeAt).toBeTruthy();
  });

  it('generates monthly end-of-month occurrences without calendar drift', async () => {
    const company = await createCompany();
    const first = await api<FluxView>('create_flux', {
      input: flowInput(company.id, {
        datePrevue: '2026-01-31',
        frequenceRecurrence: 'mensuel',
        dateFinRecurrence: '2026-04-30',
      }),
    });
    const listed = await api<FluxView[]>('list_flux');
    const series = listed
      .filter(flow => flow.recurrenceId === first.recurrenceId)
      .sort((a, b) => a.datePrevue.localeCompare(b.datePrevue));
    expect(series.map(flow => flow.datePrevue)).toEqual([
      '2026-01-31', '2026-02-28', '2026-03-31', '2026-04-30',
    ]);
    expect(new Set(series.map(flow => flow.id)).size).toBe(4);
  });

  it('stops a recurrence without deleting later scheduled occurrences', async () => {
    const company = await createCompany();
    const first = await api<FluxView>('create_flux', {
      input: flowInput(company.id, {
        datePrevue: '2026-10-13',
        frequenceRecurrence: 'mensuel',
        dateFinRecurrence: '2027-02-13',
      }),
    });
    await api<void>('arreter_flux_recurrence', {
      recurrenceId: first.recurrenceId,
      dateFin: '2026-11-13',
    });
    const listed = await api<FluxView[]>('list_flux');
    const series = listed
      .filter(flow => flow.recurrenceId === first.recurrenceId)
      .sort((a, b) => a.datePrevue.localeCompare(b.datePrevue));
    expect(series).toHaveLength(5);
    expect(series.every(flow => !flow.recurrenceActive)).toBe(true);
    expect(series.filter(flow => flow.datePrevue <= '2026-11-13').every(flow => flow.statut === 'actif')).toBe(true);
    expect(series.filter(flow => flow.datePrevue > '2026-11-13').every(flow => flow.statut === 'annule')).toBe(true);
  });

  it('refuses a retroactive stop that would cancel an occurrence with an active realization', async () => {
    const company = await createCompany();
    const first = await api<FluxView>('create_flux', {
      input: flowInput(company.id, {
        datePrevue: '2026-10-13',
        frequenceRecurrence: 'mensuel',
        dateFinRecurrence: '2027-02-13',
      }),
    });
    const listed = await api<FluxView[]>('list_flux');
    const later = listed.find(flow => flow.recurrenceId === first.recurrenceId && flow.datePrevue === '2026-12-13');
    if (!later) throw new Error('Occurrence de décembre manquante dans la série de test.');
    await api<FluxView>('register_flux_realisation', {
      fluxId: later.id,
      input: {
        dateRealisation: '2026-12-13',
        montantCents: 10_000,
        reference: 'PAY-TEST',
        note: null,
      },
    });
    await expect(api<void>('arreter_flux_recurrence', {
      recurrenceId: first.recurrenceId,
      dateFin: '2026-11-13',
    })).rejects.toThrow('réalisation active');
    const after = await api<FluxView[]>('list_flux');
    const preserved = after.find(flow => flow.id === later.id);
    expect(preserved?.statut).toBe('actif');
    expect(preserved?.montantRealiseCents).toBe(10_000);
    expect(after.filter(flow => flow.recurrenceId === first.recurrenceId).every(flow => flow.recurrenceActive)).toBe(true);
  });

  it('only stops a recurrence on an existing occurrence within the visible horizon', async () => {
    const company = await createCompany();
    const first = await api<FluxView>('create_flux', {
      input: flowInput(company.id, {
        datePrevue: '2026-10-13',
        frequenceRecurrence: 'mensuel',
        dateFinRecurrence: '2027-02-13',
      }),
    });
    await expect(api<void>('arreter_flux_recurrence', {
      recurrenceId: first.recurrenceId,
      dateFin: '2026-11-12',
    })).rejects.toThrow('correspondre à une échéance réelle');
    const listed = await api<FluxView[]>('list_flux');
    const series = listed.filter(flow => flow.recurrenceId === first.recurrenceId);
    expect(series.every(flow => flow.recurrenceActive)).toBe(true);
    expect(series.every(flow => flow.statut === 'actif')).toBe(true);
  });

  it('rejects end dates for one-off forecasts and end dates before the first occurrence', async () => {
    const company = await createCompany();
    await expect(api<FluxView>('create_flux', {
      input: flowInput(company.id, { dateFinRecurrence: '2026-12-31' }),
    })).rejects.toThrow('date de fin');
    await expect(api<FluxView>('create_flux', {
      input: flowInput(company.id, {
        frequenceRecurrence: 'mensuel',
        dateFinRecurrence: '2026-11-30',
      }),
    })).rejects.toThrow('ne peut pas précéder');
  });

  it('deletes a task from the browser demo when it has no execution history', async () => {
    const input: NewTask = {
      idempotencyKey: 'delete-demo-task',
      module: 'general',
      taskType: 'manual',
      title: 'Tâche de test supprimable',
      payload: { test: true },
      scheduledAt: '2026-12-01T10:00:00.000Z',
      cronExpression: null,
      maxAttempts: 3,
    };
    const task = await api<Task>('create_task', { input });
    expect((await api<Task[]>('list_tasks')).some(item => item.id === task.id)).toBe(true);

    await api<void>('delete_task', { taskId: task.id });

    expect((await api<Task[]>('list_tasks')).some(item => item.id === task.id)).toBe(false);
    await expect(api<void>('delete_task', { taskId: task.id }))
      .rejects.toThrow('Cette tâche n’existe plus.');
  });

  it('supports pause, resume, cancel and reactivate without duplicating a task', async () => {
    const input: NewTask = {
      idempotencyKey: 'lifecycle-demo-task',
      module: 'general',
      taskType: 'reminder',
      title: 'Cycle de vie de test',
      payload: {},
      scheduledAt: '2027-01-31T10:00:00.000Z',
      recurrenceFrequency: 'monthly',
      cronExpression: null,
      maxAttempts: 3,
    };

    const created = await api<Task>('create_task', { input });
    const paused = await api<Task>('update_task_status', { taskId: created.id, status: 'paused' });
    const resumed = await api<Task>('update_task_status', { taskId: created.id, status: 'scheduled' });
    const cancelled = await api<Task>('update_task_status', { taskId: created.id, status: 'cancelled' });
    const reactivated = await api<Task>('update_task_status', { taskId: created.id, status: 'scheduled' });

    expect([paused.status, resumed.status, cancelled.status, reactivated.status]).toEqual([
      'paused', 'scheduled', 'cancelled', 'scheduled',
    ]);
    expect([paused.id, resumed.id, cancelled.id, reactivated.id]).toEqual([
      created.id, created.id, created.id, created.id,
    ]);
    const listed = await api<Task[]>('list_tasks');
    expect(listed.filter(item => item.id === created.id)).toHaveLength(1);
  });

  it('rejects values that exceed the native field limits', async () => {
    const company = await createCompany();
    await expect(
      api<FluxView>('create_flux', { input: flowInput(company.id, { libelle: 'x'.repeat(161) }) }),
    ).rejects.toThrow('160 caractères');

    const flow = await api<FluxView>('create_flux', { input: flowInput(company.id) });
    const invalidActual: RealisationFluxInput = {
      dateRealisation: '2026-10-13',
      montantCents: 1_000,
      reference: null,
      note: 'x'.repeat(501),
    };
    await expect(
      api<FluxView>('register_flux_realisation', { fluxId: flow.id, input: invalidActual }),
    ).rejects.toThrow('500 caractères');
  });
});
