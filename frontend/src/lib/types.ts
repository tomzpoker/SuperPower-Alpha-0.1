export type DatabaseStatus = 'needs_setup' | 'locked' | 'ready';
export type FormeJuridique = 'sci' | 'sarl' | 'sas' | 'autre';
export type RegimeFiscal = 'ir' | 'is' | 'non_renseigne';
export type RegimeTva = 'non_renseigne' | 'hors_champ' | 'franchise_base' | 'reel_simplifie' | 'reel_normal';
export type ExigibiliteTva = 'non_definie' | 'non_applicable' | 'encaissements' | 'debits';
export type PeriodiciteTva = 'non_definie' | 'non_applicable' | 'mensuelle' | 'trimestrielle' | 'annuelle';

export interface Societe {
  id: string;
  nom: string;
  formeJuridique: FormeJuridique;
  regimeFiscal: RegimeFiscal;
  regimeTva: RegimeTva;
  exigibiliteTva: ExigibiliteTva;
  periodiciteTva: PeriodiciteTva;
  siret: string | null;
  finExercice: string;
  createdAt: string;
  warnings: string[];
}

export interface Account {
  code: string;
  label: string;
  accountClass: number;
}

export interface EntryLine {
  accountCode: string;
  accountLabel: string;
  debitCents: number;
  creditCents: number;
  lineLabel?: string | null;
}

export interface EntryView {
  id: string;
  societeId: string;
  dateEcriture: string;
  libelle: string;
  reference: string | null;
  statut: 'validated' | string;
  totalCents: number;
  lines: EntryLine[];
}

export interface DashboardSummary {
  societes: number;
  ecrituresValidees: number;
  lignesComptables: number;
  volumeDebitCents: number;
}

export interface CreateSocieteInput {
  nom: string;
  formeJuridique: FormeJuridique;
  regimeFiscal: RegimeFiscal;
  regimeTva: RegimeTva;
  exigibiliteTva: ExigibiliteTva;
  periodiciteTva: PeriodiciteTva;
  siret: string | null;
  finExercice: string;
}

export interface CreateEntryInput {
  societeId: string;
  dateEcriture: string;
  libelle: string;
  reference: string | null;
  lines: EntryLine[];
}

export type TaskStatus = 'scheduled' | 'paused' | 'cancelled';
export type TaskExecutionStatus = 'queued' | 'running' | 'succeeded' | 'failed' | 'cancelled';
export type TaskAttemptStatus = 'running' | 'succeeded' | 'failed';

export interface NewTask {
  idempotencyKey: string;
  module: string;
  taskType: string;
  title: string;
  payload: unknown;
  scheduledAt: string;
  cronExpression?: string | null;
  maxAttempts: number;
}

export interface Task extends NewTask {
  id: string;
  /** Date de départ d'origine conservée pour l'idempotence des tâches cron. */
  cronAnchorAt?: string | null;
  status: TaskStatus;
  createdAt: string;
  updatedAt: string;
}

export interface TaskExecution {
  id: string;
  taskId: string;
  idempotencyKey: string;
  scheduledFor: string;
  availableAt: string;
  status: TaskExecutionStatus;
  attemptCount: number;
  maxAttempts: number;
  leaseUntil: string | null;
  lastError: string | null;
  createdAt: string;
  updatedAt: string;
  startedAt: string | null;
  finishedAt: string | null;
}

export interface TaskAttempt {
  id: string;
  executionId: string;
  attemptNumber: number;
  status: TaskAttemptStatus;
  startedAt: string;
  finishedAt: string | null;
  errorMessage: string | null;
}


export type SensFlux = 'encaissement' | 'decaissement';
export type CategorieFlux = 'loyer' | 'charge_courante' | 'travaux' | 'impot_taxe' | 'emprunt' | 'apport_associe' | 'autre';
export type StatutFlux = 'actif' | 'annule';
export type EtatFlux = 'a_prevoir' | 'en_retard' | 'partiellement_realise' | 'realise' | 'depasse' | 'annule';
export type FrequenceFlux = 'ponctuel' | 'mensuel' | 'trimestriel' | 'annuel';

export interface FluxInput {
  societeId: string;
  libelle: string;
  sens: SensFlux;
  categorie: CategorieFlux;
  datePrevue: string;
  montantPrevuCents: number;
  tiers: string | null;
  reference: string | null;
  frequenceRecurrence: FrequenceFlux;
  dateFinRecurrence: string | null;
}
export interface RealisationFluxInput {
  dateRealisation: string;
  montantCents: number;
  reference: string | null;
  note: string | null;
}
export interface FluxRealisationView {
  id: string;
  fluxId: string;
  dateRealisation: string;
  montantCents: number;
  reference: string | null;
  note: string | null;
  createdAt: string;
  annuleeAt: string | null;
  motifAnnulation: string | null;
}
export interface FluxView extends FluxInput {
  id: string;
  statut: StatutFlux;
  etat: EtatFlux;
  montantRealiseCents: number;
  resteCents: number;
  realisations: FluxRealisationView[];
  recurrenceId: string | null;
  recurrenceActive: boolean;
  createdAt: string;
  updatedAt: string;
}
