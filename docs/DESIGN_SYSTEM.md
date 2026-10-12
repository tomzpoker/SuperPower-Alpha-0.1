# Design System — SuperPower Alpha

## 1. Philosophie visuelle

L'interface doit procurer une **sensation de calme, de contrôle et de plaisir**. L'utilisateur doit avoir envie d'ouvrir l'application, comme on ouvre une belle app de méditation ou un jeu bien designé.

**Mots-clés** : futuriste, pastel, apaisant, fluide, minimal, lumineux, organique.

**Anti-mots-clés** : corporate, froid, dense, gris, austère, bordélique.

## 2. Les 5 Thèmes

Chaque thème définit une palette complète. L'utilisateur peut basculer instantanément, et seul ce choix visuel est conservé localement.

### 2.1 Aurora (thème par défaut)

Ambiance : aurore boréale, nuit douce, reflets violets et cyans.

| Rôle | Couleur | Hex |
|---|---|---|
| Background | Deep night | `#0F0E1A` |
| Surface | Night soft | `#1A1830` |
| Surface elevated | Night light | `#252347` |
| Primary | Aurora violet | `#A78BFA` |
| Primary hover | Aurora violet light | `#C4B5FD` |
| Secondary | Aurora cyan | `#67E8F9` |
| Accent | Aurora pink | `#F0ABFC` |
| Text primary | Soft white | `#F5F3FF` |
| Text secondary | Lavender gray | `#B8B4D4` |
| Success | Mint | `#6EE7B7` |
| Warning | Peach | `#FDBA74` |
| Error | Coral | `#FCA5A5` |

### 2.2 Sakura

Ambiance : fleurs de cerisier, douceur, féminin, apaisant.

| Rôle | Couleur | Hex |
|---|---|---|
| Background | Cream | `#FFF8F3` |
| Surface | White pink | `#FFEFEA` |
| Primary | Sakura pink | `#F9A8D4` |
| Secondary | Blush | `#FBCFE8` |
| Accent | Petal | `#FDE68A` |
| Text primary | Deep plum | `#4C1D24` |
| Text secondary | Rose gray | `#9F7A82` |

### 2.3 Mint

Ambiance : nature, fraîcheur, zen, concentration.

| Rôle | Couleur | Hex |
|---|---|---|
| Background | Off-white mint | `#F0FDF9` |
| Surface | Mint light | `#DCFCE7` |
| Primary | Emerald | `#34D399` |
| Secondary | Teal | `#5EEAD4` |
| Accent | Lime | `#BEF264` |
| Text primary | Forest | `#064E3B` |
| Text secondary | Sage | `#6B9080` |

### 2.4 Sunset

Ambiance : coucher de soleil, chaleur, énergie douce.

| Rôle | Couleur | Hex |
|---|---|---|
| Background | Warm cream | `#FFF7ED` |
| Surface | Peach light | `#FFEDD5` |
| Primary | Coral | `#FB923C` |
| Secondary | Rose | `#F472B6` |
| Accent | Amber | `#FCD34D` |
| Text primary | Burgundy | `#7C2D12` |
| Text secondary | Terracotta | `#C2856B` |

### 2.5 Midnight

Ambiance : nuit étoilée, focus, professionnel, sombre élégant.

| Rôle | Couleur | Hex |
|---|---|---|
| Background | Deep navy | `#0B1120` |
| Surface | Navy soft | `#1E293B` |
| Primary | Electric blue | `#60A5FA` |
| Secondary | Indigo | `#818CF8` |
| Accent | Cyan | `#22D3EE` |
| Text primary | Ice white | `#F1F5F9` |
| Text secondary | Slate | `#94A3B8` |

## 3. Typographie

- **Titres et corps** : pile système (Inter si disponible, puis Segoe UI / sans-serif) ; pas de police téléchargée depuis un serveur externe.
- **Chiffres** : police monospace système pour aligner les montants, dates et codes.

### Échelle

| Rôle | Taille | Poids | Line-height |
|---|---|---|---|
| Display | 48px | 700 | 1.1 |
| H1 | 32px | 700 | 1.2 |
| H2 | 24px | 600 | 1.3 |
| H3 | 20px | 600 | 1.4 |
| Body L | 16px | 400 | 1.6 |
| Body | 14px | 400 | 1.6 |
| Small | 12px | 400 | 1.5 |
| Mono | 13px | 500 | 1.4 |

## 4. Espacement & Grille

- **Base** : 4px.
- **Échelle** : 4, 8, 12, 16, 24, 32, 48, 64, 96.
- **Grille** : 12 colonnes, gap 24px, max-width 1440px.
- **Rayons** : 8px (inputs), 12px (cards), 16px (modals), 999px (pills).

## 5. Ombres & Profondeur

Utiliser des ombres douces et colorées (pas de noir pur) :

```css
--shadow-sm: 0 1px 2px rgba(0,0,0,0.05);
--shadow-md: 0 4px 12px rgba(0,0,0,0.08);
--shadow-lg: 0 12px 32px rgba(0,0,0,0.12);
--shadow-glow: 0 0 24px var(--primary-40);
```

## 6. Composants clés

### 6.1 Layout principal

```text

┌──────────────────────────────────────────────────┐
│  Sidebar (icônes + labels, collapsible)          │
│  ┌────┐                                          │
│  │ 🏠 │  Dashboard                               │
│  │ 📥 │  Inbox                                   │
│  │ 📄 │  Documents                               │
│  │ 💰 │  Trésorerie                              │
│  │ 🏢 │  Locatif                                 │
│  │ 📊 │  Fiscalité                               │
│  │ ⚙️ │  Paramètres                              │
│  └────┘                                          │
│                                                  │
│                  Contenu principal               │
│                                                  │
└──────────────────────────────────────────────────┘
```

### 6.2 Composants signature

    GlassCard : carte avec effet verre dépoli (backdrop-blur, bordure subtile, ombre colorée).

    PillStatus : badge arrondi avec couleur sémantique.

    MetricCard : grande carte avec chiffre principal, variation, sparkline.

    InboxItem : ligne de validation rapide avec actions inline.

    TimelineFlow : visualisation des flux de trésorerie.

    CommandPalette : palette de commandes (Cmd+K) pour tout faire au clavier.

    ThemeSwitcher : sélecteur de thème avec aperçu live.

6.3 Micro-interactions

    Hover : légère élévation + glow subtil.

    Click : scale 0.98 en 100ms.

    Transitions : 200ms ease-out par défaut.

    Loading : skeletons pulsants plutôt que spinners.

    Success : animation de check qui se dessine + confetti discret (optionnel).

    Notifications : toasts en haut à droite, auto-dismiss 4s.

7. Iconographie

    Bibliothèque : Lucide (cohérent, léger, 1000+ icônes).

    Style : stroke 1.5px, coins arrondis.

    Couleur : hérite du texte, sauf icônes sémantiques (success/warning/error).

8. Navigation
8.1 Desktop

    Sidebar collapsible (240px → 72px).

    Header avec breadcrumbs + recherche + notifications + profil.

    Cmd+K pour la command palette.

    Raccourcis : g d (dashboard), g i (inbox), g t (trésorerie)...

8.2 Mobile

    Bottom tabs : 5 onglets max (Dashboard, Inbox, Actions, Docs, Profil).

    Header minimal avec titre + actions contextuelles.

    Gestes : swipe pour archiver, pull-to-refresh.

    Actions rapides : bouton flottant contextuel.

9. Mode simplifié vs mode expert

L'utilisateur peut basculer entre :

    Mode simplifié (par défaut) : uniquement les actions essentielles, langage clair, pas de jargon comptable.

    Mode expert : accès aux journaux, balances, grand livre, paramètres avancés.

Le mode est un toggle dans le header, persisté par utilisateur.
10. Logo

Concept : un nœud ou un cristal stylisé, évoquant :

    La solidité (entreprise).

    La modularité (facettes).

    La légèreté (formes épurées).

Proposition : un hexagone avec une spirale interne en dégradé pastel, ou un nœud de trèfle abstrait.

Format : SVG, adaptable, décliné en icône (16px), favicon (32px), logo complet (512px).
## 11. Accessibilité

    Contraste : AA minimum (4.5:1 pour le texte).

    Focus : toujours visible, anneau coloré.

    Clavier : navigation complète sans souris.

    Lecteurs d'écran : ARIA labels, rôles sémantiques.

    Réduction de mouvement : respect de prefers-reduced-motion.

## 12. Ressources

    Figma (à créer) : maquettes des écrans clés.

    Storybook : catalogue des composants.

    Tokens : fichier tokens.json avec toutes les variables.