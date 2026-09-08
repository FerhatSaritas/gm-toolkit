import {
  resetEncounter as resetCmd,
  saveEncounterState,
  startEncounter as startCmd
} from "../commands";
import type { CombatantState, EncounterState } from "../commands";
import { notify } from "./campaign.svelte";

export const encounter = $state<{
  state: EncounterState | null;
  selectedId: string | null;
}>({
  state: null,
  selectedId: null
});

export const CONDITIONS = [
  "Blinded",
  "Charmed",
  "Concentration",
  "Deafened",
  "Frightened",
  "Grappled",
  "Incapacitated",
  "Invisible",
  "Paralyzed",
  "Petrified",
  "Poisoned",
  "Prone",
  "Restrained",
  "Stunned",
  "Unconscious"
];

function d20(): number {
  return 1 + Math.floor(Math.random() * 20);
}

export function sortCombatants(list: CombatantState[]): CombatantState[] {
  return [...list].sort(
    (a, b) =>
      (b.initiative ?? Number.NEGATIVE_INFINITY) -
        (a.initiative ?? Number.NEGATIVE_INFINITY) ||
      (b.initiativeMod ?? 0) - (a.initiativeMod ?? 0) ||
      a.name.localeCompare(b.name)
  );
}

function uniqueId(name: string, existing: EncounterState): string {
  const base =
    name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "combatant";
  let id = base;
  let n = 2;
  while (existing.combatants.some((c) => c.id === id)) {
    id = `${base}-${n}`;
    n++;
  }
  return id;
}

function persist(next: EncounterState) {
  encounter.state = next;
  saveEncounterState(next).catch((e) => notify(String(e), "error"));
}

export async function startEncounter(relPath: string) {
  try {
    const res = await startCmd(relPath);
    encounter.state = res.state;
    encounter.selectedId = null;
    notify(res.restored ? `Resumed encounter — round ${res.state.round}` : "Encounter started");
  } catch (e) {
    notify(String(e), "error");
  }
}

export async function resetEncounter() {
  const s = encounter.state;
  if (!s) return;
  try {
    encounter.state = await resetCmd(s.encounterPath);
    encounter.selectedId = null;
    notify("Encounter reset");
  } catch (e) {
    notify(String(e), "error");
  }
}

export function rollInitiative() {
  const s = encounter.state;
  if (!s) return;
  const combatants = sortCombatants(
    s.combatants.map((c) => ({
      ...c,
      initiative: d20() + (c.initiativeMod ?? 0)
    }))
  );
  persist({ ...s, combatants, turnId: s.turnId });
  notify("Initiative rolled");
}

export function setInitiative(id: string, value: number | null) {
  const s = encounter.state;
  if (!s) return;
  const combatants = sortCombatants(
    s.combatants.map((c) => (c.id === id ? { ...c, initiative: value ?? undefined } : c))
  );
  persist({ ...s, combatants });
}

function applyDelta(c: CombatantState, delta: number): CombatantState {
  if (delta < 0) {
    let dmg = -delta;
    const tempHp = Math.max(0, c.tempHp - dmg);
    dmg -= c.tempHp - tempHp;
    return { ...c, tempHp, hp: Math.max(0, c.hp - dmg) };
  }
  const cap = c.maxHp ?? Number.POSITIVE_INFINITY;
  return { ...c, hp: Math.min(cap, c.hp + delta) };
}

export function adjustHp(id: string, delta: number) {
  const s = encounter.state;
  if (!s || delta === 0 || Number.isNaN(delta)) return;
  persist({
    ...s,
    combatants: s.combatants.map((c) => (c.id === id ? applyDelta(c, delta) : c))
  });
}

export function setHp(id: string, value: number) {
  const s = encounter.state;
  if (!s || Number.isNaN(value)) return;
  persist({
    ...s,
    combatants: s.combatants.map((c) => {
      if (c.id !== id) return c;
      const cap = c.maxHp ?? Number.POSITIVE_INFINITY;
      return { ...c, hp: Math.max(0, Math.min(cap, Math.round(value))) };
    })
  });
}

export function setTempHp(id: string, value: number) {
  const s = encounter.state;
  if (!s || Number.isNaN(value)) return;
  persist({
    ...s,
    combatants: s.combatants.map((c) =>
      c.id === id ? { ...c, tempHp: Math.max(0, Math.round(value)) } : c
    )
  });
}

export function toggleCondition(id: string, condition: string) {
  const s = encounter.state;
  if (!s) return;
  persist({
    ...s,
    combatants: s.combatants.map((c) =>
      c.id === id
        ? {
            ...c,
            conditions: c.conditions.includes(condition)
              ? c.conditions.filter((x) => x !== condition)
              : [...c.conditions, condition]
          }
        : c
    )
  });
}

export function addCombatant(
  name: string,
  kind: CombatantState["kind"],
  hp: number | null,
  ac: number | null,
  statblock?: string
) {
  const s = encounter.state;
  if (!s || !name.trim()) return;
  const maxHp = hp ?? 0;
  const combatant: CombatantState = {
    id: uniqueId(name.trim(), s),
    name: name.trim(),
    kind,
    initiative: undefined,
    initiativeMod: undefined,
    hp: maxHp,
    maxHp: maxHp > 0 ? maxHp : undefined,
    tempHp: 0,
    ac: ac ?? undefined,
    statblock: statblock ?? undefined,
    conditions: []
  };
  persist({ ...s, combatants: [...s.combatants, combatant] });
}

export function removeCombatant(id: string) {
  const s = encounter.state;
  if (!s) return;
  persist({
    ...s,
    combatants: s.combatants.filter((c) => c.id !== id),
    turnId: s.turnId === id ? null : s.turnId
  });
}

export function advanceTurn() {
  const s = encounter.state;
  if (!s || s.combatants.length === 0) return;
  const ids = sortCombatants(s.combatants).map((c) => c.id);
  if (!s.turnId || !ids.includes(s.turnId)) {
    persist({ ...s, turnId: ids[0], round: s.round || 1 });
    return;
  }
  const idx = ids.indexOf(s.turnId);
  if (idx + 1 < ids.length) {
    persist({ ...s, turnId: ids[idx + 1] });
  } else {
    persist({ ...s, turnId: ids[0], round: s.round + 1 });
  }
}

export function rewindTurn() {
  const s = encounter.state;
  if (!s || s.combatants.length === 0) return;
  const ids = sortCombatants(s.combatants).map((c) => c.id);
  if (!s.turnId || !ids.includes(s.turnId)) {
    persist({ ...s, turnId: ids[ids.length - 1] });
    return;
  }
  const idx = ids.indexOf(s.turnId);
  if (idx > 0) {
    persist({ ...s, turnId: ids[idx - 1] });
  } else {
    persist({ ...s, turnId: ids[ids.length - 1], round: Math.max(1, s.round - 1) });
  }
}

export function setTurn(id: string) {
  const s = encounter.state;
  if (!s) return;
  persist({ ...s, turnId: id });
}

export function moveSelection(delta: 1 | -1) {
  const s = encounter.state;
  if (!s || s.combatants.length === 0) return;
  const ids = sortCombatants(s.combatants).map((c) => c.id);
  const current = encounter.selectedId && ids.includes(encounter.selectedId)
    ? ids.indexOf(encounter.selectedId)
    : -1;
  const next = Math.max(0, Math.min(ids.length - 1, current + delta));
  encounter.selectedId = ids[next];
}

export function clearEncounterView() {
  encounter.state = null;
  encounter.selectedId = null;
}
