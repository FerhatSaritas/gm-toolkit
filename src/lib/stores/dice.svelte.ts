import { rollDice } from "../commands";
import type { RollResult } from "../commands";

export interface DiceLogEntry extends RollResult {
  at: number;
}

export const dice = $state<{ history: DiceLogEntry[] }>({ history: [] });

const MAX_HISTORY = 40;

export function breakdown(r: RollResult): string {
  const parts: string[] = [];
  for (const g of r.groups) {
    const kept = g.kept ?? g.rolls;
    if (g.kept && g.kept.length !== g.rolls.length) {
      parts.push(`[${g.rolls.join(", ")}] keep ${kept.join(", ")}`);
    } else {
      parts.push(kept.join(", "));
    }
  }
  if (r.modifier !== 0) {
    parts.push(`${r.modifier >= 0 ? "+" : ""}${r.modifier}`);
  }
  return parts.join("  ");
}

export async function roll(expr: string): Promise<RollResult> {
  const cleaned = expr.trim().replace(/^\/roll\s+/i, "");
  const result = await rollDice(cleaned);
  dice.history.unshift({ ...result, at: Date.now() });
  if (dice.history.length > MAX_HISTORY) dice.history.length = MAX_HISTORY;
  return result;
}
