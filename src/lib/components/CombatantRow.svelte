<script lang="ts">
  import type { CombatantState } from "../commands";
  import { CONDITIONS } from "../stores/encounter.svelte";

  interface Props {
    combatant: CombatantState;
    active?: boolean;
    selected?: boolean;
    onAdjust: (id: string, delta: number) => void;
    onSetHp: (id: string, value: number) => void;
    onSetTempHp: (id: string, value: number) => void;
    onSetInitiative: (id: string, value: number | null) => void;
    onToggleCondition: (id: string, condition: string) => void;
    onSelect: (id: string) => void;
    onSetTurn: (id: string) => void;
    onRemove: (id: string) => void;
    onOpenStatblock: (id: string) => void;
  }

  let {
    combatant: c,
    active = false,
    selected = false,
    onAdjust,
    onSetHp,
    onSetTempHp,
    onSetInitiative,
    onToggleCondition,
    onSelect,
    onSetTurn,
    onRemove,
    onOpenStatblock
  }: Props = $props();

  let hpInput = $state("");
  let tempInput = $state("");
  let initInput = $state("");
  let hpFocused = $state(false);

  // Keep inputs in sync with store updates from any source.
  $effect(() => {
    if (!hpFocused) hpInput = String(c.hp);
    tempInput = c.tempHp > 0 ? String(c.tempHp) : "";
    initInput = c.initiative != null ? String(c.initiative) : "";
  });

  function parseHp() {
    const raw = hpInput.trim();
    hpFocused = false;
    if (!raw) {
      hpInput = String(c.hp);
      return;
    }
    const n = Number(raw.replace(/^\+/, ""));
    if (Number.isNaN(n)) {
      hpInput = String(c.hp);
      return;
    }
    if (raw.startsWith("+")) onSetHp(c.id, c.hp + n);
    else if (raw.startsWith("-")) onAdjust(c.id, n);
    else onSetHp(c.id, n);
  }

  function parseTemp() {
    const raw = tempInput.trim();
    if (!raw) {
      if (c.tempHp > 0) onSetTempHp(c.id, 0);
      return;
    }
    const n = Number(raw);
    if (!Number.isNaN(n)) onSetTempHp(c.id, n);
    else tempInput = c.tempHp > 0 ? String(c.tempHp) : "";
  }

  function parseInitiative() {
    const raw = initInput.trim();
    if (!raw) {
      onSetInitiative(c.id, null);
      return;
    }
    const n = Number(raw);
    if (!Number.isNaN(n)) onSetInitiative(c.id, Math.round(n));
    else initInput = c.initiative != null ? String(c.initiative) : "";
  }

  const hpRatio = $derived(
    c.maxHp && c.maxHp > 0 ? Math.min(100, (c.hp / c.maxHp) * 100) : c.hp > 0 ? 100 : 0
  );
  const hpTone = $derived(
    c.hp <= 0 ? "dead" : c.maxHp && c.hp <= c.maxHp / 2 ? "bloodied" : "healthy"
  );
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div
  class="row"
  class:active
  class:selected
  class:dead={c.hp <= 0}
  onclick={() => onSelect(c.id)}
  role="button"
  tabindex="-1"
>
  <button
    class="turn-slot"
    type="button"
    title={active ? "Current turn" : "Set turn here"}
    onclick={(e) => {
      e.stopPropagation();
      onSetTurn(c.id);
    }}
  >
    {#if active}<span class="turn-beacon t-mono-sm">▸</span>{/if}
  </button>

  <div class="cell-init">
    <input
      class="num t-mono-lg"
      type="text"
      inputmode="numeric"
      placeholder="—"
      bind:value={initInput}
      onkeydown={(e) => {
        if (e.key === "Enter") (e.target as HTMLInputElement).blur();
      }}
      onblur={parseInitiative}
      onclick={(e) => e.stopPropagation()}
      title="Initiative total ({c.initiativeMod != null ? `mod ${c.initiativeMod >= 0 ? "+" : ""}${c.initiativeMod}` : "no mod"})"
    />
    {#if c.initiativeMod != null}
      <span class="mod t-mono-sm">{c.initiativeMod >= 0 ? "+" : ""}{c.initiativeMod}</span>
    {/if}
  </div>

  <div class="cell-name">
    <button
      class="c-name-btn"
      type="button"
      title={c.statblock ? `Open statblock (${c.statblock})` : "Open statblock by name"}
      onclick={(e) => {
        e.stopPropagation();
        onOpenStatblock(c.id);
      }}
    >
      <span class="c-name t-body-lg">{c.name}</span>
    </button>
    <span class="c-kind t-mono-sm">{c.kind}</span>
    <div class="cond-row">
      {#each c.conditions as cond (cond)}
        <button
          class="cond t-mono-sm"
          type="button"
          title="Click to remove"
          onclick={(e) => {
            e.stopPropagation();
            onToggleCondition(c.id, cond);
          }}
        >
          <span class="cond-dot"></span>{cond}
        </button>
      {/each}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <details class="cond-details" onclick={(e) => e.stopPropagation()}>
        <summary class="cond-add t-mono-sm" title="Toggle conditions">+</summary>
        <ul class="cond-menu">
          {#each CONDITIONS as cond (cond)}
            <li>
              <button
                type="button"
                class="t-body-sm"
                class:on={c.conditions.includes(cond)}
                onclick={() => onToggleCondition(c.id, cond)}
              >
                {cond}
              </button>
            </li>
          {/each}
        </ul>
      </details>
    </div>
  </div>

  <div class="cell-hp">
    <input
      class="num t-mono-lg hp-{hpTone}"
      type="text"
      inputmode="numeric"
      bind:value={hpInput}
      onfocus={() => {
        hpFocused = true;
        hpInput = String(c.hp);
      }}
      onblur={parseHp}
      onkeydown={(e) => {
        if (e.key === "Enter") (e.target as HTMLInputElement).blur();
        if (e.key === "Escape") {
          hpInput = String(c.hp);
          (e.target as HTMLInputElement).blur();
        }
      }}
      onclick={(e) => e.stopPropagation()}
      title="HP — type a value, or +12 / −8 to adjust"
    />
    <span class="hp-max t-mono-sm">{c.maxHp != null ? `/ ${c.maxHp}` : ""}</span>
    <div class="hp-track"><div class="hp-bar hp-{hpTone}" style="width:{hpRatio}%"></div></div>
    {#if c.tempHp > 0}
      <input
        class="num t-mono-sm temp"
        type="text"
        inputmode="numeric"
        bind:value={tempInput}
        onblur={parseTemp}
        onkeydown={(e) => {
          if (e.key === "Enter") (e.target as HTMLInputElement).blur();
        }}
        onclick={(e) => e.stopPropagation()}
        title="Temp HP"
      />
      <span class="temp-label t-mono-sm">T</span>
    {/if}
  </div>

  <div class="cell-ac">
    {#if c.ac != null}
      <span class="ac-chip t-mono-lg" title="Armor Class">{c.ac}</span>
    {:else}
      <span class="ac-chip t-mono-lg empty">—</span>
    {/if}
  </div>

  <button
    class="remove t-mono-sm"
    type="button"
    title="Remove from encounter"
    onclick={(e) => {
      e.stopPropagation();
      onRemove(c.id);
    }}
  >
    ×
  </button>
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: 1.75rem 3.6rem minmax(11rem, 1fr) 12.5rem 3.5rem 1.5rem;
    align-items: center;
    gap: var(--gutter-sm);
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded);
    padding: var(--gutter-xs) var(--gutter-sm);
    cursor: pointer;
    transition: background 100ms ease, box-shadow 100ms ease, border-color 100ms ease;
    position: relative;
    min-height: 2.6rem;
  }

  .row:hover {
    background: var(--surface-highlight);
  }

  .row.selected {
    border-color: var(--border-focus);
  }

  /* Active turn: amber beacon + arcane glow */
  .row.active {
    background: var(--surface-overlay);
    border-color: rgba(224, 169, 83, 0.45);
    box-shadow:
      inset 3px 0 0 var(--amber),
      var(--glow-amber);
  }

  .turn-slot {
    width: 100%;
    height: 100%;
    min-height: 1.75rem;
    background: transparent;
    border: none;
    color: var(--amber);
    cursor: pointer;
    border-radius: var(--rounded-sm);
  }

  .turn-beacon {
    text-shadow: 0 0 8px rgba(224, 169, 83, 0.8);
  }

  .cell-init {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0;
  }

  .mod {
    color: var(--text-tertiary);
    line-height: 1;
  }

  .num {
    width: 3.2rem;
    text-align: center;
    color: var(--text-primary);
    background: var(--surface-base);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: 2px 4px;
    outline: none;
    transition: border-color 100ms ease, box-shadow 100ms ease;
  }

  .num:focus {
    border-color: var(--amber);
    box-shadow: 0 0 0 1px var(--border-focus);
  }

  .num::placeholder {
    color: var(--text-tertiary);
  }

  .num.hp-healthy {
    color: var(--emerald);
  }

  .num.hp-bloodied {
    color: var(--amber-bright);
  }

  .num.hp-dead {
    color: var(--crimson-bright);
  }

  .cell-name {
    min-width: 0;
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--gutter-xs) var(--gutter-sm);
    padding-right: var(--gutter-xs);
  }

  .c-name-btn {
    background: transparent;
    border: none;
    padding: 0;
    cursor: pointer;
    min-width: 0;
    text-align: left;
  }

  .c-name-btn:hover .c-name {
    color: var(--amber);
  }

  .c-name {
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
    transition: color 100ms ease;
  }

  .row.dead .c-name {
    color: var(--crimson-bright);
    text-decoration: line-through;
    text-decoration-color: rgba(225, 29, 72, 0.6);
  }

  .c-kind {
    color: var(--text-tertiary);
    text-transform: uppercase;
    font-size: 9px;
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: 0 4px;
  }

  .cond-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--gutter-xs);
    width: 100%;
  }

  .cond {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--crimson-bright);
    background: var(--surface-overlay);
    border: 1px solid rgba(225, 29, 72, 0.3);
    border-radius: var(--rounded-sm);
    padding: 1px 6px;
    cursor: pointer;
  }

  .cond:hover {
    background: var(--surface-highlight);
  }

  .cond-dot {
    width: 5px;
    height: 5px;
    border-radius: var(--rounded-full);
    background: var(--crimson);
  }

  .cond-details {
    position: relative;
  }

  .cond-add {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.25rem;
    height: 1.25rem;
    color: var(--text-tertiary);
    border: 1px dashed var(--border-static);
    border-radius: var(--rounded-sm);
    cursor: pointer;
    list-style: none;
  }

  .cond-add:hover {
    color: var(--amber);
    border-color: var(--border-focus);
  }

  .cond-details[open] .cond-add {
    color: var(--amber);
    border-color: var(--border-focus);
  }

  .cond-menu {
    position: absolute;
    z-index: 30;
    top: calc(100% + 4px);
    left: 0;
    min-width: 11rem;
    max-height: 16rem;
    overflow-y: auto;
    margin: 0;
    padding: var(--gutter-xs);
    list-style: none;
    background: var(--surface-overlay);
    border: 1px solid var(--border-focus);
    border-radius: var(--rounded);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  }

  .cond-menu button {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-radius: var(--rounded-sm);
    padding: 3px 8px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .cond-menu button:hover {
    background: var(--surface-highlight);
    color: var(--text-primary);
  }

  .cond-menu button.on {
    color: var(--amber);
  }

  .cell-hp {
    display: flex;
    align-items: center;
    gap: var(--gutter-xs);
  }

  .hp-max {
    color: var(--text-tertiary);
  }

  .hp-track {
    flex: 1;
    height: 4px;
    min-width: 3rem;
    background: var(--surface-base);
    border-radius: var(--rounded-full);
    overflow: hidden;
  }

  .hp-bar {
    height: 100%;
    border-radius: var(--rounded-full);
    transition: width 150ms ease;
  }

  .hp-bar.hp-healthy {
    background: var(--emerald);
  }

  .hp-bar.hp-bloodied {
    background: var(--amber-bright);
  }

  .hp-bar.hp-dead {
    background: var(--crimson);
  }

  .num.temp {
    width: 2.6rem;
  }

  .temp-label {
    color: var(--sapphire);
  }

  .cell-ac {
    display: flex;
    justify-content: center;
  }

  .ac-chip {
    color: var(--sapphire);
    background: var(--surface-base);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: 2px 8px;
  }

  .ac-chip.empty {
    color: var(--text-tertiary);
  }

  .remove {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    border-radius: var(--rounded-sm);
    height: 1.5rem;
    transition: color 100ms ease, background 100ms ease;
  }

  .remove:hover {
    color: var(--crimson-bright);
    background: var(--surface-base);
  }
</style>
