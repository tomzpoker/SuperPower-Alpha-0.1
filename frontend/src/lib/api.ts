import { invoke } from '@tauri-apps/api/core';
import type {
  Account, CreateEntryInput, CreateSocieteInput, DatabaseStatus,
  EntryLine, EntryView, FluxInput, FluxRealisationView, FluxView, RealisationFluxInput, Societe,
} from './types';

// In the browser, only an in-memory preview is provided. Real data belongs in the
// native Tauri shell, where the database is encrypted. This demo resets on reload.
const isDesktop = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
let demoReady = false;
let demoSocietes: Societe[] = [];
let demoEntries: EntryView[] = [];
let demoFlux: FluxView[] = [];
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
}

function normalizeDemoFluxInput(input: FluxInput): FluxInput {
  return {
    ...input,
    societeId: input.societeId.trim(),
    libelle: input.libelle.trim(),
    tiers: cleanDemoOptional(input.tiers),
    reference: cleanDemoOptional(input.reference),
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
      const flow: FluxView = summarizeFlux({ ...input, id: crypto.randomUUID(), statut: 'actif', etat: 'a_prevoir', montantRealiseCents: 0, resteCents: input.montantPrevuCents, realisations: [], createdAt: now, updatedAt: now });
      demoFlux = [flow, ...demoFlux];
      return flow as T;
    }
    case 'update_flux': {
      const id = args.id as string, rawInput = args.input as FluxInput;
      const current = demoFlux.find(flow => flow.id === id);
      if (!current) throw new Error('Ce flux n’existe plus.');
      if (current.statut !== 'actif') throw new Error('Un flux annulé ne peut pas être modifié.');
      if (current.societeId !== rawInput.societeId) throw new Error('La société d’un flux existant ne peut pas être changée.');
      validateDemoFluxInput(rawInput);
      const input = normalizeDemoFluxInput(rawInput);
      const updated = summarizeFlux({ ...current, ...input, updatedAt: new Date().toISOString() });
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
