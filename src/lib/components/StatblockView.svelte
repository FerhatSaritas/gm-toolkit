<script lang="ts">
  import type { Doc, NamedText } from "../commands";
  import { roll } from "../stores/dice.svelte";
  import { closeStatblock } from "../stores/inspector.svelte";

  let { doc }: { doc: Doc } = $props();

  const meta = $derived(doc.statblock!);

  interface Seg {
    text: string;
    expr?: string;
  }

  const DICE_RE = /\b(\d*\s*d\s*\d+(?:\s*(?:kh|kl)\s*\d+)?(?:\s*[+-]\s*\d+)?)\b/gi;

  function diceify(text: string): Seg[] {
    const segs: Seg[] = [];
    let last = 0;
    for (const m of text.matchAll(DICE_RE)) {
      const idx = m.index ?? 0;
      if (idx > last) segs.push({ text: text.slice(last, idx) });
      segs.push({ text: m[1], expr: m[1].replace(/\s+/g, "").toLowerCase() });
      last = idx + m[1].length;
    }
    if (last < text.length) segs.push({ text: text.slice(last) });
    return segs;
  }

  type AbilityKey = "strength" | "dexterity" | "constitution" | "intelligence" | "wisdom" | "charisma";

  const ABILITY_KEYS: [string, AbilityKey][] = [
    ["STR", "strength"],
    ["DEX", "dexterity"],
    ["CON", "constitution"],
    ["INT", "intelligence"],
    ["WIS", "wisdom"],
    ["CHA", "charisma"]
  ];

  function bonus(score?: number): string | null {
    if (score == null) return null;
    return Math.floor((score - 10) / 2) >= 0
      ? `+${Math.floor((score - 10) / 2)}`
      : `${Math.floor((score - 10) / 2)}`;
  }

  async function doRoll(expr: string) {
    try {
      await roll(expr);
    } catch {
      /* notify comes from the store layer on failure */
    }
  }

  function section(list: NamedText[]): boolean {
    return list.length > 0;
  }
</script>

<div class="statblock">
  <header class="sb-head">
    <div class="sb-title-row">
      <h2 class="sb-name t-headline-lg">{meta.name}</h2>
      <button class="sb-close t-mono-sm" type="button" title="Close" onclick={closeStatblock}>×</button>
    </div>
    <div class="sb-kind-row">
      <span class="sb-kind t-mono-sm">{meta.kind}</span>
      {#if meta.power != null}
        <span class="sb-power t-mono-sm" title="Power rating">PWR {meta.power}</span>
      {/if}
      {#each meta.tags as tag (tag)}
        <span class="sb-tag t-mono-sm">{tag}</span>
      {/each}
    </div>
    <div class="sb-rule" role="separator"></div>
  </header>

  <div class="sb-core">
    {#if meta.hp != null}
      <div class="core-chip"><span class="core-label t-label-lg">HP</span><span class="t-mono-lg hp">{meta.hp}</span></div>
    {/if}
    {#if meta.ac != null}
      <div class="core-chip"><span class="core-label t-label-lg">AC</span><span class="t-mono-lg ac">{meta.ac}</span></div>
    {/if}
    {#if meta.initiative != null}
      <div class="core-chip"><span class="core-label t-label-lg">INIT</span><span class="t-mono-lg init">{meta.initiative >= 0 ? "+" : ""}{meta.initiative}</span></div>
    {/if}
    {#if meta.speed}
      <div class="core-chip"><span class="core-label t-label-lg">SPEED</span><span class="t-mono-lg speed">{meta.speed}</span></div>
    {/if}
  </div>

  {#if meta.abilities}
    <div class="abilities">
      {#each ABILITY_KEYS as [label, key] (label)}
        {@const value = meta.abilities?.[key]}
        <div class="ability" class:empty={value == null}>
          <span class="ab-label t-label-lg">{label}</span>
          <span class="ab-bonus t-mono-lg">{bonus(value) ?? "—"}</span>
          <span class="ab-score t-mono-sm">{value ?? ""}</span>
        </div>
      {/each}
    </div>
  {/if}

  <div class="sb-body">
    {#if section(meta.traits)}
      <div class="sb-section">
        <h3 class="sb-section-title t-headline-sm">Traits</h3>
        {#each meta.traits as t (t.name)}
          <div class="entry">
            <span class="entry-name t-label-lg">{t.name}.</span>
            <span class="entry-text t-body-md">
              {#each diceify(t.text) as seg, i (i)}
                {#if seg.expr}
                  <button class="die t-mono-sm" type="button" onclick={() => doRoll(seg.expr!)} title={`Roll ${seg.expr}`}>
                    {seg.text.trim()}
                  </button>
                {:else}
                  {seg.text}
                {/if}
              {/each}
            </span>
          </div>
        {/each}
      </div>
    {/if}

    {#if section(meta.actions)}
      <div class="sb-section">
        <h3 class="sb-section-title t-headline-sm">Actions</h3>
        {#each meta.actions as t (t.name)}
          <div class="entry">
            <span class="entry-name t-label-lg">{t.name}.</span>
            <span class="entry-text t-body-md">
              {#each diceify(t.text) as seg, i (i)}
                {#if seg.expr}
                  <button class="die t-mono-sm" type="button" onclick={() => doRoll(seg.expr!)} title={`Roll ${seg.expr}`}>
                    {seg.text.trim()}
                  </button>
                {:else}
                  {seg.text}
                {/if}
              {/each}
            </span>
          </div>
        {/each}
      </div>
    {/if}

    {#if section(meta.reactions)}
      <div class="sb-section">
        <h3 class="sb-section-title t-headline-sm">Reactions</h3>
        {#each meta.reactions as t (t.name)}
          <div class="entry">
            <span class="entry-name t-label-lg">{t.name}.</span>
            <span class="entry-text t-body-md">
              {#each diceify(t.text) as seg, i (i)}
                {#if seg.expr}
                  <button class="die t-mono-sm" type="button" onclick={() => doRoll(seg.expr!)} title={`Roll ${seg.expr}`}>
                    {seg.text.trim()}
                  </button>
                {:else}
                  {seg.text}
                {/if}
              {/each}
            </span>
          </div>
        {/each}
      </div>
    {/if}

    <p class="sb-path t-mono-sm">{doc.relPath}</p>
  </div>
</div>

<style>
  .statblock {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .sb-head {
    flex: none;
  }

  .sb-title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--gutter-sm);
  }

  .sb-name {
    margin: 0;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sb-close {
    flex: none;
    width: 1.5rem;
    height: 1.5rem;
    background: transparent;
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 100ms ease;
  }

  .sb-close:hover {
    color: var(--crimson-bright);
    border-color: var(--crimson);
  }

  .sb-kind-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--gutter-xs);
    margin-top: var(--gutter-xs);
  }

  .sb-kind {
    color: var(--sapphire);
    text-transform: uppercase;
    font-size: 9px;
    border: 1px solid rgba(56, 189, 248, 0.35);
    border-radius: var(--rounded-sm);
    padding: 1px 6px;
  }

  .sb-power {
    color: var(--amber);
    text-transform: uppercase;
    font-size: 9px;
    border: 1px solid var(--border-focus);
    border-radius: var(--rounded-sm);
    padding: 1px 6px;
  }

  .sb-tag {
    color: var(--text-tertiary);
    font-size: 9px;
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: 1px 6px;
  }

  /* Amber → crimson dual-stop rule divider (per spec) */
  .sb-rule {
    height: 2px;
    margin-top: var(--gutter-sm);
    border-radius: var(--rounded-full);
    background: linear-gradient(to right, var(--amber), var(--crimson) 70%, transparent 100%);
  }

  .sb-core {
    display: flex;
    flex-wrap: wrap;
    gap: var(--gutter-xs);
    margin-top: var(--gutter-sm);
  }

  .core-chip {
    display: flex;
    align-items: baseline;
    gap: 6px;
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: 3px 8px;
  }

  .core-label {
    color: var(--text-tertiary);
    font-size: 9px;
    text-transform: uppercase;
  }

  .core-chip .hp {
    color: var(--emerald);
  }

  .core-chip .ac {
    color: var(--sapphire);
  }

  .core-chip .init {
    color: var(--amber);
  }

  .core-chip .speed {
    color: var(--text-primary);
  }

  .abilities {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: var(--gutter-xs);
    margin-top: var(--gutter-sm);
    background: var(--surface-base);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded);
    padding: var(--gutter-xs);
  }

  .ability {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0;
    border-radius: var(--rounded-sm);
    padding: 3px 0;
  }

  .ability.empty {
    opacity: 0.35;
  }

  .ab-label {
    color: var(--text-tertiary);
    font-size: 9px;
    text-transform: uppercase;
  }

  .ab-bonus {
    color: var(--amber);
    line-height: 1.4;
  }

  .ab-score {
    color: var(--text-secondary);
  }

  .sb-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    margin-top: var(--gutter-sm);
    display: flex;
    flex-direction: column;
    gap: var(--gutter-md);
    padding-bottom: var(--gutter-sm);
  }

  .sb-section-title {
    margin: 0 0 var(--gutter-xs);
    color: var(--amber);
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .entry {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: var(--gutter-xs) 0;
    border-bottom: 1px solid var(--border-static);
  }

  .entry:last-child {
    border-bottom: none;
  }

  .entry-name {
    color: var(--amber);
  }

  .entry-text {
    color: var(--text-secondary);
  }

  .die {
    display: inline-flex;
    align-items: center;
    font-family: var(--font-mono);
    font-size: var(--fs-mono-sm);
    color: var(--amber-bright);
    background: var(--surface-base);
    border: 1px solid var(--border-focus);
    border-radius: var(--rounded-sm);
    padding: 0 5px;
    margin: 0 1px;
    cursor: pointer;
    transition: all 100ms ease;
  }

  .die:hover {
    color: var(--surface-canvas);
    background: var(--amber);
    box-shadow: var(--glow-amber);
  }

  .sb-path {
    color: var(--text-tertiary);
    margin: 0;
    word-break: break-all;
  }
</style>
