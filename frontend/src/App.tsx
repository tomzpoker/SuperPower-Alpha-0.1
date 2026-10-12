import { useCallback, useEffect, useMemo, useState } from 'react';
import type { FormEvent, ReactNode } from 'react';
import {
  Activity, ArrowRight, Bell, BookOpenCheck,
  Building2, CalendarDays, CheckCircle2, ChevronDown, CircleAlert, Clock3,
  Files, House, Landmark, LayoutDashboard, LockKeyhole, LogOut,
  Menu, Pencil, Plus, ReceiptText, ShieldCheck, Sparkles, Trash2, Wallet, X,
} from 'lucide-react';
import { api, isDesktopRuntime } from './lib/api';
import { formatDate, formatMoney, parseAmountToCents } from './lib/money';
import type {
  Account, CreateEntryInput, CreateSocieteInput, DashboardSummary, DatabaseStatus,
  EntryView, ExigibiliteTva, FormeJuridique, PeriodiciteTva, RegimeFiscal, RegimeTva, Societe,
} from './lib/types';

type PageKey = 'dashboard' | 'societes' | 'ecritures' | 'documents' | 'tresorerie' | 'locatif' | 'fiscalite' | 'parametres';
type ThemeKey = 'aurora' | 'sakura' | 'mint' | 'sunset' | 'midnight';

const navigation: Array<{ key: PageKey; label: string; icon: typeof LayoutDashboard; group: string }> = [
  { key: 'dashboard', label: 'Cockpit', icon: LayoutDashboard, group: 'ESPACE DE TRAVAIL' },
  { key: 'societes', label: 'Sociétés', icon: Building2, group: 'ESPACE DE TRAVAIL' },
  { key: 'ecritures', label: 'Comptabilité', icon: BookOpenCheck, group: 'ESPACE DE TRAVAIL' },
  { key: 'documents', label: 'Documents', icon: Files, group: 'OPÉRATIONS' },
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
  const [toast, setToast] = useState('');
  const [sidebarOpen, setSidebarOpen] = useState(false);

  const activeCompany = useMemo(() => societes.find((company) => company.id === selectedSocieteId) ?? null, [societes, selectedSocieteId]);
  const activePage = navigation.find((item) => item.key === page) ?? navigation[0];

  const loadData = useCallback(async () => {
    setError('');
    const [companies, accountList, entryList, metrics] = await Promise.all([
      api<Societe[]>('list_societes'), api<Account[]>('list_comptes'), api<EntryView[]>('list_ecritures', { societeId: null }), api<DashboardSummary>('dashboard_summary'),
    ]);
    setSocietes(companies);
    setAccounts(accountList);
    setEntries(entryList);
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
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(''), 3200);
    return () => window.clearTimeout(timer);
  }, [toast]);

  async function handleDatabaseAccess(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError('');
    try {
      const command = status === 'needs_setup' ? 'initialize_database' : 'unlock_database';
      await api<void>(command, { passphrase });
      setPassphrase('');
      setStatus('ready');
      setToast(!isDesktopRuntime() ? 'Mode démo activé : données temporaires en mémoire uniquement.' : status === 'needs_setup' ? 'Coffre local chiffré créé.' : 'Coffre déverrouillé.');
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }

  async function handleLock() {
    try {
      await api<void>('lock_database');
      setStatus('locked');
      setPassphrase('');
      setSocietes([]); setEntries([]); setAccounts([]); setSummary(emptySummary);
      setPage('dashboard');
    } catch (reason) { setError(errorMessage(reason)); }
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
      setToast(isDesktopRuntime() ? 'Société supprimée du coffre local.' : 'Société supprimée du mode démo volatile.');
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
      setToast(isDesktopRuntime() ? (isEditing ? 'Société modifiée dans le coffre local.' : 'Société enregistrée dans le coffre local.') : (isEditing ? 'Société modifiée dans le mode démo volatile.' : 'Société ajoutée au mode démo volatile.'));
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
      setToast('Écriture équilibrée et validée. Elle est verrouillée : une correction devra passer par une contre-écriture.');
    } catch (reason) { setError(errorMessage(reason)); }
    finally { setEntryBusy(false); }
  }

  async function refreshEntries() {
    try {
      const list = await api<EntryView[]>('list_ecritures', { societeId: null });
      setEntries(list);
    } catch (reason) { setError(errorMessage(reason)); }
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
        {page === 'dashboard' && <Dashboard summary={summary} societes={societes} entries={entries} onNavigate={setPage} onNewCompany={() => { setPage('societes'); setEditingSocieteId(null); setCompanyForm(defaultCompanyForm); setShowCompanyForm(true); }} onNewEntry={() => { setPage('ecritures'); setShowEntryForm(true); }} />}
        {page === 'societes' && <CompaniesPage onNew={() => { setEditingSocieteId(null); setCompanyForm(defaultCompanyForm); setShowCompanyForm(true); }} editingSocieteId={editingSocieteId} onEdit={handleEditCompany} onDelete={handleDeleteCompany} societes={societes} showForm={showCompanyForm} setShowForm={setShowCompanyForm} form={companyForm} setForm={setCompanyForm} onSubmit={handleCreateCompany} />}
        {page === 'ecritures' && <EntriesPage entries={entries.filter((entry) => !selectedSocieteId || entry.societeId === selectedSocieteId)} societes={societes} accounts={accounts} activeCompany={activeCompany} setSelectedSocieteId={setSelectedSocieteId} showForm={showEntryForm} setShowForm={setShowEntryForm} onSubmit={handleCreateEntry} entryDate={entryDate} setEntryDate={setEntryDate} entryLabel={entryLabel} setEntryLabel={setEntryLabel} entryReference={entryReference} setEntryReference={setEntryReference} debitAccount={debitAccount} setDebitAccount={setDebitAccount} creditAccount={creditAccount} setCreditAccount={setCreditAccount} entryAmount={entryAmount} setEntryAmount={setEntryAmount} busy={entryBusy} onRefresh={refreshEntries} />}
        {page !== 'dashboard' && page !== 'societes' && page !== 'ecritures' && <PlannedPage page={page} onNewCompany={() => { setPage('societes'); setEditingSocieteId(null); setCompanyForm(defaultCompanyForm); setShowCompanyForm(true); }} />}
        <footer className="page-footer"><span>SuperPower Alpha 0.1</span><span><span className="footer-dot" /> Coffre local {isDesktopRuntime() ? 'chiffré' : 'démo volatile'}</span><span>Conçu pour garder le contrôle.</span></footer>
      </div>
    </main>
    {toast && <div className="toast"><CheckCircle2 size={18} /><span>{toast}</span><button aria-label="Fermer" onClick={() => setToast('')}><X size={15} /></button></div>}
  </div>;
}

function Dashboard({ summary, societes, entries, onNavigate, onNewCompany, onNewEntry }: {
  summary: DashboardSummary; societes: Societe[]; entries: EntryView[];
  onNavigate: (page: PageKey) => void; onNewCompany: () => void; onNewEntry: () => void;
}) {
  const now = new Intl.DateTimeFormat('fr-FR', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' }).format(new Date());
  const firstName = 'Bonjour';
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
          <button className={`step-item ${summary.ecrituresValidees ? 'complete' : societes.length ? 'current' : ''}`} onClick={onNewEntry}><div className="step-icon">{summary.ecrituresValidees ? <CheckCircle2 size={18} /> : <span>02</span>}</div><div className="step-copy"><strong>Créer la première écriture</strong><span>Le contrôle d'équilibre est automatique</span></div><ArrowRight size={16} /></button>
          <button className="step-item planned" onClick={() => onNavigate('documents')}><div className="step-icon"><span>03</span></div><div className="step-copy"><strong>Préparer l'arrivée des documents</strong><span>Le classement intelligent vient ensuite</span></div><span className="nav-soon">Sprint 04</span></button>
        </div>
      </div>
      <div className="panel activity-panel"><div className="panel-heading"><div><div className="eyebrow">VOTRE ACTIVITÉ</div><h3>Dernières écritures</h3></div><button className="text-link" onClick={() => onNavigate('ecritures')}>Tout voir <ArrowRight size={15} /></button></div>
        {entries.length ? <div className="activity-list">{entries.slice(0, 4).map((entry) => <div className="activity-row" key={entry.id}><div className="activity-glyph"><BookOpenCheck size={16} /></div><div className="activity-main"><strong>{entry.libelle}</strong><span>{formatDate(entry.dateEcriture)} · {entry.lines.length} lignes</span></div><div className="activity-amount"><strong>{formatMoney(entry.totalCents)}</strong><span><CheckCircle2 size={12} /> Validée</span></div></div>)}</div> : <div className="empty-activity"><div className="empty-art"><BookOpenCheck size={23} /></div><strong>Votre journal est encore calme.</strong><span>La première écriture équilibrée apparaîtra ici.</span><button className="button secondary small" onClick={onNewEntry}><Plus size={15} /> Ajouter une écriture</button></div>}
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

function PlannedPage({ page, onNewCompany }: { page: PageKey; onNewCompany: () => void }) {
  const meta: Record<Exclude<PageKey, 'dashboard' | 'societes' | 'ecritures'>, { eyebrow: string; title: string; description: string; sprint: string; icon: typeof Files; features: string[] }> = {
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
