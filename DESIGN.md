---
name: Obsidian Grimoire
colors:
  surface: '#111319'
  surface-dim: '#111319'
  surface-bright: '#373940'
  surface-container-lowest: '#0c0e14'
  surface-container-low: '#191b22'
  surface-container: '#1d1f26'
  surface-container-high: '#282a30'
  surface-container-highest: '#33343b'
  on-surface: '#e2e2ea'
  on-surface-variant: '#d4c4b2'
  inverse-surface: '#e2e2ea'
  inverse-on-surface: '#2e3037'
  outline: '#9c8f7e'
  outline-variant: '#504537'
  surface-tint: '#f6bc64'
  primary: '#fec46b'
  on-primary: '#442b00'
  primary-container: '#e0a953'
  on-primary-container: '#5f3e00'
  inverse-primary: '#805600'
  secondary: '#ffb3b6'
  on-secondary: '#68001a'
  secondary-container: '#cc003c'
  on-secondary-container: '#ffdcdc'
  tertiary: '#58e7aa'
  on-tertiary: '#003824'
  tertiary-container: '#33ca90'
  on-tertiary-container: '#005035'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#ffddb0'
  primary-fixed-dim: '#f6bc64'
  on-primary-fixed: '#291800'
  on-primary-fixed-variant: '#614000'
  secondary-fixed: '#ffdada'
  secondary-fixed-dim: '#ffb3b6'
  on-secondary-fixed: '#40000c'
  on-secondary-fixed-variant: '#920028'
  tertiary-fixed: '#6ffbbe'
  tertiary-fixed-dim: '#4edea3'
  on-tertiary-fixed: '#002113'
  on-tertiary-fixed-variant: '#005236'
  background: '#111319'
  on-background: '#e2e2ea'
  surface-variant: '#33343b'
typography:
  display-lg:
    fontFamily: Cinzel
    fontSize: 32px
    fontWeight: '700'
    lineHeight: 40px
    letterSpacing: 0.05em
  display-sm:
    fontFamily: Cinzel
    fontSize: 24px
    fontWeight: '600'
    lineHeight: 32px
    letterSpacing: 0.04em
  headline-lg:
    fontFamily: Cinzel
    fontSize: 20px
    fontWeight: '600'
    lineHeight: 28px
    letterSpacing: 0.03em
  headline-sm:
    fontFamily: Cinzel
    fontSize: 16px
    fontWeight: '600'
    lineHeight: 24px
    letterSpacing: 0.02em
  body-lg:
    fontFamily: Inter
    fontSize: 15px
    fontWeight: '400'
    lineHeight: 24px
  body-md:
    fontFamily: Inter
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 20px
  body-sm:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 18px
  label-lg:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '600'
    lineHeight: 16px
    letterSpacing: 0.02em
  label-mono-lg:
    fontFamily: JetBrains Mono
    fontSize: 14px
    fontWeight: '500'
    lineHeight: 20px
    letterSpacing: -0.01em
  label-mono-sm:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: '500'
    lineHeight: 14px
    letterSpacing: 0.02em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  window-drag: 2.25rem
  sidebar-width: 18rem
  statblock-width: 22rem
  gutter-xs: 0.25rem
  gutter-sm: 0.5rem
  gutter-md: 0.75rem
  gutter-lg: 1rem
  gutter-xl: 1.5rem
  inset-panel: 0.875rem
---

## Brand & Style

This design system crafts an immersive, high-utility workspace tailored for tabletop roleplaying Game Masters orchestrating complex campaigns in real time. The tone balances arcane gravitas with computational precision—evoking an ancient leather-bound grimoire re-engineered into an elite mission-control interface.

The design movement merges **Modern Desktop Skeuomorphism / Tactile Minimalism** with **Atmospheric Dark Fantasy**:
- **Tactility:** Razor-sharp panel dividers, micro-beveled borders, subtle linear ambient glow on active states, and custom native-like title bars.
- **Utility & Focus:** Zero visual bloat. High information density without clutter, letting campaign notes, monster statblocks, and dynamic turn trackers co-exist effortlessly under high cognitive load.
- **Atmospheric Immersion:** Deep obsidian and cold slate foundations paired with warm parchment amber and arterial crimson, evoking candlelight casting shadows across stone archives.

## Colors

The color architecture is built around deep spatial layers, high-contrast utility accents, and thematic RPG semantic coding.

### Palette Roles
- **Base Obsidian & Slate Surfaces:**
  - `surface-canvas` (`#0d0f12`): Root Tauri window background and custom window frame.
  - `surface-base` (`#12141a`): Primary background for inactive splits and markdown canvas.
  - `surface-raised` (`#1a1d24`): Main cards, tracker panels, and sidebar segments.
  - `surface-overlay` (`#242933`): Floating inspectors, dropdown menus, context tooltips, and modal sheets.
  - `surface-highlight` (`#2e3440`): Hover states, active table rows, and selected segments.

- **Warm Parchment Amber (`#e0a953` / `#f59e0b`):** The primary brand and interactive accent. Represents experience points, spell slots, active turn indicators, key entity links, and primary CTA buttons.
- **Arterial Crimson (`#e11d48` / `#ef4444`):** Secondary accent for combat initiative trackers, damage counters, lethal hazards, bloodied states, and critical alerts.
- **Mana Emerald (`#10b981`):** Tertiary status accent representing hit point pools, healing surges, success rolls, and stabilized states.
- **Arcane Sapphire (`#38bdf8`):** Utility accent used for passive senses, spellcasting modifiers, and markdown links.

### Contrast & State Hierarchy
- **Borders & Dividers:** Built using subtle translucent overlays (`rgba(255, 255, 255, 0.07)` for static containers, `rgba(224, 169, 83, 0.35)` for focused containers).
- **Text Tiers:**
  - `text-primary`: `#f1f5f9` (High-contrast slate white for rules, names, and rolls).
  - `text-secondary`: `#94a3b8` (Muted silver for secondary metadata, descriptions, and passive modifiers).
  - `text-tertiary`: `#64748b` (Low-contrast gray for inactive dice notations, hotkeys, and table headers).

## Typography

The type stack combines authoritative classical display typography, ultra-legible administrative body text, and an uncompromising monospace font for numeric calculations:

- **Headlines (`Cinzel`):** Classical Roman proportions give monsters, campaign chapters, locations, and panel titles an editorial fantasy presence. Used selectively for headings, badges, and major statblock titles. Always set with subtle letter-spacing.
- **User Interface & Markdown Prose (`Inter`):** Powers all note editing, rules references, narrative descriptions, and UI controls. Inter provides neutral clarity across dense tables and multi-column layouts without inducing eye strain during four-hour sessions.
- **Dice, Math, & Stats (`JetBrains Mono`):** Dedicated to dice rolls (`1d20+7`), armor class values, initiative counts, hit points, spell slots, and hotkey accelerators. Tabular figures ensure numeric columns never jitter during rapid combat round updates.

## Layout & Spacing

Designed for a multi-pane desktop canvas inside a Tauri wrapper, utilizing an adaptive multi-column split layout:

- **Window Chrome:** Fixed 36px (`2.25rem`) top region acting as the Tauri draggable title bar, holding session status, scene switcher, quick dice roller, and window control jewels.
- **Docked Multi-Split Workspace:**
  - **Left Rail (Campaign Tree / Index):** Collapsible panel (`sidebar-width`: 288px) with dense hierarchical tree navigation.
  - **Center Stage (Markdown Workspace & Encounter Grid):** Fluid flex-grow container with optional 2-up vertical split view.
  - **Right Rail (Monster Statblocks & Initiative Drawer):** Context-aware inspector (`statblock-width`: 352px) that slides or pins into the active frame.
- **Rhythm & Insets:** Built on a strict 4px base increment. Data density is prioritized: 8px gaps between entity lists, 12px panel internal padding, and 4px gutter between micro stat pills.

## Elevation & Depth

Visual hierarchy uses a refined combination of **tonal surface stacking** and **micro-beveled illumination**:

- **Tonal Layers:** Layer 0 (`#0d0f12`) anchors the shell. Layer 1 (`#12141a`) hosts editing areas. Layer 2 (`#1a1d24`) lifts statblock cards and trackers. Layer 3 (`#242933`) isolates floating popovers, dice breakdown tooltips, and contextual command palettes.
- **Micro-Bevels & Ghost Insets:** Elements do not use muddy drop shadows. Instead, depth is articulated through 1px border lines:
  - Top and left borders: `rgba(255, 255, 255, 0.08)` (simulating a soft top-down ambient key light).
  - Bottom and right borders: `rgba(0, 0, 0, 0.5)` (recessive grounding shadow).
- **Arcane Ambient Sheen:** High-priority cards (such as the creature whose turn is active in the combat tracker) utilize a tinted amber edge stroke combined with a faint diffuse glow: `0 0 16px -4px rgba(224, 169, 83, 0.2)`.

## Shapes

The design system maintains a **Soft (`1`)** shape geometry.

- Standard UI containers, buttons, inputs, and statblock blocks use `0.25rem` (4px) radii.
- Secondary sheets, floating quick-reference cards, and modal dialogs step up to `0.5rem` (8px).
- Small functional tags (condition badges, spell schools, damage types) utilize `0.25rem` (4px) or angled 45-degree chamfered clips to preserve a precision instrument feel.
- Pure circular forms are reserved strictly for character portrait tokens, token status indicators, and dice condition counters.

## Components

### Window Title Bar
- Integrated Tauri custom frame in `#0d0f12`.
- Left-aligned campaign breadcrumb (`Campaign / Act II / Sunken Temple`).
- Centered quick action hub with instant roll input (`/roll 2d6+4`) rendered in `JetBrains Mono`.
- Right-aligned window actions with low-contrast slate controls.

### Buttons & Interactive Triggers
- **Primary (Amber Arcane):** Solid `#e0a953` surface with `#0d0f12` text in `Inter` weight 600. Active states drop scale to 0.98 with subtle inset shadow.
- **Secondary (Obsidian Slab):** Background `#1a1d24`, border `1px solid rgba(255, 255, 255, 0.1)`, hover shifts to `#242933` with primary amber border glow.
- **Destructive / Combat Action:** `#e11d48` background with crisp white text, used for round resets, kill triggers, and deletion.

### Initiative & Turn Tracker
- Row-based table with instant keyboard navigation (Space to advance turn, J/K for selection).
- Active creature row gains an amber vertical border beacon (3px solid `#e0a953`) and glowing ambient row highlight.
- Inline numeric fields for HP, Temp HP, and AC render in `JetBrains Mono` with immediate click-to-edit and math evaluation (`+12`, `-8`).
- Drag handles on the left for swift initiative re-ordering.

### Monster Statblock Container
- Modeled after traditional reference tomes reimagined for modern screens:
  - Header: Large Cinzel title with amber-to-crimson dual-stop rule divider.
  - Core Attributes Grid: 6-column tabular layout (STR, DEX, CON, INT, WIS, CHA) framed in `#12141a` wells with centered large bonus values (`+4`) and raw scores beneath (`18`).
  - Action Blocks: Indented markdown paragraphs with bold amber action titles and one-click dice roll tags (`[Attack: +7 | Hit: 2d8+4 piercing]`) that execute directly into the combat dice log.

### Tags, Chips, & Condition Badges
- Compact badges in `JetBrains Mono` 11px.
- **Conditions (Stunned, Blinded, Prone):** `#242933` background with crimson or amber tint and micro dot indicator.
- **Hit Point Pips:** Emerald `#10b981` pill chip tracking current/max with micro progress track underlay.

### Input Fields & Markdown Editor
- Surface: `#12141a` inset with 1px border `rgba(255, 255, 255, 0.08)`.
- Focused state: Border transitions to `#e0a953` with zero outline offset.
- Editor supports split-pane preview and live markdown rendering with custom fantasy horizontal rules (`---` produces a tapered diamond divider).