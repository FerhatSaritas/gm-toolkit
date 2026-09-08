import { invoke } from "@tauri-apps/api/core";

export interface CampaignCounts {
  scenes: number;
  encounters: number;
  npcs: number;
  statblocks: number;
  items: number;
  total: number;
}

export interface CampaignInfo {
  name: string;
  path: string;
  counts: CampaignCounts;
}

export interface TreeNode {
  name: string;
  relPath: string;
  kind: "dir" | "file";
  ext: string | null;
  open: boolean;
  children: TreeNode[];
}

export interface FileDoc {
  path: string;
  name: string;
  content: string;
}

export type DocKind = "scene" | "encounter" | "statblock" | "item";

export interface SceneMeta {
  title: string;
  tags: string[];
  npcs: string[];
  readAloud?: boolean;
  status?: string;
  [key: string]: unknown;
}

export interface Combatant {
  name: string;
  kind: "monster" | "npc" | "player";
  hp?: number;
  ac?: number;
  initiative?: number;
  statblock?: string;
  tags: string[];
  [key: string]: unknown;
}

export interface EncounterMeta {
  title: string;
  partyLevel?: number;
  combatants: Combatant[];
  tags: string[];
  [key: string]: unknown;
}

export interface Abilities {
  strength?: number;
  dexterity?: number;
  constitution?: number;
  intelligence?: number;
  wisdom?: number;
  charisma?: number;
  [key: string]: unknown;
}

export interface NamedText {
  name: string;
  text: string;
}

export interface StatblockMeta {
  name: string;
  kind: "monster" | "npc" | "player";
  hp?: number;
  ac?: number;
  initiative?: number;
  speed?: string;
  power?: number;
  abilities?: Abilities;
  traits: NamedText[];
  actions: NamedText[];
  reactions: NamedText[];
  tags: string[];
  [key: string]: unknown;
}

export interface ItemMeta {
  name: string;
  rarity?: string;
  kind?: string;
  value?: string;
  tags: string[];
  [key: string]: unknown;
}

export interface Doc {
  relPath: string;
  kind: DocKind;
  body: string;
  /** Rendered HTML of `body` (client-side cache, not persisted). */
  bodyHtml?: string;
  scene?: SceneMeta;
  encounter?: EncounterMeta;
  statblock?: StatblockMeta;
  item?: ItemMeta;
}

export const getConfig = () => invoke<CampaignInfo | null>("get_config");
export const chooseCampaign = () =>
  invoke<CampaignInfo | null>("choose_campaign");
export const readCampaignTree = () => invoke<TreeNode[]>("read_campaign_tree");
export const readFile = (relPath: string) =>
  invoke<FileDoc>("read_file", { relPath });
export const loadDoc = (relPath: string) => invoke<Doc>("load_doc", { relPath });
export const renderMarkdown = (md: string) =>
  invoke<string>("render_markdown", { md });

export interface CombatantState {
  id: string;
  name: string;
  kind: "monster" | "npc" | "player";
  initiative?: number;
  initiativeMod?: number;
  hp: number;
  maxHp?: number;
  tempHp: number;
  ac?: number;
  statblock?: string;
  conditions: string[];
}

export interface EncounterState {
  encounterPath: string;
  round: number;
  turnId: string | null;
  combatants: CombatantState[];
}

export interface EncounterLoad {
  state: EncounterState;
  restored: boolean;
}

export const startEncounter = (relPath: string) =>
  invoke<EncounterLoad>("start_encounter", { relPath });
export const saveEncounterState = (encounterState: EncounterState) =>
  invoke<void>("save_encounter_state", { encounterState });
export const resetEncounter = (relPath: string) =>
  invoke<EncounterState>("reset_encounter", { relPath });

export interface DiceGroup {
  rolls: number[];
  kept?: number[];
  sides: number;
}

export interface RollResult {
  notation: string;
  total: number;
  modifier: number;
  groups: DiceGroup[];
}

export const rollDice = (expr: string) => invoke<RollResult>("roll_dice", { expr });
export const openStatblock = (basePath: string | null, link: string) =>
  invoke<Doc>("open_statblock", { basePath, link });

export type CreatureKind = "monster" | "npc" | "player";

export interface StatblockSummary {
  relPath: string;
  name: string;
  kind: CreatureKind;
  hp?: number;
  ac?: number;
  initiative?: number;
  speed?: string;
  power?: number;
  tags: string[];
  traits: number;
  actions: number;
  reactions: number;
}

export interface EncounterSummary {
  relPath: string;
  title: string;
  combatants: number;
  tags: string[];
}

export interface CombatantInput {
  name: string;
  kind: CreatureKind;
  hp?: number;
  ac?: number;
  initiative?: number;
  statblock?: string;
  tags?: string[];
}

export const listStatblocks = () => invoke<StatblockSummary[]>("list_statblocks");
export const listEncounters = () => invoke<EncounterSummary[]>("list_encounters");
export const addToEncounter = (encounterPath: string, combatant: CombatantInput) =>
  invoke<EncounterSummary>("add_combatant_to_encounter", { encounterPath, combatant });
export const createEncounter = (title: string, combatant?: CombatantInput) =>
  invoke<string>("create_encounter", { title, combatant: combatant ?? null });
export const createStatblock = (
  name: string,
  kind: CreatureKind,
  hp?: number,
  ac?: number,
  power?: number
) => invoke<string>("create_statblock", { name, kind, hp: hp ?? null, ac: ac ?? null, power: power ?? null });
export const duplicateFile = (relPath: string) =>
  invoke<string>("duplicate_file", { relPath });
export const deleteFile = (relPath: string) => invoke<void>("delete_file", { relPath });

export interface ImportPreview {
  sourcePath: string;
  format: "md" | "json";
  statblock: StatblockMeta;
  body: string;
  warnings: string[];
  suggestedPath: string;
}

export const pickImportFile = () => invoke<string | null>("pick_import_file");
export const requestImportPreview = (path: string) =>
  invoke<ImportPreview>("import_preview", { path });
export const commitImport = (statblock: StatblockMeta, body: string) =>
  invoke<string>("commit_import", { statblock, body });

export interface SearchHit {
  relPath: string;
  kind: DocKind;
  title: string;
  snippet: string;
  score: number;
}

export const searchCampaign = (query: string) =>
  invoke<SearchHit[]>("search_campaign", { query });

export type PackKindT = "npc" | "item" | "encounter";

export interface PackInfo {
  id: string;
  source: "builtin" | "user";
  name: string;
  kind: PackKindT | null;
  templates: string[];
  tables: string[];
  error: string | null;
}

export interface Generated {
  packId: string;
  packName: string;
  kind: PackKindT | null;
  template: string;
  text: string;
  fields: Record<string, string>;
}

export const listGeneratorPacks = () => invoke<PackInfo[]>("list_generator_packs");
export const generateFromPack = (packId: string, template?: string) =>
  invoke<Generated>("generate_from_pack", { packId, template: template ?? null });
export const generateEncounterDraft = (partyLevel: number, band: number, count: number) =>
  invoke<StatblockSummary[]>("generate_encounter_draft", { partyLevel, band, count });
export const saveGeneratedStatblock = (statblock: StatblockMeta, body: string) =>
  invoke<string>("save_generated_statblock", { statblock, body });
export const saveGeneratedItem = (item: ItemMeta, body: string) =>
  invoke<string>("save_generated_item", { item, body });
export const saveGeneratedEncounter = (
  title: string,
  partyLevel: number | null,
  combatants: CombatantInput[],
  body: string
) => invoke<string>("save_generated_encounter", { title, partyLevel, combatants, body });
