import { invoke } from '@tauri-apps/api/core';
import type {
  Account, CreateEntryInput, CreateSocieteInput, DatabaseStatus,
  EntryLine, EntryView, Societe,
} from './types';

// In the browser, only an in-memory preview is provided. Real data belongs in the
// native Tauri shell, where the database is encrypted. This demo resets on reload.
const isDesktop = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
let demoReady = false;
let demoSocietes: Societe[] = [];
let demoEntries: EntryView[] = [];
const demoAccounts: Account[] = [
  ['101', 'Capital', 1], ['164', 'Emprunts auprès des établissements de crédit', 1],
  ['213', 'Constructions', 2], ['2813', 'Amortissements des constructions', 2],
  ['401', 'Fournisseurs', 4], ['411', 'Clients', 4], ['44566', 'TVA déductible sur autres biens et services', 4],
  ['44571', 'TVA collectée', 4], ['471', "Compte d'attente", 4], ['512', 'Banque', 5], ['530', 'Caisse', 5],
  ['606', 'Achats non stockés de matières et fournitures', 6], ['613', 'Locations', 6], ['616', "Primes d'assurances", 6],
  ['622', 'Honoraires', 6], ['63512', 'Taxes foncières', 6], ['661', "Charges d'intérêts", 6],
  ['706', 'Prestations de services', 7], ['708', 'Produits des activités annexes', 7], ['758', 'Produits divers de gestion courante', 7],
].map(([code, label, accountClass]) => ({ code: String(code), label: String(label), accountClass: Number(accountClass) }));

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
      demoSocietes = demoSocietes.filter((company) => company.id !== id);
      return undefined as T;
    }    case 'list_comptes': return demoAccounts as T;
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
