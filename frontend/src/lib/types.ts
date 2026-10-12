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
