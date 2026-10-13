import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { FormEvent, ReactNode } from 'react';
import {
  Activity, ArrowDownLeft, ArrowRight, ArrowUpRight, Bell, BookOpenCheck,
  Building2, CalendarDays, CheckCircle2, ChevronDown, ChevronLeft, ChevronRight, CircleAlert, Clock3,
  Files, House, Landmark, LayoutDashboard, LockKeyhole, LogOut,
  Menu, Pencil, Plus, ReceiptText, ShieldCheck, Sparkles, Trash2, Wallet, X,
} from 'lucide-react';
import { api, isDesktopRuntime } from './lib/api';
import { formatDate, formatMoney, parseAmountToCents } from './lib/money';
import { readReminderMessage, withReminderMessage } from './lib/task-payload';
import { taskCalendarOccurrenceDates } from './lib/task-calendar';
import { filterTasks } from './lib/task-filter';
import type { TaskStatusFilter } from './lib/task-filter';
import type {
  Account, CreateEntryInput, CreateSocieteInput, DashboardSummary, DatabaseStatus,
  EntryView, ExigibiliteTva, FluxInput, FluxRealisationView, FluxView, FormeJuridique,
  CategorieFlux, FrequenceFlux, NewTask, PeriodiciteTva, RealisationFluxInput, RegimeFiscal, RegimeTva, SensFlux, Societe,
  Task, TaskAttempt, TaskExecution, TaskRecurrenceFrequency, TaskStatus,
} from './lib/types';

type PageKey = 'dashboard' | 'societes' | 'ecritures' | 'documents' | 'taches' | 'tresorerie' | 'locatif' | 'fiscalite' | 'parametres';
type ThemeKey = 'aurora' | 'sakura' | 'mint' | 'sunset' | 'midnight';

const taskRecurrenceLabels: Record<TaskRecurrenceFrequency, string> = { once: 'Ponctuelle', monthly: 'Mensuelle', quarterly: 'Trimestrielle', yearly: 'Annuelle', custom_cron: 'Cron personnalisé' };

type TaskReminderEvent = {
  executionId: string;
  title: string;
  message: string;
  scheduledFor: string;
};

type ToastMessage = {
  id: number;
  text: string;
};

const navigation: Array<{ key: PageKey; label: string; icon: typeof LayoutDashboard; group: string }> = [
  { key: 'dashboard', label: 'Cockpit', icon: LayoutDashboard, group: 'ESPACE DE TRAVAIL' },
  { key: 'societes', label: 'Sociétés', icon: Building2, group: 'ESPACE DE TRAVAIL' },
  { key: 'ecritures', label: 'Comptabilité', icon: BookOpenCheck, group: 'ESPACE DE TRAVAIL' },
  { key: 'documents', label: 'Documents', icon: Files, group: 'OPÉRATIONS' },
  { key: 'taches', label: 'Tâches', icon: Clock3, group: 'OPÉRATIONS' },
  { key: 'locatif', label: 'Gestion locative', icon: House, group: 'OPÉRATIONS' },
  { key: 'tresorerie', label: 'Trésorerie', icon: Wallet, group: 'PILOTAGE' },
  { key: 'fiscalite', label: 'Fiscalité', icon: ReceiptText, group: 'PILOTAGE' },
  { key: 'parametres', label: 'Paramètres', icon: ShieldCheck, group: 'SYSTÈME' },
];

const emptySummary: DashboardSummary = { societes: 0, ecrituresValidees: 0, lignesComptables: 0, volumeDebitCents: 0 };
const themeNames: Record<ThemeKey, string> = { aurora: 'Aurora', sakura: 'Sakura', mint: 'Mint', sunset: 'Sunset', midnight: 'Midnight' };

const defaultCompanyForm: CreateSocieteInput = {
  nom: '', formeJuridique: 'sci', regimeFiscal: 'ir', regimeTva: 'non_renseigne',
  exigibiliteTva: 'non_definie', periodiciteTva: 'non_definie', siret: null,
  finExercice: `${new Date().getFullYear()}-12-31`,
};

function formatUtcDateTime(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat('fr-FR', {
    dateStyle: 'medium',
    timeStyle: 'short',
    timeZone: 'UTC',
  }).format(date);
}

function App() {
  const [theme, setTheme] = useState<ThemeKey>(() => {
    const saved = localStorage.getItem('superpower-theme');
    return saved && saved in themeNames ? saved as ThemeKey : 'aurora';
  });
  const [status, setStatus] = useState<DatabaseStatus | 'loading'>('loading');
  const [passphrase, setPassphrase] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [page, setPage] = useState<PageKey>('dashboard');
  const [societes, setSocietes] = useState<Societe[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [entries, setEntries] = useState<EntryView[]>([]);
  const [flux, setFlux] = useState<FluxView[]>([]);
  const [summary, setSummary] = useState<DashboardSummary>(emptySummary);
  const [selectedSocieteId, setSelectedSocieteId] = useState('');
  const [showCompanyForm, setShowCompanyForm] = useState(false);
  const [editingSocieteId, setEditingSocieteId] = useState<string | null>(null);
  const [companyForm, setCompanyForm] = useState<CreateSocieteInput>(defaultCompanyForm);
  const [showEntryForm, setShowEntryForm] = useState(false);
  const [entryDate, setEntryDate] = useState(() => new Date().toISOString().slice(0, 10));
  const [entryLabel, setEntryLabel] = useState('');
  const [entryReference, setEntryReference] = useState('');
  const [debitAccount, setDebitAccount] = useState('512');
  const [creditAccount, setCreditAccount] = useState('706');
  const [entryAmount, setEntryAmount] = useState('');
  const [entryBusy, setEntryBusy] = useState(false);
  const [toast, setToast] = useState<ToastMessage | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const activeToast = useRef<ToastMessage | null>(null);
  const toastQueue = useRef<ToastMessage[]>([]);
  const toastSequence = useRef(0);

  const showToast = useCallback((text: string) => {
    const message = { id: ++toastSequence.current, text };
    if (activeToast.current) {
      toastQueue.current.push(message);
    } else {
      activeToast.current = message;
      setToast(message);
    }
  }, []);

  const dismissToast = useCallback(() => {
    const next = toastQueue.current.shift() ?? null;
    activeToast.current = next;
    setToast(next);
  }, []);

  const activeCompany = useMemo(() => societes.find((company) => company.id === selectedSocieteId) ?? null, [societes, selectedSocieteId]);
  const activePage = navigation.find((item) => item.key === page) ?? navigation[0];

  const loadData = useCallback(async () => {
    setError('');
    const [companies, accountList, entryList, metrics, flowList] = await Promise.all([
      api<Societe[]>('list_societes'), api<Account[]>('list_comptes'),
      api<EntryView[]>('list_ecritures', { societeId: null }),
      api<DashboardSummary>('dashboard_summary'),
      api<FluxView[]>('list_flux', { societeId: null }),
    ]);
    setSocietes(companies);
    setAccounts(accountList);
    setEntries(entryList);
    setFlux(flowList);
    setSummary(metrics);
    setSelectedSocieteId((current) => current && companies.some((company) => company.id === current) ? current : companies[0]?.id ?? '');
    if (accountList.length) {
      setDebitAccount((current) => accountList.some((account) => account.code === current) ? current : accountList[0].code);
      setCreditAccount((current) => accountList.some((account) => account.code === current) ? current : accountList[Math.min(1, accountList.length - 1)].code);
    }
  }, []);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem('superpower-theme', theme);
  }, [theme]);

  useEffect(() => {
    let active = true;
    api<DatabaseStatus>('database_status')
      .then((databaseStatus) => { if (active) setStatus(databaseStatus); })
      .catch((reason: unknown) => { if (active) { setStatus('locked'); setError(errorMessage(reason)); } });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    if (status !== 'ready') return;
    loadData().catch((reason: unknown) => setError(errorMessage(reason)));
  }, [status, loadData]);

  useEffect(() => {
    if (!isDesktopRuntime()) return;
    let active = true;
    let unlisten: (() => void) | undefined;
    void listen<TaskReminderEvent>('task-reminder', ({ payload }) => {
      const detail = payload.message && payload.message !== payload.title ? ` — ${payload.message}` : '';
      showToast(`Rappel : ${payload.title}${detail}`);
    })
      .then((stopListening) => {
        if (active) unlisten = stopListening;
        else stopListening();
      })
      .catch((reason: unknown) => {
        if (active) setError(errorMessage(reason));
      });
    return () => {
      active = false;
      unlisten?.();
    };
  }, [showToast]);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(dismissToast, 6500);
    return () => window.clearTimeout(timer);
  }, [toast, dismissToast]);

  async function handleDatabaseAccess(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError('');
    try {
      const command = status === 'needs_setup' ? 'initialize_database' : 'unlock_database';
      await api<void>(command, { passphrase });
      setPassphrase('');
      setStatus('ready');
      showToast(!isDesktopRuntime() ? 'Mode démo activé : données temporaires en mémoire uniquement.' : status === 'needs_setup' ? 'Coffre local chiffré créé.' : 'Coffre déverrouillé.');
    } catch (reason) {
      setError(errorMessage(reason));
      try {
        setStatus(await api<DatabaseStatus>('database_status'));
      } catch {
        // Keep the original error visible if the status query also fails.
      }
    } finally {
      setBusy(false);
    }
  }

  async function handleLock() {
    try {
      await api<void>('lock_database');
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setStatus('locked');
      setPassphrase('');
      setSocietes([]);
      setEntries([]);
      setFlux([]);
      setAccounts([]);
      setSummary(emptySummary);
      setPage('dashboard');
    }
  }

  function handleEditCompany(company: Societe) {
    setError('');
    setEditingSocieteId(company.id);
    setCompanyForm({
      nom: company.nom,
      formeJuridique: company.formeJuridique,
      regimeFiscal: company.regimeFiscal,
      regimeTva: company.regimeTva,
      exigibiliteTva: company.exigibiliteTva,
      periodiciteTva: company.periodiciteTva,
      siret: company.siret,
      finExercice: company.finExercice,
    });
    setShowCompanyForm(true);
  }

  async function handleDeleteCompany(company: Societe) {
    const approved = window.confirm(`Supprimer la société « ${company.nom} » ? Cette action est irréversible. La suppression sera refusée si des écritures comptables lui sont liées.`);
    if (!approved) return;
    setError('');
    try {
      await api<void>('delete_societe', { id: company.id });
      if (editingSocieteId === company.id) {
        setEditingSocieteId(null);
        setCompanyForm(defaultCompanyForm);
        setShowCompanyForm(false);
      }
      await loadData();
      showToast(isDesktopRuntime() ? 'Société supprimée du coffre local.' : 'Société supprimée du mode démo volatile.');
    } catch (reason) { setError(errorMessage(reason)); }
  }
  async function handleCreateCompany(event: FormEvent) {
    event.preventDefault();
    setError('');
    try {
      const editingId = editingSocieteId;
      const isEditing = editingId !== null;
      const input = { ...companyForm, siret: companyForm.siret?.trim() || null };
      const created = editingId
        ? await api<Societe>('update_societe', { id: editingId, input })
        : await api<Societe>('create_societe', { input });
      setCompanyForm(defaultCompanyForm);
      setEditingSocieteId(null);
      setShowCompanyForm(false);
      setSelectedSocieteId(created.id);
      await loadData();
      showToast(isDesktopRuntime() ? (isEditing ? 'Société modifiée dans le coffre local.' : 'Société enregistrée dans le coffre local.') : (isEditing ? 'Société modifiée dans le mode démo volatile.' : 'Société ajoutée au mode démo volatile.'));
    } catch (reason) { setError(errorMessage(reason)); }
  }

  async function handleCreateEntry(event: FormEvent) {
    event.preventDefault();
    const targetSocieteId = selectedSocieteId || societes[0]?.id;
    if (!targetSocieteId) { setError('Créez ou sélectionnez une société avant de saisir une écriture.'); return; }
    const amountCents = parseAmountToCents(entryAmount);
    if (amountCents === null) { setError('Montant invalide. Saisissez un montant strictement positif avec au maximum deux décimales.'); return; }
    if (debitAccount === creditAccount) { setError('Choisissez deux comptes différents pour cette écriture.'); return; }
    setEntryBusy(true); setError('');
    try {
      const debit = accounts.find((account) => account.code === debitAccount);
      const credit = accounts.find((account) => account.code === creditAccount);
      if (!debit || !credit) throw new Error('Le compte choisi ne fait pas partie du plan de comptes chargé.');
      const input: CreateEntryInput = {
        societeId: targetSocieteId, dateEcriture: entryDate, libelle: entryLabel.trim(), reference: entryReference.trim() || null,
        lines: [
          { accountCode: debit.code, accountLabel: debit.label, debitCents: amountCents, creditCents: 0 },
          { accountCode: credit.code, accountLabel: credit.label, debitCents: 0, creditCents: amountCents },
        ],
      };
      await api<EntryView>('create_ecriture', { input });
      setEntryLabel(''); setEntryReference(''); setEntryAmount(''); setShowEntryForm(false);
      await loadData();
      showToast('Écriture équilibrée et validée. Elle est verrouillée : une correction devra passer par une contre-écriture.');
    } catch (reason) { setError(errorMessage(reason)); }
    finally { setEntryBusy(false); }
  }

  async function refreshEntries() {
    try {
      const list = await api<EntryView[]>('list_ecritures', { societeId: null });
      setEntries(list);
    } catch (reason) { setError(errorMessage(reason)); }
  }

  async function refreshFlows() {
    setFlux(await api<FluxView[]>('list_flux', { societeId: null }));
  }
  async function createFlow(input: FluxInput): Promise<boolean> {
    setError('');
    try { await api<FluxView>('create_flux', { input }); await refreshFlows(); showToast(isDesktopRuntime() ? 'Prévision enregistrée dans le coffre local.' : 'Prévision enregistrée en mémoire démo volatile.'); return true; }
    catch (reason) { setError(errorMessage(reason)); return false; }
  }
  async function updateFlow(id: string, input: FluxInput): Promise<boolean> {
    setError('');
    try { await api<FluxView>('update_flux', { id, input }); await refreshFlows(); showToast('Prévision mise à jour ; les réalisations sont conservées.'); return true; }
    catch (reason) { setError(errorMessage(reason)); return false; }
  }
  async function realiseFlow(fluxId: string, input: RealisationFluxInput): Promise<boolean> {
    setError('');
    try { await api<FluxView>('register_flux_realisation', { fluxId, input }); await refreshFlows(); showToast('Réalisation enregistrée ; le total et l’écart ont été recalculés.'); return true; }
    catch (reason) { setError(errorMessage(reason)); return false; }
  }
  async function cancelFlow(id: string): Promise<boolean> {
    setError('');
    try { await api<FluxView>('cancel_flux', { id }); await refreshFlows(); showToast('Prévision annulée ; historique conservé.'); return true; }
    catch (reason) { setError(errorMessage(reason)); return false; }
  }
  async function cancelRealisation(id: string, motifAnnulation: string): Promise<boolean> {
    setError('');
    try { await api<FluxView>('cancel_flux_realisation', { id, motifAnnulation }); await refreshFlows(); showToast('Réalisation annulée et motif conservé.'); return true; }
    catch (reason) { setError(errorMessage(reason)); return false; }
  }
  async function stopRecurrence(recurrenceId: string, dateFin: string): Promise<boolean> {
    setError('');
    try {
      await api<void>('arreter_flux_recurrence', { recurrenceId, dateFin });
      await refreshFlows();
      showToast('Récurrence arrêtée ; les échéances postérieures sont conservées dans l’historique et annulées.');
      return true;
    } catch (reason) { setError(errorMessage(reason)); return false; }
  }

  if (status === 'loading') {
    return <div className="boot-screen"><div className="brand-mark large">S<span>+</span></div><p>Ouverture de SuperPower…</p></div>;
  }

  if (status !== 'ready') {
    return <div className="auth-screen" data-theme={theme}>
      <div className="auth-orb orb-one" /><div className="auth-orb orb-two" />
      <div className="auth-panel">
        <div className="brand-lockup"><div className="brand-mark">S<span>+</span></div><div><div className="brand-name">SuperPower</div><div className="brand-subtitle">ALPHA · ESPACE LOCAL</div></div></div>
        <div className="eyebrow">VOTRE COCKPIT DE GESTION</div>
        <h1>{status === 'needs_setup' ? 'Créons votre coffre.' : 'Content de vous revoir.'}</h1>
        <p className="auth-copy">{status === 'needs_setup' ? 'Une phrase secrète protège votre base de données chiffrée. Choisissez-en une longue et mémorisable.' : 'Votre coffre est verrouillé. Entrez la phrase secrète utilisée lors de son initialisation.'}</p>
        <form onSubmit={handleDatabaseAccess} className="auth-form">
          <label htmlFor="passphrase">Phrase secrète principale</label>
          <input id="passphrase" type="password" autoComplete="current-password" value={passphrase} onChange={(event) => setPassphrase(event.target.value)} minLength={12} required placeholder="12 caractères minimum" />
          {status === 'needs_setup' && <div className="security-note"><ShieldCheck size={16} /><span>La phrase secrète n'est pas stockée. Sans elle, la base ne pourra pas être récupérée.</span></div>}
          {error && <div className="alert error"><CircleAlert size={17} />{error}</div>}
          <button className="button primary full" type="submit" disabled={busy || passphrase.trim().length < 12}>
            {busy ? 'Ouverture…' : status === 'needs_setup' ? 'Créer le coffre chiffré' : 'Déverrouiller'} <ArrowRight size={17} />
          </button>
        </form>
        <div className="auth-foot"><LockKeyhole size={14} /> Chiffrement local SQLCipher · Clé dérivée via Argon2id</div>
        {!isDesktopRuntime() && <div className="alert warning browser-alert"><CircleAlert size={16} />Aperçu navigateur : données en mémoire uniquement, non chiffrées et perdues au rechargement. Utilisez la fenêtre Tauri pour vos données réelles.</div>}
        <div className="theme-mini"><span>Ambiance</span><select value={theme} onChange={(event) => setTheme(event.target.value as ThemeKey)} aria-label="Thème visuel">{Object.entries(themeNames).map(([key, name]) => <option key={key} value={key}>{name}</option>)}</select></div>
      </div>
    </div>;
  }

  return <div className="app-shell" data-theme={theme}>
    <aside className={`sidebar ${sidebarOpen ? 'sidebar-open' : ''}`}>
      <div className="brand-lockup sidebar-brand"><div className="brand-mark">S<span>+</span></div><div><div className="brand-name">SuperPower</div><div className="brand-subtitle">ALPHA 0.1 · LOCAL</div></div><button className="icon-button mobile-only" onClick={() => setSidebarOpen(false)} aria-label="Fermer le menu"><X size={18} /></button></div>
      <div className="workspace-pill"><span className="workspace-dot" /> Espace de gestion <ChevronDown size={14} /></div>
      <nav className="main-nav" aria-label="Navigation principale">
        {(['ESPACE DE TRAVAIL', 'OPÉRATIONS', 'PILOTAGE', 'SYSTÈME']).map((group) => <div className="nav-group" key={group}>
          <div className="nav-caption">{group}</div>
          {navigation.filter((item) => item.group === group).map((item) => {
            const Icon = item.icon;
            return <button key={item.key} className={`nav-item ${page === item.key ? 'active' : ''}`} onClick={() => { setPage(item.key); setSidebarOpen(false); setError(''); }}>
              <Icon size={18} strokeWidth={1.8} /><span>{item.label}</span>{item.key === 'societes' && societes.length > 0 && <span className="nav-count">{societes.length}</span>}
              {item.key === 'fiscalite' && <span className="nav-soon">Bientôt</span>}
            </button>;
          })}
        </div>)}
      </nav>
      <div className="sidebar-bottom">
        <div className="sidebar-sparkle"><div className="sparkle-icon"><Sparkles size={17} /></div><div><strong>Un geste à la fois.</strong><span>Une gestion plus sereine.</span></div></div>
        <div className="user-row"><div className="avatar">SP</div><div className="user-meta"><strong>Administrateur local</strong><span>Stockage sur cet appareil</span></div><button className="icon-button" onClick={handleLock} title="Verrouiller le coffre" aria-label="Verrouiller le coffre"><LogOut size={17} /></button></div>
      </div>
    </aside>
    {sidebarOpen && <button className="sidebar-scrim" aria-label="Fermer le menu" onClick={() => setSidebarOpen(false)} />}
    <main className="main-area">
      <header className="topbar">
        <div className="topbar-left"><button className="icon-button mobile-only" onClick={() => setSidebarOpen(true)} aria-label="Ouvrir le menu"><Menu size={19} /></button><div className="breadcrumb"><span>SuperPower</span><span className="crumb-slash">/</span><strong>{activePage.label}</strong></div></div>
        <div className="topbar-actions">
          <label className="company-picker"><Landmark size={15} /><select aria-label="Société active" value={selectedSocieteId} onChange={(event) => { setSelectedSocieteId(event.target.value); }}><option value="">Toutes les sociétés</option>{societes.map((company) => <option key={company.id} value={company.id}>{company.nom}</option>)}</select><ChevronDown size={13} /></label>
          <div className="top-divider" />
          <label className="theme-picker"><span className="theme-dot" /> <select value={theme} onChange={(event) => setTheme(event.target.value as ThemeKey)} aria-label="Thème visuel">{Object.entries(themeNames).map(([key, name]) => <option value={key} key={key}>{name}</option>)}</select></label>
          <button className="icon-button notification-button" aria-label="Notifications" title="Les alertes automatiques arriveront dans un sprint ultérieur"><Bell size={18} /><span /></button>
          <button className="avatar top-avatar" onClick={handleLock} title="Verrouiller le coffre">SP</button>
        </div>
      </header>
      <div className="content-scroll">
        {!isDesktopRuntime() && <div className="demo-banner"><CircleAlert size={16} /><span><strong>Mode aperçu navigateur.</strong> Données temporaires, non chiffrées et non sauvegardées. Lancez <code>npm run tauri:dev</code> pour tester le coffre natif.</span></div>}
        {error && <div className="alert error global-alert"><CircleAlert size={17} /><span>{error}</span><button onClick={() => setError('')} aria-label="Fermer l'erreur"><X size={16} /></button></div>}
        {page === 'dashboard' && <Dashboard summary={summary} societes={societes} entries={entries} flux={flux} onNavigate={setPage} onNewCompany={() => { setPage('societes'); setEditingSocieteId(null); setCompanyForm(defaultCompanyForm); setShowCompanyForm(true); }} onNewFlow={() => setPage('tresorerie')} />}
        {page === 'societes' && <CompaniesPage onNew={() => { setEditingSocieteId(null); setCompanyForm(defaultCompanyForm); setShowCompanyForm(true); }} editingSocieteId={editingSocieteId} onEdit={handleEditCompany} onDelete={handleDeleteCompany} societes={societes} showForm={showCompanyForm} setShowForm={setShowCompanyForm} form={companyForm} setForm={setCompanyForm} onSubmit={handleCreateCompany} />}
        {page === 'ecritures' && <EntriesPage entries={entries.filter((entry) => !selectedSocieteId || entry.societeId === selectedSocieteId)} societes={societes} accounts={accounts} activeCompany={activeCompany} setSelectedSocieteId={setSelectedSocieteId} showForm={showEntryForm} setShowForm={setShowEntryForm} onSubmit={handleCreateEntry} entryDate={entryDate} setEntryDate={setEntryDate} entryLabel={entryLabel} setEntryLabel={setEntryLabel} entryReference={entryReference} setEntryReference={setEntryReference} debitAccount={debitAccount} setDebitAccount={setDebitAccount} creditAccount={creditAccount} setCreditAccount={setCreditAccount} entryAmount={entryAmount} setEntryAmount={setEntryAmount} busy={entryBusy} onRefresh={refreshEntries} />}
        {page === 'taches' && <TasksPage />}
        {page === 'tresorerie' && <CashflowPage societes={societes} selectedSocieteId={selectedSocieteId} flux={flux.filter(flow => !selectedSocieteId || flow.societeId === selectedSocieteId)} onNewCompany={() => { setPage('societes'); setEditingSocieteId(null); setCompanyForm(defaultCompanyForm); setShowCompanyForm(true); }} onCreate={createFlow} onUpdate={updateFlow} onRealise={realiseFlow} onCancel={cancelFlow} onCancelRealisation={cancelRealisation} onStopRecurrence={stopRecurrence} />}
        {page !== 'dashboard' && page !== 'societes' && page !== 'ecritures' && page !== 'tresorerie' && page !== 'taches' && <PlannedPage page={page} onNewCompany={() => { setPage('societes'); setEditingSocieteId(null); setCompanyForm(defaultCompanyForm); setShowCompanyForm(true); }} />}
        <footer className="page-footer"><span>SuperPower Alpha 0.1</span><span><span className="footer-dot" /> Coffre local {isDesktopRuntime() ? 'chiffré' : 'démo volatile'}</span><span>Conçu pour garder le contrôle.</span></footer>
      </div>
    </main>
    {toast && <div className="toast" aria-live="polite"><CheckCircle2 size={18} /><span>{toast.text}</span><button aria-label="Fermer le message" onClick={dismissToast}><X size={15} /></button></div>}
  </div>;
}

function Dashboard({ summary, societes, entries, flux, onNavigate, onNewCompany, onNewFlow }: {
  summary: DashboardSummary; societes: Societe[]; entries: EntryView[]; flux: FluxView[];
  onNavigate: (page: PageKey) => void; onNewCompany: () => void; onNewFlow: () => void;
}) {
  const now = new Intl.DateTimeFormat('fr-FR', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' }).format(new Date());
  const firstName = 'Bonjour';
  const hasFlux = flux.some(flow => flow.statut === 'actif');
  return <>
    <section className="welcome-row"><div><div className="eyebrow"><span className="eyebrow-line" /> VOTRE ESPACE DE PILOTAGE</div><h1>{firstName}, reprenons le contrôle<span className="title-period">.</span></h1><p>Les chiffres importants, les prochaines actions, et rien de superflu.</p></div><div className="date-chip"><CalendarDays size={16} /><span>{now}</span></div></section>
    <section className="hero-card"><div className="hero-copy"><div className="hero-kicker"><Sparkles size={15} /> LE COCKPIT SUPERPOWER</div><h2>Tout commence<br />par une base solide.</h2><p>Vos sociétés, vos écritures et vos règles de gestion réunies au même endroit. Le socle local est prêt à être construit avec vous.</p><div className="hero-buttons"><button className="button primary" onClick={onNewCompany}><Plus size={17} /> Ajouter une société</button><button className="button ghost-light" onClick={() => onNavigate('ecritures')}>Voir la comptabilité <ArrowRight size={16} /></button></div></div><div className="hero-visual" aria-hidden="true"><div className="visual-glow" /><div className="orbit orbit-one" /><div className="orbit orbit-two" /><div className="orbit orbit-three" /><div className="visual-center"><span>S</span><i>+</i></div><div className="float-card float-card-top"><Activity size={14} /><span>En local</span><strong>100%</strong></div><div className="float-card float-card-bottom"><ShieldCheck size={15} /><span>Chiffrement</span><strong>SQLCipher</strong></div></div><div className="hero-stamp"><span>ALPHA</span><strong>0.1</strong></div></section>
    <section className="metric-grid">
      <MetricCard label="Sociétés" value={String(summary.societes).padStart(2, '0')} note={summary.societes === 1 ? '1 entité configurée' : `${summary.societes} entité(s) configurée(s)`} icon={Building2} tone="violet" trend={summary.societes ? 'Actif' : 'À démarrer'} />
      <MetricCard label="Écritures validées" value={String(summary.ecrituresValidees).padStart(2, '0')} note="Écritures équilibrées et verrouillées" icon={BookOpenCheck} tone="cyan" trend="Partie double" />
      <MetricCard label="Lignes comptables" value={String(summary.lignesComptables).padStart(2, '0')} note="Détail des mouvements enregistrés" icon={Activity} tone="pink" trend="Traçables" />
      <MetricCard label="Volume comptabilisé" value={formatMoney(summary.volumeDebitCents)} note="Somme des totaux débit des écritures" icon={Landmark} tone="orange" trend="Cumul" compact />
    </section>
    <section className="dashboard-lower-grid">
      <div className="panel quick-panel"><div className="panel-heading"><div><div className="eyebrow">POUR BIEN DÉMARRER</div><h3>Les prochaines étapes</h3></div><span className="soft-count">{societes.length ? 'Socle actif' : 'Étape 1 / 3'}</span></div>
        <div className="step-list">
          <button className={`step-item ${societes.length ? 'complete' : 'current'}`} onClick={onNewCompany}><div className="step-icon">{societes.length ? <CheckCircle2 size={18} /> : <span>01</span>}</div><div className="step-copy"><strong>Configurer vos sociétés</strong><span>Forme juridique, fiscalité, TVA et clôture</span></div><ArrowRight size={16} /></button>
          <button className={`step-item ${hasFlux ? 'complete' : societes.length ? 'current' : ''}`} onClick={onNewFlow}><div className="step-icon">{hasFlux ? <CheckCircle2 size={18} /> : <span>02</span>}</div><div className="step-copy"><strong>Prévoir un premier flux</strong><span>Décrivez l'opération sans choisir de comptes</span></div><ArrowRight size={16} /></button>
          <button className="step-item planned" onClick={() => onNavigate('documents')}><div className="step-icon"><span>03</span></div><div className="step-copy"><strong>Préparer l'arrivée des documents</strong><span>Le classement intelligent vient ensuite</span></div><span className="nav-soon">Sprint 04</span></button>
        </div>
      </div>
      <div className="panel activity-panel"><div className="panel-heading"><div><div className="eyebrow">VOTRE ACTIVITÉ</div><h3>Dernières écritures</h3></div><button className="text-link" onClick={() => onNavigate('ecritures')}>Tout voir <ArrowRight size={15} /></button></div>
        {entries.length ? <div className="activity-list">{entries.slice(0, 4).map((entry) => <div className="activity-row" key={entry.id}><div className="activity-glyph"><BookOpenCheck size={16} /></div><div className="activity-main"><strong>{entry.libelle}</strong><span>{formatDate(entry.dateEcriture)} · {entry.lines.length} lignes</span></div><div className="activity-amount"><strong>{formatMoney(entry.totalCents)}</strong><span><CheckCircle2 size={12} /> Validée</span></div></div>)}</div> : <div className="empty-activity"><div className="empty-art"><BookOpenCheck size={23} /></div><strong>Votre journal est encore calme.</strong><span>La première écriture équilibrée apparaîtra ici.</span><button className="button secondary small" onClick={onNewFlow}><Plus size={15} /> Prévoir un flux</button></div>}
      </div>
    </section>
    <section className="footnote-strip"><div className="footnote-icon"><ShieldCheck size={17} /></div><div><strong>Vos données restent chez vous.</strong><span>La base native est chiffrée. Les fonctions fiscales automatiques ne seront activées qu'avec des règles explicites et vérifiables.</span></div><button className="text-link" onClick={() => onNavigate('parametres')}>Sécurité <ArrowRight size={15} /></button></section>
  </>;
}

function MetricCard({ label, value, note, icon: Icon, tone, trend, compact }: {
  label: string; value: string; note: string; icon: typeof Building2; tone: string; trend: string; compact?: boolean;
}) {
  return <article className={`metric-card tone-${tone}`}><div className="metric-top"><span>{label}</span><div className="metric-icon"><Icon size={18} /></div></div><div className={`metric-value ${compact ? 'compact' : ''}`}>{value}</div><div className="metric-bottom"><span>{note}</span><span className="metric-tag">{trend}</span></div></article>;
}

function CompaniesPage({ societes, showForm, setShowForm, form, setForm, onSubmit, editingSocieteId, onNew, onEdit, onDelete }: {
  societes: Societe[]; showForm: boolean; setShowForm: (value: boolean) => void; editingSocieteId: string | null; onNew: () => void;
  form: CreateSocieteInput; setForm: (value: CreateSocieteInput) => void; onSubmit: (event: FormEvent) => void; onEdit: (company: Societe) => void; onDelete: (company: Societe) => void;
}) {
  return <section className="module-page"><PageHeading eyebrow="RÉFÉRENTIEL JURIDIQUE" title="Vos sociétés" description="Un dossier par entité, avec son régime fiscal et ses paramètres propres." action={<button className="button primary" onClick={() => { if (showForm) setShowForm(false); else onNew(); }}>{showForm ? <X size={16} /> : <Plus size={16} />}{showForm ? 'Fermer' : 'Nouvelle société'}</button>} />
    {showForm && <form className="panel form-panel" onSubmit={onSubmit}><div className="form-heading"><div><span className="eyebrow">{editingSocieteId ? 'MODIFICATION DU DOSSIER' : 'NOUVEAU DOSSIER'}</span><h3>Informations générales</h3></div><span className="required-note">* Champs obligatoires</span></div><div className="form-grid two-cols"><Field label="Nom de la société *"><input required value={form.nom} onChange={(event) => setForm({ ...form, nom: event.target.value })} placeholder="Ex. SCI des Tilleuls" maxLength={120} /></Field><Field label="Forme juridique *"><select value={form.formeJuridique} onChange={(event) => setForm({ ...form, formeJuridique: event.target.value as FormeJuridique })}><option value="sci">SCI</option><option value="sarl">SARL</option><option value="sas">SAS</option><option value="autre">Autre</option></select></Field><Field label="Régime d'imposition du résultat *"><select value={form.regimeFiscal} onChange={(event) => setForm({ ...form, regimeFiscal: event.target.value as RegimeFiscal })}><option value="ir">Impôt sur le revenu (IR)</option><option value="is">Impôt sur les sociétés (IS)</option><option value="non_renseigne">À confirmer</option></select></Field><Field label="SIRET (facultatif)"><input inputMode="numeric" value={form.siret ?? ''} onChange={(event) => setForm({ ...form, siret: event.target.value || null })} placeholder="14 chiffres" maxLength={14} /></Field></div><div className="form-section-label"><span>Paramètres de TVA</span><small>Indépendants du régime IR / IS</small></div><div className="form-grid three-cols"><Field label="Régime de TVA"><select value={form.regimeTva} onChange={(event) => setForm({ ...form, regimeTva: event.target.value as RegimeTva })}><option value="non_renseigne">À confirmer</option><option value="hors_champ">Hors champ / non applicable</option><option value="franchise_base">Franchise en base</option><option value="reel_simplifie">Réel simplifié</option><option value="reel_normal">Réel normal</option></select></Field><Field label="Exigibilité de la TVA"><select value={form.exigibiliteTva} onChange={(event) => setForm({ ...form, exigibiliteTva: event.target.value as ExigibiliteTva })}><option value="non_definie">À confirmer</option><option value="non_applicable">Non applicable</option><option value="encaissements">Sur les encaissements</option><option value="debits">Sur les débits</option></select></Field><Field label="Périodicité déclarative"><select value={form.periodiciteTva} onChange={(event) => setForm({ ...form, periodiciteTva: event.target.value as PeriodiciteTva })}><option value="non_definie">À confirmer</option><option value="non_applicable">Non applicable</option><option value="mensuelle">Mensuelle</option><option value="trimestrielle">Trimestrielle</option><option value="annuelle">Annuelle</option></select></Field></div><div className="form-grid two-cols"><Field label="Date de clôture de l'exercice *"><input type="date" required value={form.finExercice} onChange={(event) => setForm({ ...form, finExercice: event.target.value })} /></Field><div className="inline-hint"><CircleAlert size={16} /><span>Les régimes et options doivent être confirmés à partir du dossier réel de la société. SuperPower n'effectue aucune déduction fiscale automatique à cette étape.</span></div></div><div className="form-actions"><button type="button" className="button secondary" onClick={() => setShowForm(false)}>Annuler</button><button type="submit" className="button primary"><CheckCircle2 size={16} /> {editingSocieteId ? 'Enregistrer les modifications' : 'Enregistrer la société'}</button></div></form>}
    <div className="section-summary"><div><strong>{societes.length}</strong><span>{societes.length > 1 ? 'sociétés enregistrées' : 'société enregistrée'}</span></div><span className="muted">Les paramètres peuvent être complétés au fil des sprints.</span></div>
    {societes.length ? <div className="company-grid">{societes.map((company) => <CompanyCard key={company.id} company={company} onEdit={() => onEdit(company)} onDelete={() => onDelete(company)} />)}</div> : !showForm && <EmptyState icon={Building2} title="Aucune société pour le moment" description="Créez le dossier de votre SCI ou de votre SARL. Les données fiscales sont paramétrées indépendamment." action={<button className="button primary" onClick={() => setShowForm(true)}><Plus size={16} /> Ajouter une société</button>} />}
  </section>;
}

function CompanyCard({ company, onEdit, onDelete }: { company: Societe; onEdit: () => void; onDelete: () => void }) {
  return <article className="panel company-card"><div className="company-card-top"><div className={`company-icon ${company.formeJuridique}`}><Building2 size={20} /></div><span className="status-pill"><span /> Dossier actif</span></div><h3>{company.nom}</h3><div className="company-badges"><span>{formLabel(company.formeJuridique)}</span><span>{regimeLabel(company.regimeFiscal)}</span></div><div className="company-details"><div><span>TVA</span><strong>{vatRegimeLabel(company.regimeTva)}</strong></div><div><span>Exigibilité</span><strong>{exigibilityLabel(company.exigibiliteTva)}</strong></div><div><span>Périodicité</span><strong>{periodicityLabel(company.periodiciteTva)}</strong></div><div><span>Clôture</span><strong>{formatDate(company.finExercice)}</strong></div></div>{company.warnings.length > 0 && <div className="company-warnings"><CircleAlert size={15} /><div>{company.warnings.map((warning) => <span key={warning}>{warning}</span>)}</div></div>}<div className="company-card-actions"><button type="button" className="button secondary" onClick={onEdit}><Pencil size={15} /> Modifier</button><button type="button" className="button secondary" onClick={onDelete}><Trash2 size={15} /> Supprimer</button></div></article>;
}

function EntriesPage({ entries, societes, accounts, activeCompany, setSelectedSocieteId, showForm, setShowForm, onSubmit, entryDate, setEntryDate, entryLabel, setEntryLabel, entryReference, setEntryReference, debitAccount, setDebitAccount, creditAccount, setCreditAccount, entryAmount, setEntryAmount, busy, onRefresh }: {
  entries: EntryView[]; societes: Societe[]; accounts: Account[]; activeCompany: Societe | null; setSelectedSocieteId: (value: string) => void;
  showForm: boolean; setShowForm: (value: boolean) => void; onSubmit: (event: FormEvent) => void;
  entryDate: string; setEntryDate: (value: string) => void; entryLabel: string; setEntryLabel: (value: string) => void;
  entryReference: string; setEntryReference: (value: string) => void; debitAccount: string; setDebitAccount: (value: string) => void;
  creditAccount: string; setCreditAccount: (value: string) => void; entryAmount: string; setEntryAmount: (value: string) => void;
  busy: boolean; onRefresh: () => void;
}) {
  const amountCents = parseAmountToCents(entryAmount);
  const debit = accounts.find((account) => account.code === debitAccount);
  const credit = accounts.find((account) => account.code === creditAccount);
  return <section className="module-page"><PageHeading eyebrow="JOURNAL COMPTABLE" title="Comptabilité" description="Écritures en partie double, montants en centimes et validation de l'équilibre côté Rust." action={<button className="button primary" disabled={!societes.length} onClick={() => setShowForm(!showForm)}>{showForm ? <X size={16} /> : <Plus size={16} />}{showForm ? 'Fermer' : 'Nouvelle écriture'}</button>} />
    {!societes.length && <div className="alert warning"><CircleAlert size={17} />Créez d'abord une société pour commencer à enregistrer des écritures.</div>}
    {showForm && !!societes.length && <form className="panel form-panel" onSubmit={onSubmit}><div className="form-heading"><div><span className="eyebrow">NOUVELLE ÉCRITURE</span><h3>Enregistrement équilibré</h3></div><span className="validated-pill"><ShieldCheck size={14} /> Contrôle serveur</span></div><div className="form-grid two-cols"><Field label="Société"><select value={activeCompany?.id ?? societes[0].id} disabled={societes.length === 1} onChange={(event) => setSelectedSocieteId(event.target.value)} aria-label="Société de l'écriture">{societes.map((company) => <option value={company.id} key={company.id}>{company.nom}</option>)}</select><small>Utilisez le sélecteur du haut pour changer la société active.</small></Field><Field label="Date de l'écriture *"><input type="date" required value={entryDate} onChange={(event) => setEntryDate(event.target.value)} /></Field><Field label="Libellé *"><input required value={entryLabel} onChange={(event) => setEntryLabel(event.target.value)} placeholder="Ex. Paiement facture fournisseur" maxLength={180} /></Field><Field label="Référence de pièce"><input value={entryReference} onChange={(event) => setEntryReference(event.target.value)} placeholder="Ex. FAC-2026-0042" maxLength={100} /></Field></div><div className="form-section-label"><span>Ventilation de l'écriture</span><small>Version alpha : une ligne au débit et une au crédit</small></div><div className="form-grid two-cols"><Field label="Compte débité *"><select value={debitAccount} onChange={(event) => setDebitAccount(event.target.value)}>{accounts.map((account) => <option key={account.code} value={account.code}>{account.code} · {account.label}</option>)}</select></Field><Field label="Compte crédité *"><select value={creditAccount} onChange={(event) => setCreditAccount(event.target.value)}>{accounts.map((account) => <option key={account.code} value={account.code}>{account.code} · {account.label}</option>)}</select></Field><Field label="Montant en euros *"><input inputMode="decimal" required value={entryAmount} onChange={(event) => setEntryAmount(event.target.value)} placeholder="0,00" /></Field><div className="balance-preview"><div className="balance-mini"><span>Débit</span><strong>{formatMoney(amountCents ?? 0)}</strong></div><div className="balance-equals">=</div><div className="balance-mini"><span>Crédit</span><strong>{formatMoney(amountCents ?? 0)}</strong></div><div className="balance-ok"><CheckCircle2 size={15} /> Équilibré</div></div></div>{debitAccount === creditAccount && <div className="alert warning"><CircleAlert size={16} />Sélectionnez deux comptes différents.</div>}{(!debit || !credit) && <div className="alert warning"><CircleAlert size={16} />Le plan de comptes est vide : vérifiez l'initialisation du coffre.</div>}<div className="form-actions"><button type="button" className="button secondary" onClick={() => setShowForm(false)}>Annuler</button><button type="submit" className="button primary" disabled={busy || amountCents === null || debitAccount === creditAccount || !entryLabel.trim()}>{busy ? 'Enregistrement…' : <><CheckCircle2 size={16} /> Valider l'écriture</>}</button></div><p className="form-footnote"><LockKeyhole size={14} /> Une écriture validée ne peut pas être modifiée dans cette alpha. La contre-passation sera ajoutée dans une prochaine brique.</p></form>}
    <div className="list-toolbar"><div><strong>Journal</strong><span>{entries.length} écriture(s) affichée(s)</span></div><button className="button secondary small" onClick={onRefresh}><Activity size={15} /> Actualiser</button></div>
    {entries.length ? <div className="panel entries-table-wrap"><table className="data-table"><thead><tr><th>Date</th><th>Libellé</th><th>Pièce</th><th>Lignes</th><th>Montant</th><th>Statut</th></tr></thead><tbody>{entries.map((entry) => <tr key={entry.id}><td className="date-cell">{formatDate(entry.dateEcriture)}</td><td><strong>{entry.libelle}</strong><small>{societes.find((company) => company.id === entry.societeId)?.nom ?? 'Société'}</small><details className="entry-details"><summary>Voir la ventilation</summary><div>{entry.lines.map((line, index) => <div className="entry-line" key={`${entry.id}-${index}`}><span>{line.accountCode} · {line.accountLabel}</span><strong>{line.debitCents ? `D ${formatMoney(line.debitCents)}` : `C ${formatMoney(line.creditCents)}`}</strong></div>)}</div></details></td><td>{entry.reference || '—'}</td><td>{entry.lines.length} lignes</td><td className="amount-cell">{formatMoney(entry.totalCents)}</td><td><span className="status-pill"><span /> Validée</span></td></tr>)}</tbody></table></div> : <EmptyState icon={BookOpenCheck} title="Aucune écriture enregistrée" description="Ajoutez une première écriture ; le noyau vérifie l'équilibre débit / crédit avant la persistance." action={<button className="button primary" disabled={!societes.length} onClick={() => setShowForm(true)}><Plus size={16} /> Nouvelle écriture</button>} />}
  </section>;
}


type FluxFormState = {
  societeId: string; libelle: string; sens: SensFlux; categorie: CategorieFlux;
  datePrevue: string; montantPrevu: string; tiers: string; reference: string;
  frequenceRecurrence: FrequenceFlux; dateFinRecurrence: string;
};
function localDateInput(): string {
  const now = new Date();
  return new Date(now.getTime() - now.getTimezoneOffset() * 60_000).toISOString().slice(0, 10);
}
function euroInput(cents: number): string { return (cents / 100).toFixed(2).replace('.', ','); }
function newFluxForm(societeId: string): FluxFormState {
  return {
    societeId, libelle: '', sens: 'encaissement', categorie: 'loyer',
    datePrevue: localDateInput(), montantPrevu: '', tiers: '', reference: '',
    frequenceRecurrence: 'ponctuel', dateFinRecurrence: '',
  };
}
function fluxCategoryLabel(value: CategorieFlux): string {
  return ({ loyer: 'Loyer', charge_courante: 'Charge courante', travaux: 'Travaux', impot_taxe: 'Impôt ou taxe', emprunt: 'Emprunt', apport_associe: 'Apport d’associé', autre: 'Autre' })[value];
}
function frequenceFluxLabel(value: FrequenceFlux): string {
  return ({ ponctuel: 'Ponctuel', mensuel: 'Mensuel', trimestriel: 'Trimestriel', annuel: 'Annuel' })[value];
}
function fluxStateLabel(value: FluxView['etat']): string {
  return ({ a_prevoir: 'À prévoir', en_retard: 'En retard', partiellement_realise: 'Partiellement réalisé', realise: 'Réalisé', depasse: 'Réalisé au-delà du prévu', annule: 'Prévision annulée' })[value];
}
function CashflowPage({ societes, selectedSocieteId, flux, onNewCompany, onCreate, onUpdate, onRealise, onCancel, onCancelRealisation, onStopRecurrence }: {
  societes: Societe[]; selectedSocieteId: string; flux: FluxView[]; onNewCompany: () => void;
  onCreate: (input: FluxInput) => Promise<boolean>; onUpdate: (id: string, input: FluxInput) => Promise<boolean>;
  onRealise: (fluxId: string, input: RealisationFluxInput) => Promise<boolean>;
  onCancel: (id: string) => Promise<boolean>; onCancelRealisation: (id: string, motif: string) => Promise<boolean>;
  onStopRecurrence: (recurrenceId: string, dateFin: string) => Promise<boolean>;
}) {
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [form, setForm] = useState<FluxFormState>(() => newFluxForm(selectedSocieteId || societes[0]?.id || ''));
  const [realisingId, setRealisingId] = useState<string | null>(null);
  const [realForm, setRealForm] = useState({ dateRealisation: localDateInput(), montant: '', reference: '', note: '' });
  const [localError, setLocalError] = useState('');
  const [showCancelled, setShowCancelled] = useState(false);
  const editingFlow = editingId ? flux.find(flow => flow.id === editingId) ?? null : null;
  const recurrenceLocked = Boolean(editingFlow?.recurrenceId);
  const active = flux.filter(flow => flow.statut === 'actif');
  const planIn = active.filter(flow => flow.sens === 'encaissement').reduce((sum, flow) => sum + flow.montantPrevuCents, 0);
  const planOut = active.filter(flow => flow.sens === 'decaissement').reduce((sum, flow) => sum + flow.montantPrevuCents, 0);
  const totalActual = active.reduce((sum, flow) => sum + flow.montantRealiseCents, 0);
  const shown = showCancelled ? flux : flux.filter(flow => flow.statut !== 'annule');

  function openNew() {
    setEditingId(null); setForm(newFluxForm(selectedSocieteId || societes[0]?.id || ''));
    setLocalError(''); setShowForm(true); setRealisingId(null);
  }
  function openEdit(flow: FluxView) {
    setEditingId(flow.id);
    setForm({
      societeId: flow.societeId, libelle: flow.libelle, sens: flow.sens, categorie: flow.categorie,
      datePrevue: flow.datePrevue, montantPrevu: euroInput(flow.montantPrevuCents),
      tiers: flow.tiers ?? '', reference: flow.reference ?? '',
      frequenceRecurrence: flow.frequenceRecurrence, dateFinRecurrence: flow.dateFinRecurrence ?? '',
    });
    setLocalError(''); setShowForm(true); setRealisingId(null);
  }
  async function submitFlow(event: FormEvent) {
    event.preventDefault(); setLocalError('');
    const cents = parseAmountToCents(form.montantPrevu);
    if (!form.societeId) { setLocalError('Choisissez une société.'); return; }
    if (!form.libelle.trim()) { setLocalError('Décrivez simplement l’opération.'); return; }
    if (cents === null) { setLocalError('Le montant prévu doit être positif avec au maximum deux décimales.'); return; }
    const input: FluxInput = {
      societeId: form.societeId, libelle: form.libelle.trim(), sens: form.sens, categorie: form.categorie,
      datePrevue: form.datePrevue, montantPrevuCents: cents, tiers: form.tiers.trim() || null,
      reference: form.reference.trim() || null, frequenceRecurrence: form.frequenceRecurrence,
      dateFinRecurrence: form.dateFinRecurrence || null,
    };
    const ok = editingId ? await onUpdate(editingId, input) : await onCreate(input);
    if (ok) { setShowForm(false); setEditingId(null); setLocalError(''); }
  }
  function openRealisation(flow: FluxView) {
    setRealisingId(flow.id); setShowForm(false); setLocalError('');
    setRealForm({ dateRealisation: localDateInput(), montant: flow.resteCents > 0 ? euroInput(flow.resteCents) : '', reference: '', note: '' });
  }
  async function submitRealisation(event: FormEvent, flow: FluxView) {
    event.preventDefault(); setLocalError('');
    const cents = parseAmountToCents(realForm.montant);
    if (cents === null) { setLocalError('Le montant réellement reçu ou payé doit être positif.'); return; }
    const ok = await onRealise(flow.id, { dateRealisation: realForm.dateRealisation, montantCents: cents, reference: realForm.reference.trim() || null, note: realForm.note.trim() || null });
    if (ok) { setRealisingId(null); setLocalError(''); }
  }
  async function cancelForecast(flow: FluxView) {
    const message = flow.recurrenceId
      ? `Annuler uniquement l’échéance du ${formatDate(flow.datePrevue)} pour « ${flow.libelle} » ? Les autres échéances resteront actives. Pour arrêter toute la série, utilisez « Arrêter la récurrence ».`
      : `Annuler la prévision « ${flow.libelle} » ? Les réalisations et l’historique seront conservés.`;
    if (window.confirm(message)) await onCancel(flow.id);
  }
  async function stopForecastRecurrence(flow: FluxView) {
    if (!flow.recurrenceId || !flow.recurrenceActive) return;
    const dateFin = window.prompt(
      'Dernière échéance à conserver (date exacte d’une échéance visible). Les échéances ultérieures seront annulées mais resteront dans l’historique :',
      flow.datePrevue,
    );
    if (dateFin === null || !dateFin.trim()) return;
    await onStopRecurrence(flow.recurrenceId, dateFin.trim());
  }
  async function cancelActual(item: FluxRealisationView) {
    const reason = window.prompt('Motif obligatoire de l’annulation (conservé dans l’historique) :');
    if (reason === null || !reason.trim()) return;
    await onCancelRealisation(item.id, reason.trim());
  }

  return <section className="module-page cashflow-page">
    <PageHeading eyebrow="TRÉSORERIE · PARCOURS SIMPLIFIÉ" title="Prévisionnel et réalisé" description="Décrivez les entrées et sorties d’argent comme au quotidien. Aucun numéro de compte à choisir." action={<button className="button primary" onClick={openNew} disabled={!societes.length}><Plus size={16} /> Nouvelle prévision</button>} />
    {!societes.length && <div className="panel empty-state"><div className="empty-state-icon"><Building2 size={25} /></div><h3>Commencez par votre société</h3><p>Une prévision doit être rattachée à une société.</p><button className="button primary" onClick={onNewCompany}><Plus size={16} /> Configurer une société</button></div>}
    {!!societes.length && <>
      <div className="cashflow-summary">
        <article className="panel cashflow-summary-card"><span>Prévu à encaisser</span><strong>{formatMoney(planIn)}</strong><small>{active.filter(flow => flow.sens === 'encaissement').length} flux actifs</small></article>
        <article className="panel cashflow-summary-card"><span>Prévu à payer</span><strong>{formatMoney(planOut)}</strong><small>{active.filter(flow => flow.sens === 'decaissement').length} flux actifs</small></article>
        <article className="panel cashflow-summary-card"><span>Total réalisé enregistré</span><strong>{formatMoney(totalActual)}</strong><small>Somme des réalisations actives, entrées et sorties confondues</small></article>
      </div>
      <div className="alert warning cashflow-hint"><CircleAlert size={17} /><span>Sans date de fin, une récurrence affiche les échéances sur 36 mois glissants et les totaux incluent les échéances visibles. Ce module suit le prévu et le réel ; il ne crée pas encore d’écriture comptable et ne calcule pas automatiquement la TVA.</span></div>
      {showForm && <form className="panel form-panel cashflow-form" onSubmit={submitFlow}>
        <div className="form-heading"><div><span className="eyebrow">{editingId ? 'MODIFIER LA PRÉVISION' : 'NOUVELLE PRÉVISION'}</span><h3>{editingId ? 'Mettre à jour l’opération prévue' : 'Décrire une opération attendue'}</h3></div><span className="required-note">* Champs obligatoires</span></div>
        {recurrenceLocked && <div className="alert warning"><CircleAlert size={16} /><span>Vous modifiez uniquement cette échéance. Les autres dates et montants de la série ne changent pas. Pour mettre fin à la série, utilisez « Arrêter la récurrence ».</span></div>}
        <div className="form-grid two-cols">
          <Field label="Société *"><select required disabled={!!editingId} value={form.societeId} onChange={e => setForm({ ...form, societeId: e.target.value })}>{societes.map(company => <option key={company.id} value={company.id}>{company.nom}</option>)}</select></Field>
          <Field label="Nature du flux *"><select value={form.sens} onChange={e => setForm({ ...form, sens: e.target.value as SensFlux })}><option value="encaissement">Argent qui doit entrer (encaissement)</option><option value="decaissement">Argent qui doit sortir (paiement)</option></select></Field>
          <Field label="Ce que cela concerne *"><select value={form.categorie} onChange={e => setForm({ ...form, categorie: e.target.value as CategorieFlux })}><option value="loyer">Loyer</option><option value="charge_courante">Charge courante</option><option value="travaux">Travaux</option><option value="impot_taxe">Impôt ou taxe</option><option value="emprunt">Emprunt</option><option value="apport_associe">Apport d’associé</option><option value="autre">Autre</option></select></Field>
          <Field label="Nom de l’opération *"><input required maxLength={160} value={form.libelle} onChange={e => setForm({ ...form, libelle: e.target.value })} placeholder="Ex. Loyer du local commercial" /></Field>
          <Field label="Date prévue *"><input type="date" required disabled={recurrenceLocked} value={form.datePrevue} onChange={e => setForm({ ...form, datePrevue: e.target.value })} />{recurrenceLocked && <small>La date dépend du calendrier de la série ; pour changer la fréquence, arrêtez la série puis créez une nouvelle prévision.</small>}</Field>
          <Field label="Récurrence *"><select value={form.frequenceRecurrence} disabled={recurrenceLocked} onChange={e => { const frequenceRecurrence = e.target.value as FrequenceFlux; setForm({ ...form, frequenceRecurrence, dateFinRecurrence: frequenceRecurrence === 'ponctuel' ? '' : form.dateFinRecurrence }); }}><option value="ponctuel">Ponctuel — une seule échéance</option><option value="mensuel">Mensuel</option><option value="trimestriel">Trimestriel</option><option value="annuel">Annuel</option></select>{recurrenceLocked && <small>La fréquence de la série existante se gère séparément de chaque échéance.</small>}</Field>
          {form.frequenceRecurrence !== 'ponctuel' && <Field label="Dernière échéance (facultatif)"><input type="date" min={form.datePrevue} value={form.dateFinRecurrence} disabled={recurrenceLocked} onChange={e => setForm({ ...form, dateFinRecurrence: e.target.value })} /><small>Sans date de fin, les échéances sont générées sur un horizon glissant de 36 mois.</small></Field>}
          <Field label="Montant prévu (€) *"><input required inputMode="decimal" value={form.montantPrevu} onChange={e => setForm({ ...form, montantPrevu: e.target.value })} placeholder="Ex. 1 200,00" /></Field>
          <Field label="Locataire / fournisseur / tiers"><input maxLength={160} value={form.tiers} onChange={e => setForm({ ...form, tiers: e.target.value })} placeholder="Nom du tiers" /></Field>
          <Field label="Référence (facultatif)"><input maxLength={100} value={form.reference} onChange={e => setForm({ ...form, reference: e.target.value })} placeholder="Facture ou référence interne" /></Field>
        </div>
        {localError && <div className="alert error">{localError}</div>}
        <div className="form-actions"><button type="button" className="button secondary" onClick={() => { setShowForm(false); setEditingId(null); setLocalError(''); }}>Annuler</button><button type="submit" className="button primary"><CheckCircle2 size={16} /> {editingId ? 'Enregistrer la prévision' : 'Créer la prévision'}</button></div>
      </form>}
      <div className="list-toolbar cashflow-toolbar"><div><strong>Vos flux</strong><span>{shown.length} prévision(s)</span></div><label className="cashflow-toggle"><input type="checkbox" checked={showCancelled} onChange={e => setShowCancelled(e.target.checked)} /> Afficher les prévisions annulées</label></div>
      {shown.length ? <div className="cashflow-list">{shown.map(flow => <article className={'panel cashflow-card' + (flow.statut === 'annule' ? ' is-cancelled' : '')} key={flow.id}>
        <div className="cashflow-card-header"><div className="cashflow-title-row"><span className={'cashflow-direction ' + flow.sens}>{flow.sens === 'encaissement' ? <ArrowDownLeft size={19} /> : <ArrowUpRight size={19} />}</span><div><h3>{flow.libelle}</h3><p>{formatDate(flow.datePrevue)} · {fluxCategoryLabel(flow.categorie)} · {societes.find(company => company.id === flow.societeId)?.nom ?? 'Société'} · {frequenceFluxLabel(flow.frequenceRecurrence)}</p></div></div><span className={'cashflow-status status-' + flow.etat}>{fluxStateLabel(flow.etat)}</span></div>
        <div className="cashflow-amounts"><div><span>Prévu</span><strong>{formatMoney(flow.montantPrevuCents)}</strong></div><div><span>Réalisé</span><strong>{formatMoney(flow.montantRealiseCents)}</strong></div><div><span>Écart (prévu − réalisé)</span><strong>{formatMoney(flow.resteCents)}</strong></div></div>
        {(flow.tiers || flow.reference) && <div className="cashflow-meta">{flow.tiers && <span><strong>Tiers :</strong> {flow.tiers}</span>}{flow.reference && <span><strong>Réf. :</strong> {flow.reference}</span>}</div>}
        {flow.statut === 'actif' && <div className="cashflow-actions"><button type="button" className="button secondary small" onClick={() => openEdit(flow)}><Pencil size={14} /> Modifier le prévu</button><button type="button" className="button primary small" onClick={() => openRealisation(flow)}><CheckCircle2 size={14} /> Enregistrer {flow.sens === 'encaissement' ? 'un encaissement' : 'un paiement'}</button><button type="button" className="button secondary small" onClick={() => void cancelForecast(flow)}><X size={14} /> Annuler la prévision</button></div>}
        {flow.recurrenceId && flow.recurrenceActive && <div className="cashflow-actions"><button type="button" className="button secondary small" onClick={() => void stopForecastRecurrence(flow)}><X size={14} /> Arrêter la récurrence</button></div>}
        {realisingId === flow.id && <form className="cashflow-realisation-form" onSubmit={event => void submitRealisation(event, flow)}><h4>Enregistrer ce qui s’est réellement passé</h4><p>Vous pouvez enregistrer une réalisation partielle. Un dépassement est signalé, jamais masqué.</p><div className="form-grid two-cols"><Field label="Date réelle *"><input type="date" required value={realForm.dateRealisation} onChange={e => setRealForm({ ...realForm, dateRealisation: e.target.value })} /></Field><Field label="Montant réellement reçu/payé (€) *"><input required inputMode="decimal" value={realForm.montant} onChange={e => setRealForm({ ...realForm, montant: e.target.value })} placeholder="Ex. 600,00" /></Field><Field label="Référence du paiement"><input maxLength={100} value={realForm.reference} onChange={e => setRealForm({ ...realForm, reference: e.target.value })} /></Field><Field label="Note"><input maxLength={500} value={realForm.note} onChange={e => setRealForm({ ...realForm, note: e.target.value })} placeholder="Virement partiel" /></Field></div>{localError && <div className="alert error">{localError}</div>}<div className="form-actions"><button type="button" className="button secondary" onClick={() => { setRealisingId(null); setLocalError(''); }}>Annuler</button><button type="submit" className="button primary"><CheckCircle2 size={16} /> Confirmer la réalisation</button></div></form>}
        {!!flow.realisations.length && <details className="cashflow-history"><summary>Historique des réalisations ({flow.realisations.length})</summary><div>{flow.realisations.map(item => <div className={'cashflow-realisation' + (item.annuleeAt ? ' is-cancelled' : '')} key={item.id}><div><strong>{formatDate(item.dateRealisation)} · {formatMoney(item.montantCents)}</strong><span>{item.note || item.reference || 'Réalisation enregistrée'}{item.annuleeAt ? ' · Annulée : ' + (item.motifAnnulation || 'motif absent') : ''}</span></div>{!item.annuleeAt && <button type="button" className="button secondary small" onClick={() => void cancelActual(item)}>Annuler cette réalisation</button>}</div>)}</div></details>}
      </article>)}</div> : <EmptyState icon={Wallet} title="Aucun flux prévisionnel" description="Ajoutez une entrée attendue (loyer) ou une sortie (facture, emprunt). Enregistrez ensuite les montants réels sans choisir de comptes." action={<button className="button primary" onClick={openNew}><Plus size={16} /> Créer une première prévision</button>} />}
    </>}
  </section>;
}

function TasksPage() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [taskSearch, setTaskSearch] = useState('');
  const [taskStatusFilter, setTaskStatusFilter] = useState<TaskStatusFilter>('all');
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [taskView, setTaskView] = useState<'list' | 'calendar'>('list');
  const [calendarMonth, setCalendarMonth] = useState(() => {
    const today = new Date();
    return new Date(today.getFullYear(), today.getMonth(), 1);
  });
  const [showForm, setShowForm] = useState(false);
  const [editingTaskId, setEditingTaskId] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [title, setTitle] = useState('');
  const [moduleName, setModuleName] = useState('general');
  const [taskType, setTaskType] = useState('reminder');
  const [scheduledAt, setScheduledAt] = useState(() => defaultTaskDateTime());
  const [recurrenceFrequency, setRecurrenceFrequency] = useState<TaskRecurrenceFrequency>('once');
  const [cronExpression, setCronExpression] = useState('');
  const [cronPreview, setCronPreview] = useState<{ anchor: string; expression: string; occurrences: string[] } | null>(null);
  const [cronPreviewLoading, setCronPreviewLoading] = useState(false);
  const [maxAttempts, setMaxAttempts] = useState(3);
  const [payloadJson, setPayloadJson] = useState('{}');
  const [reminderMessage, setReminderMessage] = useState('');
  const isReminderTask = moduleName.trim().toLowerCase() === 'general'
    && taskType.trim().toLowerCase() === 'reminder';
  const [historyTaskId, setHistoryTaskId] = useState<string | null>(null);
  const [executionHistory, setExecutionHistory] = useState<Record<string, TaskExecution[]>>({});
  const [attemptHistory, setAttemptHistory] = useState<Record<string, TaskAttempt[]>>({});
  const [openAttemptId, setOpenAttemptId] = useState<string | null>(null);
  const filteredTasks = useMemo(() => filterTasks(tasks, taskSearch, taskStatusFilter), [tasks, taskSearch, taskStatusFilter]);
  const [selectedCalendarOccurrence, setSelectedCalendarOccurrence] = useState<{ taskId: string; scheduledAt: string } | null>(null);

  const refresh = useCallback(async (isInitial = false) => {
    if (isInitial) setLoading(true);
    else setRefreshing(true);
    setError('');
    try {
      const result = await api<Task[]>('list_tasks');
      setTasks(result);
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, []);

  useEffect(() => {
    void refresh(true);
  }, [refresh]);

  useEffect(() => {
    if (taskView !== 'list' || !selectedCalendarOccurrence) return;
    document.getElementById('task-card-' + selectedCalendarOccurrence.taskId)?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }, [taskView, selectedCalendarOccurrence]);

  function reminderMessageFromPayloadJson(): string {
    try {
      return readReminderMessage(JSON.parse(payloadJson) as unknown);
    } catch {
      return '';
    }
  }

  function changeTaskModule(nextModule: string) {
    if (!isReminderTask && nextModule.trim().toLowerCase() === 'general'
      && taskType.trim().toLowerCase() === 'reminder') {
      setReminderMessage(reminderMessageFromPayloadJson());
    }
    setModuleName(nextModule);
  }

  function changeTaskType(nextType: string) {
    if (!isReminderTask && moduleName.trim().toLowerCase() === 'general'
      && nextType.trim().toLowerCase() === 'reminder') {
      setReminderMessage(reminderMessageFromPayloadJson());
    }
    setTaskType(nextType);
  }

  function changeReminderMessage(nextMessage: string) {
    setReminderMessage(nextMessage);
    try {
      const payload: unknown = JSON.parse(payloadJson);
      setPayloadJson(JSON.stringify(withReminderMessage(payload, nextMessage), null, 2));
    } catch {
      // Keep the JSON editor visible; save will report malformed/non-object payloads.
    }
  }

  function changePayloadJson(nextJson: string) {
    setPayloadJson(nextJson);
    if (isReminderTask) {
      try {
        setReminderMessage(readReminderMessage(JSON.parse(nextJson) as unknown));
      } catch {
        // Leave the draft message visible until the JSON is valid again.
      }
    }
  }

  function startNewTask() {
    setEditingTaskId(null);
    setTitle('');
    setModuleName('general');
    setTaskType('reminder');
    setReminderMessage('');
    setScheduledAt(defaultTaskDateTime());
    setRecurrenceFrequency('once');
    setCronExpression('');
    setCronPreview(null);
    setMaxAttempts(3);
    setPayloadJson('{}');
    setShowForm(true);
  }

  async function previewCronSchedule() {
    setError('');
    const expression = cronExpression.trim();
    if (!expression) {
      setError('Saisissez une expression cron avant de demander un aperçu.');
      return;
    }
    const anchor = new Date(scheduledAt);
    if (Number.isNaN(anchor.getTime())) {
      setError('Choisissez une date de départ valide.');
      return;
    }
    setCronPreviewLoading(true);
    setCronPreview(null);
    try {
      const occurrences = await api<string[]>('preview_cron_schedule', {
        expression,
        after: anchor.toISOString(),
        count: 5,
      });
      setCronPreview({ anchor: anchor.toISOString(), expression, occurrences });
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setCronPreviewLoading(false);
    }
  }

  async function startEditingTask(task: Task) {
    setError('');
    setNotice('');
    if (task.status === 'cancelled') {
      setError('Réactivez la tâche avant de la modifier.');
      return;
    }
    try {
      const executions = await api<TaskExecution[]>('list_task_executions', { taskId: task.id });
      if (executions.length > 0) {
        setError('Cette tâche possède déjà un historique d’exécution. Pour préserver cet historique, annulez-la et créez une nouvelle tâche.');
        return;
      }
      setEditingTaskId(task.id);
      setCronPreview(null);
      setTitle(task.title);
      setModuleName(task.module);
      setTaskType(task.taskType);
      setReminderMessage(readReminderMessage(task.payload));
      const taskFrequency = task.recurrenceFrequency ?? (task.cronExpression ? 'custom_cron' : 'once');
      setRecurrenceFrequency(taskFrequency);
      setScheduledAt(toLocalDateTimeInput(taskFrequency !== 'once' ? task.cronAnchorAt ?? task.scheduledAt : task.scheduledAt));
      setCronExpression(task.cronExpression ?? '');
      setMaxAttempts(task.maxAttempts);
      setPayloadJson(JSON.stringify(task.payload, null, 2) ?? '{}');
      setShowForm(true);
    } catch (reason) {
      setError(errorMessage(reason));
    }
  }

  function toggleTaskForm() {
    if (showForm) {
      setShowForm(false);
      setEditingTaskId(null);
    } else {
      startNewTask();
    }
  }

  async function saveTask(event: FormEvent) {
    event.preventDefault();
    setSubmitting(true);
    setError('');
    setNotice('');
    const wasEditing = editingTaskId !== null;
    try {
      const parsedPayload: unknown = JSON.parse(payloadJson);
      const usesReminderHandler = moduleName.trim().toLowerCase() === 'general'
        && taskType.trim().toLowerCase() === 'reminder';
      const taskPayload = usesReminderHandler
        ? withReminderMessage(parsedPayload, reminderMessage)
        : parsedPayload;
      const plannedDate = new Date(scheduledAt);
      if (Number.isNaN(plannedDate.getTime())) throw new Error('Choisissez une date de planification valide.');
      if (recurrenceFrequency === 'custom_cron' && !cronExpression.trim()) throw new Error('Saisissez une expression cron pour utiliser le mode avancé.');
      const existingTask = editingTaskId ? tasks.find((task) => task.id === editingTaskId) : undefined;
      if (editingTaskId && !existingTask) throw new Error('Cette tâche n’existe plus. Actualisez la liste.');
      const input: NewTask = {
        idempotencyKey: existingTask?.idempotencyKey ?? 'manual-' + crypto.randomUUID(),
        module: moduleName,
        taskType,
        title: title.trim(),
        payload: taskPayload,
        scheduledAt: plannedDate.toISOString(),
        recurrenceFrequency,
        cronExpression: recurrenceFrequency === 'custom_cron' ? cronExpression.trim() || null : null,
        maxAttempts,
      };
      const saved = existingTask
        ? await api<Task>('update_task', { taskId: existingTask.id, input })
        : await api<Task>('create_task', { input });
      setTasks((current) => [saved, ...current.filter((task) => task.id !== saved.id)]
        .sort((a, b) => a.scheduledAt.localeCompare(b.scheduledAt)));
      setTitle('');
      setTaskType('reminder');
      setReminderMessage('');
      setScheduledAt(defaultTaskDateTime());
      setRecurrenceFrequency('once');
      setCronExpression('');
      setCronPreview(null);
      setMaxAttempts(3);
      setPayloadJson('{}');
      setEditingTaskId(null);
      setShowForm(false);
      setNotice(wasEditing ? 'Modifications enregistrées dans le coffre local.' : 'Tâche enregistrée dans le coffre local.');
    } catch (reason) {
      setError(reason instanceof SyntaxError ? 'Le contenu JSON est invalide.' : errorMessage(reason));
    } finally {
      setSubmitting(false);
    }
  }

  async function changeStatus(task: Task, status: TaskStatus) {
    setError('');
    setNotice('');
    try {
      const updated = await api<Task>('update_task_status', { taskId: task.id, status });
      setTasks((current) => current.map((item) => item.id === updated.id ? updated : item));
      setNotice(status === 'cancelled' ? 'Tâche annulée ; son historique est conservé.' : status === 'paused' ? 'Tâche mise en pause.' : 'Tâche réactivée.');
    } catch (reason) {
      setError(errorMessage(reason));
    }
  }

  async function deleteTask(task: Task) {
    setError('');
    setNotice('');
    if (!window.confirm(`Supprimer définitivement « ${task.title} » ? Cette action est possible uniquement avant la première exécution ; elle sera inscrite au journal d’audit.`)) {
      return;
    }
    try {
      const executions = await api<TaskExecution[]>('list_task_executions', { taskId: task.id });
      if (executions.length > 0) {
        throw new Error('Cette tâche possède un historique d’exécution. Annulez-la pour préserver cet historique.');
      }
      await api<void>('delete_task', { taskId: task.id });
      setTasks((current) => current.filter((item) => item.id !== task.id));
      setExecutionHistory((current) => {
        const next = { ...current };
        delete next[task.id];
        return next;
      });
      if (historyTaskId === task.id) setHistoryTaskId(null);
      setNotice('Tâche supprimée. L’action est inscrite au journal d’audit.');
    } catch (reason) {
      setError(errorMessage(reason));
    }
  }

  function selectCalendarOccurrence(taskId: string, scheduledAt: string) {
    setSelectedCalendarOccurrence({ taskId, scheduledAt });
    setTaskSearch('');
    setTaskStatusFilter('all');
    setTaskView('list');
    setError('');
    setNotice(`Occurrence du ${new Date(scheduledAt).toLocaleString('fr-FR')} sélectionnée : la tâche correspondante est mise en évidence dans la liste.`);
  }

  async function toggleExecutionHistory(taskId: string) {
    if (historyTaskId === taskId) {
      setHistoryTaskId(null);
      return;
    }
    setError('');
    try {
      const rows = await api<TaskExecution[]>('list_task_executions', { taskId });
      setExecutionHistory((current) => ({ ...current, [taskId]: rows }));
      setHistoryTaskId(taskId);
      setOpenAttemptId(null);
    } catch (reason) {
      setError(errorMessage(reason));
    }
  }

  async function toggleAttempts(executionId: string) {
    if (openAttemptId === executionId) {
      setOpenAttemptId(null);
      return;
    }
    setError('');
    try {
      const rows = await api<TaskAttempt[]>('list_task_attempts', { executionId });
      setAttemptHistory((current) => ({ ...current, [executionId]: rows }));
      setOpenAttemptId(executionId);
    } catch (reason) {
      setError(errorMessage(reason));
    }
  }

  return <section className="module-page tasks-page">
    <PageHeading
      eyebrow="ORGANISATION ET SUIVI"
      title="Tâches"
      description="Des actions planifiées, conservées dans votre coffre, avec un historique consultable."
      action={<button className="button primary" onClick={toggleTaskForm}><Plus size={16} /> Nouvelle tâche</button>}
    />
    <div className="panel tasks-notice">
      <ShieldCheck size={18} />
      <div><strong>Rappels locaux automatisés</strong><p>Le worker démarre après le déverrouillage du coffre et s’arrête au verrouillage ou à la fermeture. Le type « Général / reminder » affiche un rappel dans l’application à l’échéance ; les autres types restent en attente tant qu’un handler dédié n’est pas disponible. Les tâches et leur historique restent dans le coffre SQLCipher.</p></div>
    </div>
    {!isDesktopRuntime() && <div className="alert warning"><CircleAlert size={16} /><span>Mode aperçu : les tâches restent en mémoire et disparaissent au rechargement. Le coffre natif est nécessaire pour valider la persistance chiffrée.</span></div>}
    {error && <div className="alert error"><CircleAlert size={17} /><span>{error}</span><button onClick={() => setError('')} aria-label="Fermer l’erreur"><X size={16} /></button></div>}
    {notice && <div className="alert success"><CheckCircle2 size={17} /><span>{notice}</span><button onClick={() => setNotice('')} aria-label="Fermer le message"><X size={16} /></button></div>}
    {showForm && <form className="panel tasks-form" onSubmit={saveTask}>
      <div className="tasks-form-heading"><div><h2>{editingTaskId ? 'Modifier la tâche' : 'Nouvelle tâche'}</h2><p>{editingTaskId ? 'Modification possible avant la première exécution, sans altérer l’historique.' : 'Un rappel « Général / reminder » s’affichera dans l’application si elle reste ouverte à l’échéance.'}</p></div><button type="button" className="icon-button" onClick={() => { setShowForm(false); setEditingTaskId(null); }} aria-label="Fermer le formulaire"><X size={17} /></button></div>
      <div className="tasks-form-grid">
        <Field label="Titre"><input value={title} onChange={(event) => setTitle(event.target.value)} maxLength={200} required placeholder="Ex. Vérifier les pièces de novembre" /></Field>
        <Field label="Module"><select value={moduleName} onChange={(event) => changeTaskModule(event.target.value)}><option value="general">Général</option><option value="documents">Documents</option><option value="import_bancaire">Import bancaire</option><option value="locatif">Gestion locative</option><option value="fiscalite">Fiscalité</option></select></Field>
        <Field label="Type technique"><input value={taskType} onChange={(event) => changeTaskType(event.target.value)} maxLength={100} required placeholder="Ex. reminder" /><small>Seul « Général / reminder » est exécuté automatiquement ; les autres types restent en attente.</small></Field>
       <Field label="Fréquence"><select value={recurrenceFrequency} onChange={(event) => { setRecurrenceFrequency(event.target.value as TaskRecurrenceFrequency); setCronExpression(''); setCronPreview(null); }}>
         <option value="once">Ponctuelle — une seule échéance</option><option value="monthly">Mensuelle</option><option value="quarterly">Trimestrielle</option><option value="yearly">Annuelle</option><option value="custom_cron">Avancée — expression cron UTC</option>
       </select><small>Les fréquences métier repartent de la date d'origine. Exemple : 31 janvier → 28 février → 31 mars.</small></Field>
       <Field label={recurrenceFrequency === 'once' ? 'Date et heure prévues' : 'Date et heure de départ'}><input type="datetime-local" value={scheduledAt} onChange={(event) => { setScheduledAt(event.target.value); setCronPreview(null); }} required /></Field>
       {recurrenceFrequency === 'custom_cron' && <Field label="Expression cron (UTC)"><input value={cronExpression} onChange={(event) => { setCronExpression(event.target.value); setCronPreview(null); }} maxLength={200} required placeholder="*/15 9-17 * * 1-5" /><div className="cron-preview-actions"><button type="button" className="button secondary small" onClick={() => void previewCronSchedule()} disabled={!isDesktopRuntime() || cronPreviewLoading || !cronExpression.trim() || !scheduledAt}>{cronPreviewLoading ? 'Calcul des occurrences…' : 'Prévisualiser les prochaines échéances'}</button>{!isDesktopRuntime() && <span>Disponible dans l’application de bureau.</span>}</div></Field>}
        <Field label="Nombre maximal de tentatives"><input type="number" value={maxAttempts} onChange={(event) => setMaxAttempts(Number(event.target.value))} min={1} max={100} required /></Field>
      </div>
      {cronPreview && <div className="cron-preview-results" aria-live="polite">
        <p><strong>Ancre choisie :</strong> {formatUtcDateTime(cronPreview.anchor)} UTC.</p>
        <p><strong>Échéances prévues</strong> pour <code>{cronPreview.expression}</code> :</p>
        <ol>{cronPreview.occurrences.map((occurrence) => <li key={occurrence}>{formatUtcDateTime(occurrence)} UTC</li>)}</ol>
      </div>}
      {isReminderTask && <Field label="Message du rappel (facultatif)">
        <textarea value={reminderMessage} onChange={(event) => changeReminderMessage(event.target.value)} rows={2} maxLength={300} placeholder="Ex. Relancer le locataire pour le loyer de novembre" />
        <small>Maximum 300 caractères. Si le champ est vide, le titre de la tâche sera affiché.</small>
      </Field>}
      <Field label={isReminderTask ? 'Données avancées (JSON)' : 'Données de la tâche (JSON)'}>
        <textarea value={payloadJson} onChange={(event) => changePayloadJson(event.target.value)} rows={3} spellCheck={false} required />
        {isReminderTask && <small>Le message ci-dessus est synchronisé avec la propriété JSON « message ». Les autres propriétés sont conservées.</small>}
      </Field>
      <div className="tasks-form-actions"><button type="button" className="button secondary" onClick={() => { setShowForm(false); setEditingTaskId(null); }}>Annuler</button><button className="button primary" type="submit" disabled={submitting || !title.trim() || !taskType.trim() || maxAttempts < 1 || maxAttempts > 100}>{submitting ? 'Enregistrement…' : editingTaskId ? 'Enregistrer les modifications' : 'Enregistrer la tâche'} <ArrowRight size={16} /></button></div>
    </form>}
    <div className="tasks-list-heading">
      <div><h2>{taskView === 'calendar' ? 'Calendrier des tâches' : 'Liste des tâches'}</h2><p>{taskView === 'list' ? `${filteredTasks.length} sur ${tasks.length} tâche${tasks.length > 1 ? 's' : ''}` : `${tasks.length} tâche${tasks.length > 1 ? 's' : ''} enregistrée${tasks.length > 1 ? 's' : ''}`}</p></div>
      <div className="tasks-heading-controls">
        <div className="tasks-view-toggle" role="group" aria-label="Mode d’affichage des tâches">
          <button type="button" className={taskView === 'list' ? 'button primary' : 'button secondary'} onClick={() => setTaskView('list')}><Menu size={15} /> Liste</button>
          <button type="button" className={taskView === 'calendar' ? 'button primary' : 'button secondary'} onClick={() => setTaskView('calendar')}><CalendarDays size={15} /> Calendrier</button>
        </div>
        <button className="button secondary" onClick={() => void refresh()} disabled={refreshing}><Activity size={15} /> {refreshing ? 'Actualisation…' : 'Actualiser'}</button>
      </div>
    </div>
    {taskView === 'list' && tasks.length > 0 && <div className="tasks-filters">
      <label className="tasks-filter-field"><span>Rechercher</span><input type="search" value={taskSearch} onChange={(event) => setTaskSearch(event.target.value)} placeholder="Titre, module ou type technique" aria-label="Rechercher une tâche par titre, module ou type" /></label>
      <label className="tasks-filter-field"><span>Statut</span><select value={taskStatusFilter} onChange={(event) => setTaskStatusFilter(event.target.value as TaskStatusFilter)} aria-label="Filtrer les tâches par statut"><option value="all">Tous les statuts</option><option value="scheduled">Planifiées</option><option value="paused">En pause</option><option value="cancelled">Annulées</option></select></label>
      {(taskSearch.trim() || taskStatusFilter !== 'all') && <button type="button" className="button ghost tasks-clear-filters" onClick={() => { setTaskSearch(''); setTaskStatusFilter('all'); }}>Effacer les filtres</button>}
    </div>}
    {loading ? <div className="panel tasks-empty"><p>Chargement des tâches…</p></div> : tasks.length === 0
      ? <EmptyState icon={Clock3} title="Aucune tâche pour le moment" description="Créez une tâche pour commencer à organiser les actions à suivre." action={<button className="button primary" onClick={() => setShowForm(true)}><Plus size={16} /> Créer une tâche</button>} />
      : taskView === 'calendar'
        ? <TaskCalendar tasks={tasks} month={calendarMonth} onMonthChange={setCalendarMonth} selectedOccurrence={selectedCalendarOccurrence} onSelectOccurrence={selectCalendarOccurrence} />
        : filteredTasks.length === 0
          ? <EmptyState icon={Clock3} title="Aucune tâche correspondante" description="Modifiez votre recherche ou choisissez un autre statut." action={<button className="button secondary" onClick={() => { setTaskSearch(''); setTaskStatusFilter('all'); }}>Effacer les filtres</button>} />
          : <div className="tasks-list">{filteredTasks.map((task) => <article id={'task-card-' + task.id} className={'panel task-card' + (selectedCalendarOccurrence?.taskId === task.id ? ' is-calendar-selected' : '')} key={task.id}>
          <div className="task-card-main">
            <div className="task-card-icon"><Clock3 size={19} /></div>
            <div className="task-card-copy"><div className="task-card-title-row"><h3>{task.title}</h3><span className={`task-status task-status-${task.status}`}>{taskStatusLabel(task.status)}</span>{selectedCalendarOccurrence?.taskId === task.id && <span className="task-calendar-selection">Occurrence sélectionnée : {new Date(selectedCalendarOccurrence.scheduledAt).toLocaleString('fr-FR')}</span>}</div><p>{new Date(task.scheduledAt).toLocaleString('fr-FR')} · {task.module} / {task.taskType}{(task.recurrenceFrequency ?? (task.cronExpression ? 'custom_cron' : 'once')) === 'custom_cron' && task.cronExpression ? <> · Cron UTC : {task.cronExpression}</> : (task.recurrenceFrequency ?? 'once') !== 'once' ? <> · {taskRecurrenceLabels[task.recurrenceFrequency ?? 'once']}</> : ''}</p><div className="task-card-meta"><span>Jusqu’à {task.maxAttempts} tentatives</span><span>Créée le {new Date(task.createdAt).toLocaleDateString('fr-FR')}</span></div></div>
          </div>
          <div className="task-card-actions">
            {task.status !== 'cancelled' && <button className="button secondary" onClick={() => void startEditingTask(task)}>Modifier</button>}
            {task.status !== 'cancelled' && <button className="button secondary" onClick={() => void changeStatus(task, task.status === 'paused' ? 'scheduled' : 'paused')}>{task.status === 'paused' ? 'Reprendre' : 'Mettre en pause'}</button>}
            {task.status === 'cancelled' ? <button className="button secondary" onClick={() => void changeStatus(task, 'scheduled')}>Réactiver</button> : <button className="button danger-ghost" onClick={() => void changeStatus(task, 'cancelled')}>Annuler</button>}
            <button className="button ghost" onClick={() => void toggleExecutionHistory(task.id)}>{historyTaskId === task.id ? 'Masquer l’historique' : 'Historique'}</button>
            {(executionHistory[task.id] ?? []).length === 0
              ? <button className="button danger-ghost" onClick={() => void deleteTask(task)}>Supprimer</button>
              : null}
          </div>
          {historyTaskId === task.id && <div className="task-history"><h4>Historique d’exécution</h4>{(executionHistory[task.id] ?? []).length === 0 ? <p>Aucune exécution enregistrée. Le worker de fond n’est pas encore lancé automatiquement.</p> : (executionHistory[task.id] ?? []).map((execution) => <div className="task-execution" key={execution.id}>
              <div><strong>{new Date(execution.scheduledFor).toLocaleString('fr-FR')}</strong><span>{taskExecutionStatusLabel(execution.status)} · {execution.attemptCount}/{execution.maxAttempts} tentative(s)</span>{execution.lastError && <p className="task-error-detail">{execution.lastError}</p>}</div>
              <button className="button ghost" onClick={() => void toggleAttempts(execution.id)}>{openAttemptId === execution.id ? 'Masquer les tentatives' : 'Tentatives'}</button>
              {openAttemptId === execution.id && <div className="task-attempts">{(attemptHistory[execution.id] ?? []).length === 0 ? <p>Aucune tentative historisée.</p> : (attemptHistory[execution.id] ?? []).map((attempt) => <div className="task-attempt" key={attempt.id}><span>#{attempt.attemptNumber} · {taskAttemptStatusLabel(attempt.status)}</span><span>{new Date(attempt.startedAt).toLocaleString('fr-FR')}</span>{attempt.errorMessage && <p>{attempt.errorMessage}</p>}</div>)}</div>}
            </div>)}</div>}
        </article>)}</div>}
  </section>;
}

function TaskCalendar({
  tasks,
  month,
  onMonthChange,
  selectedOccurrence,
  onSelectOccurrence,
}: {
  tasks: Task[];
  month: Date;
  onMonthChange: (month: Date) => void;
  selectedOccurrence: { taskId: string; scheduledAt: string } | null;
  onSelectOccurrence: (taskId: string, scheduledAt: string) => void;
}) {
  const year = month.getFullYear();
  const monthIndex = month.getMonth();
  const firstOfMonth = new Date(year, monthIndex, 1);
  const mondayOffset = (firstOfMonth.getDay() + 6) % 7;
  const gridStart = new Date(year, monthIndex, 1 - mondayOffset);
  const days = Array.from(
    { length: 42 },
    (_, index) => new Date(gridStart.getFullYear(), gridStart.getMonth(), gridStart.getDate() + index),
  );
  const tasksByDay = useMemo(() => {
    const grouped = new Map<string, Array<{ task: Task; scheduledAt: string }>>();
    const rangeStart = days[0];
    const lastDay = days[days.length - 1];
    const rangeEndExclusive = new Date(
      lastDay.getFullYear(),
      lastDay.getMonth(),
      lastDay.getDate() + 1,
    );

    for (const task of tasks) {
      for (const scheduledAt of taskCalendarOccurrenceDates(task, rangeStart, rangeEndExclusive)) {
        const date = new Date(scheduledAt);
        if (Number.isNaN(date.getTime())) continue;
        const key = localDateKey(date);
        const dayTasks = grouped.get(key) ?? [];
        dayTasks.push({ task, scheduledAt });
        grouped.set(key, dayTasks);
      }
    }

    for (const dayTasks of grouped.values()) {
      dayTasks.sort((a, b) => a.scheduledAt.localeCompare(b.scheduledAt));
    }
    return grouped;
  }, [tasks, month]);
  const todayKey = localDateKey(new Date());

  return <div className="panel tasks-calendar-panel">
    <div className="tasks-calendar-toolbar">
      <button className="button secondary" type="button" onClick={() => onMonthChange(new Date(year, monthIndex - 1, 1))} aria-label="Mois précédent"><ChevronLeft size={16} /></button>
      <h3>{firstOfMonth.toLocaleDateString('fr-FR', { month: 'long', year: 'numeric' })}</h3>
      <button className="button secondary" type="button" onClick={() => onMonthChange(new Date(year, monthIndex + 1, 1))} aria-label="Mois suivant"><ChevronRight size={16} /></button>
      <button className="button ghost" type="button" onClick={() => {
        const today = new Date();
        onMonthChange(new Date(today.getFullYear(), today.getMonth(), 1));
      }}>Aujourd’hui</button>
    </div>
    <div className="tasks-calendar-shell">
      <div className="tasks-calendar-grid">
        {['Lun', 'Mar', 'Mer', 'Jeu', 'Ven', 'Sam', 'Dim'].map((weekday) =>
          <div className="tasks-calendar-weekday" key={weekday}>{weekday}</div>
        )}
        {days.map((date) => {
          const key = localDateKey(date);
          const dayTasks = tasksByDay.get(key) ?? [];
          const className = [
            'tasks-calendar-day',
            date.getMonth() !== monthIndex ? 'is-outside-month' : '',
            key === todayKey ? 'is-today' : '',
          ].filter(Boolean).join(' ');
          return <div className={className} key={key} aria-label={date.toLocaleDateString('fr-FR')}>
            <span className="tasks-calendar-day-number">{date.getDate()}</span>
            <div className="tasks-calendar-events">
              {dayTasks.slice(0, 3).map(({ task, scheduledAt }) =>
                <button
                  type="button"
                  className={'tasks-calendar-event task-status-' + task.status + (selectedOccurrence?.taskId === task.id && selectedOccurrence.scheduledAt === scheduledAt ? ' is-selected' : '')}
                  key={task.id + '-' + scheduledAt}
                  title={task.title + ' · ' + new Date(scheduledAt).toLocaleString('fr-FR') + ' · ' + taskStatusLabel(task.status)}
                  aria-label={'Sélectionner ' + task.title + ', occurrence du ' + new Date(scheduledAt).toLocaleString('fr-FR')}
                  aria-pressed={selectedOccurrence?.taskId === task.id && selectedOccurrence.scheduledAt === scheduledAt}
                  onClick={() => onSelectOccurrence(task.id, scheduledAt)}
                >
                  {task.title}
                </button>
              )}
              {dayTasks.length > 3 && <span className="tasks-calendar-overflow">+{dayTasks.length - 3} autre{dayTasks.length > 4 ? 's' : ''}</span>}
            </div>
          </div>;
        })}
      </div>
    </div>
    <p className="tasks-calendar-footnote">Les tâches ponctuelles et les récurrences mensuelles, trimestrielles et annuelles s’affichent aux dates prévues dans la grille. Le mode cron personnalisé affiche seulement son échéance courante ; son aperçu détaillé est disponible dans l’édition. Choisissez une occurrence pour passer à la liste, mettre en évidence la tâche et consulter son historique ou la modifier. Les dates sont affichées dans le fuseau horaire local de cet ordinateur.</p>
  </div>;
}

function localDateKey(value: Date): string {
  const year = value.getFullYear();
  const month = String(value.getMonth() + 1).padStart(2, '0');
  const day = String(value.getDate()).padStart(2, '0');
  return year + '-' + month + '-' + day;
}

function defaultTaskDateTime(): string {
  const value = new Date(Date.now() + 5 * 60_000);
  value.setSeconds(0, 0);
  return new Date(value.getTime() - value.getTimezoneOffset() * 60_000).toISOString().slice(0, 16);
}

function toLocalDateTimeInput(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return defaultTaskDateTime();
  return new Date(date.getTime() - date.getTimezoneOffset() * 60_000).toISOString().slice(0, 16);
}

function taskStatusLabel(status: TaskStatus): string {
  return ({ scheduled: 'Planifiée', paused: 'En pause', cancelled: 'Annulée' })[status];
}

function taskExecutionStatusLabel(status: TaskExecution['status']): string {
  return ({ queued: 'En attente', running: 'En cours', succeeded: 'Terminée', failed: 'Échouée', cancelled: 'Annulée' })[status];
}

function taskAttemptStatusLabel(status: TaskAttempt['status']): string {
  return ({ running: 'En cours', succeeded: 'Réussie', failed: 'Échouée' })[status];
}

function PlannedPage({ page, onNewCompany }: { page: PageKey; onNewCompany: () => void }) {
  const meta: Record<Exclude<PageKey, 'dashboard' | 'societes' | 'ecritures' | 'taches'>, { eyebrow: string; title: string; description: string; sprint: string; icon: typeof Files; features: string[] }> = {
    documents: { eyebrow: 'CENTRE DOCUMENTAIRE', title: 'Documents', description: 'Une boîte d’entrée pour vos pièces, avec un classement explicable et vérifiable.', sprint: 'Sprint 04', icon: Files, features: ['Import PDF et fichiers bureautiques', 'Détection des doublons', 'Classification par règles et score de confiance'] },
    tresorerie: { eyebrow: 'ANTICIPATION FINANCIÈRE', title: 'Trésorerie', description: 'Projeter les entrées et sorties, comparer les prévisions à la banque et anticiper les tensions.', sprint: 'Sprint 06', icon: Wallet, features: ['Flux récurrents et échéances', 'Projection à 12, 24 et 36 mois', 'Alertes de seuil et rapprochement du réel'] },
    locatif: { eyebrow: 'GESTION IMMOBILIÈRE', title: 'Gestion locative', description: 'Baux, loyers, charges et échéances réunis dans un suivi adapté aux lots immobiliers.', sprint: 'Sprint 05', icon: House, features: ['Biens, lots, baux et locataires', 'Appels de loyers, encaissements et impayés', 'Révisions contractuelles et suivi des justificatifs'] },
    fiscalite: { eyebrow: 'CALENDRIER ET PRÉPARATION', title: 'Fiscalité', description: 'Préparer les échéances, rapprocher les sources et produire des brouillons justifiables — jamais des déclarations automatiques opaques.', sprint: 'Sprint 07', icon: ReceiptText, features: ['TVA selon le régime et l’exigibilité configurés', 'Préparation des formulaires applicables (2072, 2044, 2065…)', 'Dates de dépôt vérifiées et liens vers les sources officielles'] },
    parametres: { eyebrow: 'CONTRÔLE ET CONFIDENTIALITÉ', title: 'Paramètres', description: 'Votre base vit en local. Cette page accueillera la sauvegarde, le contrôle de sécurité et les réglages du coffre.', sprint: 'Socle / Sprint 00', icon: ShieldCheck, features: ['Base SQLCipher et phrase secrète', 'Sauvegarde conjointe de la base et du fichier de sel', 'Journal d’audit et préférences de confidentialité'] },
  };
  const item = meta[page as keyof typeof meta];
  const Icon = item.icon;
  return <section className="module-page planned-page"><PageHeading eyebrow={item.eyebrow} title={item.title} description={item.description} action={<span className="sprint-badge"><Clock3 size={14} /> {item.sprint}</span>} /><div className="planned-hero panel"><div className="planned-illustration"><div className="planned-orbit"><Icon size={31} /></div><span className="planned-star star-a">✦</span><span className="planned-star star-b">✧</span><span className="planned-star star-c">✦</span></div><div className="planned-copy"><span className="status-label">PLANIFIÉ · NON LIVRÉ DANS CETTE ALPHA</span><h2>Le socle est posé.<br />Le métier arrive ensuite.</h2><p>Le module est référencé dans la roadmap, mais les traitements et écrans de production ne sont pas encore implémentés. Aucun calcul ni dépôt fiscal n'est simulé.</p><div className="planned-feature-list">{item.features.map((feature) => <div key={feature}><CheckCircle2 size={16} /><span>{feature}</span></div>)}</div>{page === 'parametres' && <button className="button secondary" onClick={onNewCompany}><Building2 size={16} /> Gérer les sociétés</button>}</div></div><div className="planned-note"><ShieldCheck size={17} /><span>Les paramètres fiscaux doivent être confirmés pour chaque entité et activité. Les futures règles conserveront les hypothèses, les sources et la date de vérification.</span></div></section>;
}

function PageHeading({ eyebrow, title, description, action }: { eyebrow: string; title: string; description: string; action?: ReactNode }) {
  return <div className="module-heading"><div><div className="eyebrow"><span className="eyebrow-line" /> {eyebrow}</div><h1>{title}<span className="title-period">.</span></h1><p>{description}</p></div>{action && <div className="heading-action">{action}</div>}</div>;
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return <label className="form-field"><span>{label}</span>{children}</label>;
}

function EmptyState({ icon: Icon, title, description, action }: { icon: typeof Building2; title: string; description: string; action: ReactNode }) {
  return <div className="panel empty-state"><div className="empty-state-icon"><Icon size={25} /></div><h3>{title}</h3><p>{description}</p>{action}</div>;
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return 'Une erreur inattendue est survenue.';
}
function formLabel(value: FormeJuridique): string { return ({ sci: 'SCI', sarl: 'SARL', sas: 'SAS', autre: 'Autre' })[value]; }
function regimeLabel(value: RegimeFiscal): string { return ({ ir: 'IR', is: 'IS', non_renseigne: 'Régime à confirmer' })[value]; }
function vatRegimeLabel(value: RegimeTva): string { return ({ non_renseigne: 'À confirmer', hors_champ: 'Hors champ', franchise_base: 'Franchise en base', reel_simplifie: 'Réel simplifié', reel_normal: 'Réel normal' })[value]; }
function exigibilityLabel(value: ExigibiliteTva): string { return ({ non_definie: 'À confirmer', non_applicable: 'Non applicable', encaissements: 'Encaissements', debits: 'Débits' })[value]; }
function periodicityLabel(value: PeriodiciteTva): string { return ({ non_definie: 'À confirmer', non_applicable: 'Non applicable', mensuelle: 'Mensuelle', trimestrielle: 'Trimestrielle', annuelle: 'Annuelle' })[value]; }

export default App;
