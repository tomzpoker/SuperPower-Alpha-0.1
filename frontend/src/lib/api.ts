import { invoke } from '@tauri-apps/api/core';
import type {
  Account, CreateEntryInput, CreateSocieteInput, DatabaseStatus,
  EntryLine, EntryView, FluxInput, FluxRealisationView, FluxView, FrequenceFlux,
  NewTask, RealisationFluxInput, Societe, Task, TaskAttempt, TaskExecution, TaskStatus,
} from './types';

// In the browser, only an in-memory preview is provided. Real data belongs in the
// native Tauri shell, where the database is encrypted. This demo resets on reload.
const isDesktop = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
let demoReady = false;
let demoSocietes: Societe[] = [];
let demoEntries: EntryView[] = [];
let demoFlux: FluxView[] = [];
let demoTasks: Task[] = [];
let demoTaskExecutions: TaskExecution[] = [];
let demoTaskAttempts: TaskAttempt[] = [];
const demoAccounts: Account[] = [
  ['101', 'Capital', 1], ['164', 'Emprunts auprès des établissements de crédit', 1],
  ['213', 'Constructions', 2], ['2813', 'Amortissements des constructions', 2],
  ['401', 'Fournisseurs', 4], ['411', 'Clients', 4], ['44566', 'TVA déductible sur autres biens et services', 4],
  ['44571', 'TVA collectée', 4], ['471', "Compte d'attente", 4], ['512', 'Banque', 5], ['530', 'Caisse', 5],
  ['606', 'Achats non stockés de matières et fournitures', 6], ['613', 'Locations', 6], ['616', "Primes d'assurances", 6],
  ['622', 'Honoraires', 6], ['63512', 'Taxes foncières', 6], ['661', "Charges d'intérêts", 6],
  ['706', 'Prestations de services', 7], ['708', 'Produits des activités annexes', 7], ['758', 'Produits divers de gestion courante', 7],
].map(([code, label, accountClass]) => ({ code: String(code), label: String(label), accountClass: Number(accountClass) }));


function demoToday(): string {
  const now = new Date();
  return new Date(now.getTime() - now.getTimezoneOffset() * 60_000).toISOString().slice(0, 10);
}
function summarizeFlux(flow: FluxView): FluxView {
  const actual = flow.realisations.reduce((sum, item) => sum + (item.annuleeAt ? 0 : item.montantCents), 0);
  let etat: FluxView['etat'];
  if (flow.statut === 'annule') etat = 'annule';
  else if (actual > flow.montantPrevuCents) etat = 'depasse';
  else if (actual === flow.montantPrevuCents) etat = 'realise';
  else if (actual > 0) etat = 'partiellement_realise';
  else if (flow.datePrevue < demoToday()) etat = 'en_retard';
  else etat = 'a_prevoir';
  return { ...flow, montantRealiseCents: actual, resteCents: flow.montantPrevuCents - actual, etat };
}

function demoStatus(): DatabaseStatus {
  return demoReady ? 'ready' : 'needs_setup';
}

function demoCompany(input: CreateSocieteInput): Societe {
  const warnings: string[] = [];
  if (input.regimeFiscal === 'non_renseigne') warnings.push("Le régime d'imposition du résultat n'est pas encore renseigné.");
  if (input.regimeTva === 'non_renseigne') warnings.push('Le régime de TVA reste à confirmer pour cette société.');
  if (['reel_normal', 'reel_simplifie'].includes(input.regimeTva) && input.exigibiliteTva === 'non_definie') {
    warnings.push("Le mode d'exigibilité de la TVA reste à renseigner.");
  }
  return { ...input, id: crypto.randomUUID(), siret: input.siret || null, createdAt: new Date().toISOString(), warnings };
}


function isValidIsoDate(value: unknown): value is string {
  if (typeof value !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const parsed = new Date(value + 'T00:00:00.000Z');
  return !Number.isNaN(parsed.getTime()) && parsed.toISOString().slice(0, 10) === value;
}

function demoAddMonthsAnchored(start: string, months: number): string {
  const [startYear, startMonth, startDay] = start.split('-').map(Number);
  const absoluteMonth = startYear * 12 + startMonth - 1 + months;
  const year = Math.floor(absoluteMonth / 12);
  const month = absoluteMonth % 12 + 1;
  const lastDay = new Date(Date.UTC(year, month, 0)).getUTCDate();
  const day = Math.min(startDay, lastDay);
  return [year, String(month).padStart(2, '0'), String(day).padStart(2, '0')].join('-');
}

function demoRecurrenceDates(input: FluxInput): string[] {
  const frequency = input.frequenceRecurrence ?? 'ponctuel';
  if (frequency === 'ponctuel') return [input.datePrevue];
  const monthStep: Record<Exclude<FrequenceFlux, 'ponctuel'>, number> = {
    mensuel: 1, trimestriel: 3, annuel: 12,
  };
  const horizon = demoAddMonthsAnchored(demoToday(), 36);
  // Conserver la première échéance demandée même si elle est encore hors fenêtre.
  // Le mode démo est volatil ; en natif, les échéances suivantes seront matérialisées
  // à mesure que la fenêtre glissante avance.
  if (input.datePrevue > horizon) return [input.datePrevue];
  const bound = input.dateFinRecurrence && input.dateFinRecurrence < horizon
    ? input.dateFinRecurrence
    : horizon;
  const step = monthStep[frequency];
  const dates: string[] = [];
  for (let index = 0; index < 1200; index += 1) {
    const occurrence = demoAddMonthsAnchored(input.datePrevue, index * step);
    if (occurrence > bound) return dates;
    dates.push(occurrence);
  }
  throw new Error('La récurrence dépasse la limite de 1 200 échéances.');
}

function demoOccurrence(
  input: FluxInput,
  datePrevue: string,
  recurrenceId: string | null,
  now: string,
  existing?: FluxView,
): FluxView {
  return summarizeFlux({
    ...input,
    ...(existing ?? {}),
    ...input,
    id: existing?.id ?? crypto.randomUUID(),
    datePrevue,
    recurrenceId,
    recurrenceActive: recurrenceId !== null,
    statut: existing?.statut ?? 'actif',
    etat: existing?.etat ?? 'a_prevoir',
    montantRealiseCents: existing?.montantRealiseCents ?? 0,
    resteCents: existing?.resteCents ?? input.montantPrevuCents,
    realisations: existing?.realisations ?? [],
    createdAt: existing?.createdAt ?? now,
    updatedAt: now,
  });
}

function textLength(value: string): number {
  return Array.from(value).length;
}

function cleanDemoOptional(value: string | null | undefined): string | null {
  const cleaned = value?.trim() ?? '';
  return cleaned || null;
}

function validateDemoFluxInput(input: FluxInput): void {
  if (!input.societeId?.trim()) throw new Error('La société du flux est obligatoire.');
  if (!input.libelle?.trim()) throw new Error('Le libellé du flux est obligatoire.');
  if (textLength(input.libelle.trim()) > 160) throw new Error('Le libellé du flux ne peut pas dépasser 160 caractères.');
  if (!isValidIsoDate(input.datePrevue)) throw new Error('La date prévue doit être au format AAAA-MM-JJ.');
  if (!Number.isSafeInteger(input.montantPrevuCents) || input.montantPrevuCents <= 0) {
    throw new Error('Le montant prévu doit être strictement positif.');
  }
  if (textLength(input.tiers?.trim() ?? '') > 160) throw new Error('Le tiers ne peut pas dépasser 160 caractères.');
  if (textLength(input.reference?.trim() ?? '') > 100) throw new Error('La référence ne peut pas dépasser 100 caractères.');
  if (!['encaissement', 'decaissement'].includes(input.sens)) throw new Error('Le sens du flux est invalide.');
  if (!['loyer', 'charge_courante', 'travaux', 'impot_taxe', 'emprunt', 'apport_associe', 'autre'].includes(input.categorie)) {
    throw new Error('La catégorie du flux est invalide.');
  }
  const frequency = input.frequenceRecurrence ?? 'ponctuel';
  if (!['ponctuel', 'mensuel', 'trimestriel', 'annuel'].includes(frequency)) {
    throw new Error('La fréquence de récurrence est invalide.');
  }
  if (input.dateFinRecurrence != null && input.dateFinRecurrence !== '') {
    if (frequency === 'ponctuel') throw new Error('Une date de fin ne s’applique pas à une prévision ponctuelle.');
    if (!isValidIsoDate(input.dateFinRecurrence)) throw new Error('La date de fin de récurrence doit être au format AAAA-MM-JJ.');
    if (input.dateFinRecurrence < input.datePrevue) throw new Error('La date de fin ne peut pas précéder la première échéance.');
  }
}

function normalizeDemoFluxInput(input: FluxInput): FluxInput {
  return {
    ...input,
    societeId: input.societeId.trim(),
    libelle: input.libelle.trim(),
    tiers: cleanDemoOptional(input.tiers),
    reference: cleanDemoOptional(input.reference),
    frequenceRecurrence: input.frequenceRecurrence ?? 'ponctuel',
    dateFinRecurrence: cleanDemoOptional(input.dateFinRecurrence),
  };
}

function validateDemoRealisationInput(input: RealisationFluxInput): void {
  if (!isValidIsoDate(input.dateRealisation)) throw new Error('La date de réalisation doit être au format AAAA-MM-JJ.');
  if (!Number.isSafeInteger(input.montantCents) || input.montantCents <= 0) {
    throw new Error('Le montant réalisé doit être strictement positif.');
  }
  if (textLength(input.reference?.trim() ?? '') > 100) throw new Error('La référence ne peut pas dépasser 100 caractères.');
  if (textLength(input.note?.trim() ?? '') > 500) throw new Error('La note ne peut pas dépasser 500 caractères.');
}

async function browserCommand<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  switch (command) {
    case 'database_status': return demoStatus() as T;
    case 'initialize_database':
      demoReady = true;
      return undefined as T;
    case 'unlock_database':
      demoReady = true;
      return undefined as T;
    case 'lock_database':
      demoReady = false;
      return undefined as T;
    case 'list_societes': return [...demoSocietes] as T;
    case 'create_societe': {
      const company = demoCompany(args.input as CreateSocieteInput);
      demoSocietes = [company, ...demoSocietes];
      return company as T;
    }
    case 'update_societe': {
      const id = args.id as string;
      const index = demoSocietes.findIndex((company) => company.id === id);
      if (index < 0) throw new Error('Cette société n’existe plus.');
      const previous = demoSocietes[index];
      const replacement = demoCompany(args.input as CreateSocieteInput);
      const updated: Societe = { ...replacement, id, createdAt: previous.createdAt };
      demoSocietes = demoSocietes.map((company) => company.id === id ? updated : company);
      return updated as T;
    }
    case 'delete_societe': {
      const id = args.id as string;
      if (!demoSocietes.some((company) => company.id === id)) throw new Error('Cette société n’existe plus.');
      if (demoEntries.some((entry) => entry.societeId === id)) {
        throw new Error('Suppression impossible : des écritures comptables sont liées à cette société.');
      }
      if (demoFlux.some((flow) => flow.societeId === id)) {
        throw new Error("Suppression refusée : cette société possède des flux prévisionnels, même annulés. Conservez l’historique.");
      }
      demoSocietes = demoSocietes.filter((company) => company.id !== id);
      return undefined as T;
    }
    case 'list_flux': {
      const filter = args.societeId as string | null | undefined;
      return demoFlux.filter(flow => !filter || flow.societeId === filter).map(summarizeFlux) as T;
    }
    case 'create_flux': {
      const rawInput = args.input as FluxInput;
      if (!demoSocietes.some(company => company.id === rawInput.societeId)) throw new Error('La société sélectionnée n’existe pas pour ce flux.');
      validateDemoFluxInput(rawInput);
      const input = normalizeDemoFluxInput(rawInput);
      const now = new Date().toISOString();
      const recurrenceId = input.frequenceRecurrence === 'ponctuel' ? null : crypto.randomUUID();
      const occurrences = demoRecurrenceDates(input).map(date => demoOccurrence(input, date, recurrenceId, now));
      demoFlux = [...occurrences, ...demoFlux];
      return (occurrences.find(flow => flow.datePrevue === input.datePrevue) ?? occurrences[0]) as T;
    }
    case 'update_flux': {
      const id = args.id as string, rawInput = args.input as FluxInput;
      const current = demoFlux.find(flow => flow.id === id);
      if (!current) throw new Error('Ce flux n’existe plus.');
      if (current.statut !== 'actif') throw new Error('Un flux annulé ne peut pas être modifié.');
      if (current.societeId !== rawInput.societeId) throw new Error('La société d’un flux existant ne peut pas être changée.');
      validateDemoFluxInput(rawInput);
      const input = normalizeDemoFluxInput(rawInput);
      if (current.recurrenceId) {
        if (input.frequenceRecurrence !== current.frequenceRecurrence || input.dateFinRecurrence !== current.dateFinRecurrence) {
          throw new Error('La fréquence d’une série existante ne se modifie pas depuis une occurrence. Arrêtez la série puis créez une nouvelle prévision récurrente.');
        }
        if (input.datePrevue !== current.datePrevue) throw new Error('La date d’une échéance récurrente est liée au calendrier de sa série ; elle ne peut pas être déplacée individuellement.');
        const updated = summarizeFlux({ ...current, ...input, recurrenceId: current.recurrenceId, recurrenceActive: current.recurrenceActive, updatedAt: new Date().toISOString() });
        demoFlux = demoFlux.map(flow => flow.id === id ? updated : flow);
        return updated as T;
      }
      if (input.frequenceRecurrence !== 'ponctuel') {
        const recurrenceId = crypto.randomUUID();
        const now = new Date().toISOString();
        const occurrenceDates = demoRecurrenceDates(input);
        const occurrences = occurrenceDates.map(date => date === input.datePrevue
          ? demoOccurrence(input, date, recurrenceId, now, current)
          : demoOccurrence(input, date, recurrenceId, now));
        demoFlux = [...occurrences, ...demoFlux.filter(flow => flow.id !== id)];
        return (occurrences.find(flow => flow.id === id) ?? occurrences[0]) as T;
      }
      const updated = summarizeFlux({ ...current, ...input, recurrenceId: null, recurrenceActive: false, updatedAt: new Date().toISOString() });
      demoFlux = demoFlux.map(flow => flow.id === id ? updated : flow);
      return updated as T;
    }
    case 'cancel_flux': {
      const id = args.id as string, current = demoFlux.find(flow => flow.id === id);
      if (!current) throw new Error('Ce flux n’existe plus.');
      if (current.statut === 'annule') throw new Error('Cette prévision est déjà annulée.');
      const updated = summarizeFlux({ ...current, statut: 'annule', updatedAt: new Date().toISOString() });
      demoFlux = demoFlux.map(flow => flow.id === id ? updated : flow);
      return updated as T;
    }
    case 'arreter_flux_recurrence': {
      const recurrenceId = String(args.recurrenceId ?? '');
      const dateFin = String(args.dateFin ?? '');
      const seriesFlows = demoFlux.filter(flow => flow.recurrenceId === recurrenceId);
      if (!seriesFlows.length) throw new Error('Cette série de récurrence n’existe plus.');
      if (!seriesFlows.some(flow => flow.recurrenceActive)) throw new Error('Cette récurrence est déjà arrêtée.');
      if (!isValidIsoDate(dateFin)) throw new Error('La date d’arrêt doit être au format AAAA-MM-JJ.');
      const start = seriesFlows.map(flow => flow.datePrevue).sort()[0];
      const existingEnd = seriesFlows[0].dateFinRecurrence;
      if (dateFin < start) throw new Error('La date d’arrêt ne peut pas précéder la première échéance.');
      if (existingEnd && dateFin > existingEnd) throw new Error('La date d’arrêt ne peut pas prolonger une récurrence déjà limitée.');
      if (!seriesFlows.some(flow => flow.datePrevue === dateFin)) {
        throw new Error('La date d’arrêt doit correspondre à une échéance réelle de cette série.');
      }
      if (seriesFlows.some(flow => flow.datePrevue > dateFin && flow.realisations.some(item => item.annuleeAt === null))) {
        throw new Error('Impossible d’arrêter la récurrence avant une échéance qui possède déjà une réalisation active. Choisissez une date de fin à cette échéance ou après.');
      }
      const now = new Date().toISOString();
      demoFlux = demoFlux.map(flow => {
        if (flow.recurrenceId !== recurrenceId) return flow;
        const cancelled = flow.datePrevue > dateFin && flow.statut === 'actif';
        return summarizeFlux({
          ...flow,
          dateFinRecurrence: dateFin,
          recurrenceActive: false,
          statut: cancelled ? 'annule' : flow.statut,
          updatedAt: now,
        });
      });
      return undefined as T;
    }
    case 'register_flux_realisation': {
      const fluxId = args.fluxId as string, rawInput = args.input as RealisationFluxInput;
      const current = demoFlux.find(flow => flow.id === fluxId);
      if (!current) throw new Error('Ce flux n’existe plus.');
      if (current.statut !== 'actif') throw new Error('Impossible de réaliser une prévision annulée.');
      validateDemoRealisationInput(rawInput);
      const input = {
        ...rawInput,
        reference: cleanDemoOptional(rawInput.reference),
        note: cleanDemoOptional(rawInput.note),
      };
      const item: FluxRealisationView = { id: crypto.randomUUID(), fluxId, ...input, createdAt: new Date().toISOString(), annuleeAt: null, motifAnnulation: null };
      const updated = summarizeFlux({ ...current, realisations: [item, ...current.realisations], updatedAt: new Date().toISOString() });
      demoFlux = demoFlux.map(flow => flow.id === fluxId ? updated : flow);
      return updated as T;
    }
    case 'cancel_flux_realisation': {
      const id = args.id as string, motif = String(args.motifAnnulation ?? '').trim();
      if (!motif || motif.length > 300) throw new Error('Indiquez un motif d’annulation de 1 à 300 caractères.');
      const owner = demoFlux.find(flow => flow.realisations.some(item => item.id === id));
      if (!owner) throw new Error('Cette réalisation n’existe plus.');
      const current = owner.realisations.find(item => item.id === id)!;
      if (current.annuleeAt) throw new Error('Cette réalisation est déjà annulée.');
      const updated = summarizeFlux({ ...owner, updatedAt: new Date().toISOString(), realisations: owner.realisations.map(item => item.id === id ? { ...item, annuleeAt: new Date().toISOString(), motifAnnulation: motif } : item) });
      demoFlux = demoFlux.map(flow => flow.id === owner.id ? updated : flow);
      return updated as T;
    }
    case 'list_tasks': return [...demoTasks] as T;
    case 'create_task': {
      const input = args.input as NewTask;
      const idempotencyKey = input.idempotencyKey?.trim();
      if (!idempotencyKey || idempotencyKey.length > 200) throw new Error('La clé d’idempotence doit comporter de 1 à 200 caractères.');
      if (!input.module?.trim() || input.module.trim().length > 100) throw new Error('Le module doit comporter de 1 à 100 caractères.');
      if (!input.taskType?.trim() || input.taskType.trim().length > 100) throw new Error('Le type de tâche doit comporter de 1 à 100 caractères.');
      if (!input.title?.trim() || input.title.trim().length > 200) throw new Error('Le titre doit comporter de 1 à 200 caractères.');
      if (!Number.isInteger(input.maxAttempts) || input.maxAttempts < 1 || input.maxAttempts > 100) throw new Error('Le nombre de tentatives doit être compris entre 1 et 100.');
      const scheduledDate = new Date(input.scheduledAt);
      if (Number.isNaN(scheduledDate.getTime())) throw new Error('La date de planification est invalide.');
      const existing = demoTasks.find(task => task.idempotencyKey === idempotencyKey);
      if (existing) {
        const same = existing.module === input.module.trim()
          && existing.taskType === input.taskType.trim()
          && existing.title === input.title.trim()
          && JSON.stringify(existing.payload) === JSON.stringify(input.payload)
          && existing.scheduledAt === scheduledDate.toISOString()
          && existing.maxAttempts === input.maxAttempts;
        if (!same) throw new Error('Cette clé d’idempotence est déjà utilisée pour une demande différente.');
        return existing as T;
      }
      const now = new Date().toISOString();
      const task: Task = {
        ...input,
        id: crypto.randomUUID(),
        idempotencyKey,
        module: input.module.trim(),
        taskType: input.taskType.trim(),
        title: input.title.trim(),
        scheduledAt: scheduledDate.toISOString(),
        status: 'scheduled',
        createdAt: now,
        updatedAt: now,
      };
      demoTasks = [task, ...demoTasks];
      return task as T;
    }
    case 'update_task': {
      const taskId = String(args.taskId ?? '');
      const input = args.input as NewTask;
      const current = demoTasks.find(item => item.id === taskId);
      if (!current) throw new Error('Cette tâche n’existe plus.');
      if (current.status === 'cancelled') throw new Error('Réactivez la tâche avant de la modifier.');
      if (demoTaskExecutions.some(item => item.taskId === taskId)) {
        throw new Error('Une exécution existe déjà : les paramètres de cette tâche sont verrouillés pour préserver l’historique.');
      }
      if (input.idempotencyKey?.trim() !== current.idempotencyKey) {
        throw new Error('La clé d’idempotence de cette tâche ne peut pas être modifiée.');
      }
      if (!input.module?.trim() || input.module.trim().length > 100) throw new Error('Le module doit comporter de 1 à 100 caractères.');
      if (!input.taskType?.trim() || input.taskType.trim().length > 100) throw new Error('Le type de tâche doit comporter de 1 à 100 caractères.');
      if (!input.title?.trim() || input.title.trim().length > 200) throw new Error('Le titre doit comporter de 1 à 200 caractères.');
      if (!Number.isInteger(input.maxAttempts) || input.maxAttempts < 1 || input.maxAttempts > 100) throw new Error('Le nombre de tentatives doit être compris entre 1 et 100.');
      const scheduledDate = new Date(input.scheduledAt);
      if (Number.isNaN(scheduledDate.getTime())) throw new Error('La date de planification est invalide.');
      const updated: Task = {
        ...current,
        ...input,
        module: input.module.trim(),
        taskType: input.taskType.trim(),
        title: input.title.trim(),
        scheduledAt: scheduledDate.toISOString(),
        updatedAt: new Date().toISOString(),
      };
      demoTasks = demoTasks.map(item => item.id === taskId ? updated : item);
      return updated as T;
    }
    case 'update_task_status': {
      const taskId = String(args.taskId ?? '');
      const status = args.status as TaskStatus;
      if (!['scheduled', 'paused', 'cancelled'].includes(status)) throw new Error('Le statut de tâche est invalide.');
      const task = demoTasks.find(item => item.id === taskId);
      if (!task) throw new Error('Cette tâche n’existe plus.');
      const now = new Date().toISOString();
      demoTasks = demoTasks.map(item => item.id === taskId ? { ...item, status, updatedAt: now } : item);
      if (status === 'cancelled') {
        demoTaskExecutions = demoTaskExecutions.map(item =>
          item.taskId === taskId && item.status === 'queued'
            ? { ...item, status: 'cancelled', finishedAt: now, leaseUntil: null, updatedAt: now }
            : item,
        );
      }
      return demoTasks.find(item => item.id === taskId)! as T;
    }
    case 'list_task_executions': {
      const taskId = String(args.taskId ?? '');
      return demoTaskExecutions.filter(item => item.taskId === taskId) as T;
    }
    case 'list_task_attempts': {
      const executionId = String(args.executionId ?? '');
      return demoTaskAttempts.filter(item => item.executionId === executionId) as T;
    }
    case 'list_comptes': return demoAccounts as T;
    case 'list_ecritures': {
      const filter = args.societeId as string | null | undefined;
      return demoEntries.filter((entry) => !filter || entry.societeId === filter) as T;
    }
    case 'create_ecriture': {
      const input = args.input as CreateEntryInput;
      const debit = input.lines.reduce((sum, line) => sum + line.debitCents, 0);
      const credit = input.lines.reduce((sum, line) => sum + line.creditCents, 0);
      if (debit !== credit || debit <= 0 || input.lines.length < 2) throw new Error('Écriture non équilibrée.');
      const entry: EntryView = { ...input, id: crypto.randomUUID(), statut: 'validated', totalCents: debit };
      demoEntries = [entry, ...demoEntries];
      return entry as T;
    }
    case 'dashboard_summary': return {
      societes: demoSocietes.length, ecrituresValidees: demoEntries.length,
      lignesComptables: demoEntries.reduce((sum, entry) => sum + entry.lines.length, 0),
      volumeDebitCents: demoEntries.reduce((sum, entry) => sum + entry.totalCents, 0),
    } as T;
    default: throw new Error(`Commande non disponible en mode navigateur : ${command}`);
  }
}

export function isDesktopRuntime(): boolean {
  return isDesktop;
}

export async function api<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isDesktop) return invoke<T>(command, args);
  return browserCommand<T>(command, args);
}

export type { EntryLine };
