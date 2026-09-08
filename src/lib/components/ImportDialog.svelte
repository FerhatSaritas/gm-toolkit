<script lang="ts">
  import type { ImportPreview, NamedText, StatblockMeta, Abilities } from "../commands";
  import { commitImport } from "../commands";
  import { notify } from "../stores/campaign.svelte";
  import { refreshCatalog } from "../stores/catalog.svelte";

  interface Props {
    preview: ImportPreview;
    onClose: () => void;
  }

  let { preview, onClose }: Props = $props();

  // The preview payload is static for this dialog's lifetime; the parent
  // remounts the dialog per import, so capturing the initial value is intended.
  // svelte-ignore state_referenced_locally
  let meta = $state<StatblockMeta>({ ...preview.statblock });
  let committing = $state(false);

  function slug(s: string): string {
    return (
      s
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-+|-+$/g, "") || "combatant"
    );
  }

  const targetPath = $derived.by(() => {
    const folder =
      meta.kind === "npc" ? "npcs" : meta.kind === "player" ? "party" : "statblocks";
    return `${folder}/${slug(meta.name)}.md`;
  });

  function bonus(score: number | unknown): string {
    if (score == null || typeof score !== "number") return "—";
    return Math.floor((score - 10) / 2) >= 0 ? `+${Math.floor((score - 10) / 2)}` : `${Math.floor((score - 10) / 2)}`;
  }

  const ABILITY_KEYS: [string, keyof Abilities][] = [
    ["STR", "strength"],
    ["DEX", "dexterity"],
    ["CON", "constitution"],
    ["INT", "intelligence"],
    ["WIS", "wisdom"],
    ["CHA", "charisma"]
  ];

  const sections = $derived<[string, NamedText[]][]>([
    ["Traits", meta.traits],
    ["Actions", meta.actions],
    ["Reactions", meta.reactions]
  ]);

  const customKeys = $derived(Object.keys(preview.statblock).filter((k) =>
    ![
      "title", "name", "kind", "hp", "ac", "initiative", "speed", "power",
      "abilities", "traits", "actions", "reactions", "tags", "docType"
    ].includes(k)
  ));

  async function doCommit() {
    committing = true;
    try {
      const rel = await commitImport(meta, preview.body);
      notify(`Imported → ${rel}`);
      await refreshCatalog();
      onClose();
    } catch (e) {
      notify(String(e), "error");
    } finally {
      committing = false;
    }
  }
</script>

<div class="overlay" role="presentation" onkeydown={(e) => e.key === "Escape" && onClose()}>
  <div class="modal" role="dialog" aria-label="Import statblock preview">
    <header class="head">
      <h2 class="t-headline-sm">Import Statblock</h2>
      <span class="format t-mono-sm">{preview.format === "json" ? "JSON" : "Markdown"}</span>
      <button class="close t-mono-sm" type="button" onclick={onClose}>×</button>
    </header>

    {#if preview.warnings.length > 0}
      <ul class="warnings">
        {#each preview.warnings as w (w)}
          <li class="t-body-sm"><span class="dot"></span>{w}</li>
        {/each}
      </ul>
    {:else}
      <p class="clean t-body-sm">Mapped cleanly — no warnings.</p>
    {/if}

    <div class="scroll">
      <div class="fields">
        <div class="field name-field">
          <span class="f-label t-label-lg">Name</span>
          <input class="t-body-md f-input" type="text" bind:value={meta.name} />
        </div>
        <div class="field">
          <span class="f-label t-label-lg">Kind</span>
          <select class="t-body-md f-input" bind:value={meta.kind}>
            <option value="monster">Monster</option>
            <option value="npc">NPC</option>
            <option value="player">Player</option>
          </select>
        </div>
      </div>

      <div class="chips">
        {#if meta.hp != null}<span class="chip hp t-mono-lg">{meta.hp} hp</span>{/if}
        {#if meta.ac != null}<span class="chip ac t-mono-lg">ac {meta.ac}</span>{/if}
        {#if meta.initiative != null}<span class="chip init t-mono-lg">init {meta.initiative >= 0 ? "+" : ""}{meta.initiative}</span>{/if}
        {#if meta.speed}<span class="chip t-mono-lg">{meta.speed}</span>{/if}
        {#if meta.power != null}<span class="chip pwr t-mono-lg">pwr {meta.power}</span>{/if}
      </div>

      {#if meta.abilities}
        <div class="abilities">
          {#each ABILITY_KEYS as [label, key] (label)}
            {@const value = meta.abilities?.[key] as number | undefined}
            <div class="ability" class:empty={value == null}>
              <span class="ab-label t-label-lg">{label}</span>
              <span class="ab-bonus t-mono-lg">{bonus(value)}</span>
              <span class="ab-score t-mono-sm">{value ?? ""}</span>
            </div>
          {/each}
        </div>
      {/if}

      {#each sections as [title, list], si (si + title)}
        {#if list.length > 0}
          <div class="section">
            <h3 class="sec-title t-label-lg">{title}</h3>
            {#each list as entry (entry.name)}
              <p class="entry t-body-sm"><span class="entry-name">{entry.name}.</span> {entry.text}</p>
            {/each}
          </div>
        {/if}
      {/each}

      {#if customKeys.length > 0}
        <p class="custom-note t-body-sm">
          {customKeys.length} custom field(s) preserved: {customKeys.join(", ")}
        </p>
      {/if}

      <p class="target t-mono-sm" title="Destination file">→ {targetPath}</p>
      <p class="source t-mono-sm">from {preview.sourcePath}</p>
    </div>

    <footer class="foot">
      <button class="btn btn-secondary t-label-lg" type="button" onclick={onClose}>Cancel</button>
      <button class="btn btn-primary t-label-lg" type="button" disabled={committing || !meta.name.trim()} onclick={doCommit}>
        {committing ? "Importing…" : "Import"}
      </button>
    </footer>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(8, 10, 13, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal {
    width: 34rem;
    max-width: 92vw;
    max-height: 86vh;
    display: flex;
    flex-direction: column;
    background: var(--surface-raised);
    border: 1px solid var(--border-focus);
    border-radius: var(--rounded-lg);
    box-shadow:
      inset 0 1px 0 var(--border-bevel-light),
      0 16px 40px rgba(0, 0, 0, 0.6),
      var(--glow-amber);
  }

  .head {
    display: flex;
    align-items: center;
    gap: var(--gutter-sm);
    padding: var(--gutter-md) var(--gutter-lg);
    border-bottom: 1px solid var(--border-static);
  }

  .head h2 {
    margin: 0;
    color: var(--text-primary);
    flex: 1;
  }

  .format {
    color: var(--sapphire);
    border: 1px solid rgba(56, 189, 248, 0.35);
    border-radius: var(--rounded-sm);
    padding: 1px 6px;
    font-size: 9px;
  }

  .close {
    width: 1.5rem;
    height: 1.5rem;
    background: transparent;
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    color: var(--text-tertiary);
    cursor: pointer;
  }

  .close:hover {
    color: var(--crimson-bright);
    border-color: var(--crimson);
  }

  .warnings {
    margin: 0;
    padding: var(--gutter-sm) var(--gutter-lg);
    list-style: none;
    background: rgba(224, 169, 83, 0.06);
    border-bottom: 1px solid var(--border-focus);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .warnings li {
    display: flex;
    align-items: baseline;
    gap: 6px;
    color: var(--amber);
  }

  .dot {
    width: 5px;
    height: 5px;
    flex: none;
    border-radius: var(--rounded-full);
    background: var(--amber);
  }

  .clean {
    margin: 0;
    padding: var(--gutter-sm) var(--gutter-lg);
    color: var(--emerald);
    border-bottom: 1px solid var(--border-static);
  }

  .scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--gutter-md) var(--gutter-lg);
    display: flex;
    flex-direction: column;
    gap: var(--gutter-md);
  }

  .fields {
    display: flex;
    gap: var(--gutter-md);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .name-field {
    flex: 1;
  }

  .f-label {
    color: var(--text-tertiary);
    font-size: 9px;
    text-transform: uppercase;
  }

  .f-input {
    background: var(--surface-base);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    color: var(--text-primary);
    padding: 4px 8px;
    outline: none;
  }

  .f-input:focus {
    border-color: var(--amber);
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--gutter-xs);
  }

  .chip {
    background: var(--surface-base);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: 2px 8px;
    color: var(--text-primary);
  }

  .chip.hp {
    color: var(--emerald);
  }

  .chip.ac {
    color: var(--sapphire);
  }

  .chip.init {
    color: var(--amber);
  }

  .chip.pwr {
    color: var(--text-secondary);
  }

  .abilities {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: var(--gutter-xs);
    background: var(--surface-base);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded);
    padding: var(--gutter-xs);
  }

  .ability {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .ability.empty {
    opacity: 0.35;
  }

  .ab-label {
    color: var(--text-tertiary);
    font-size: 9px;
  }

  .ab-bonus {
    color: var(--amber);
  }

  .ab-score {
    color: var(--text-secondary);
  }

  .sec-title {
    margin: 0 0 var(--gutter-xs);
    color: var(--amber);
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.06em;
  }

  .entry {
    margin: 0 0 4px;
    color: var(--text-secondary);
  }

  .entry-name {
    color: var(--text-primary);
  }

  .custom-note {
    margin: 0;
    color: var(--text-tertiary);
  }

  .target {
    margin: 0;
    color: var(--amber);
  }

  .source {
    margin: 0;
    color: var(--text-tertiary);
    word-break: break-all;
  }

  .foot {
    display: flex;
    justify-content: flex-end;
    gap: var(--gutter-sm);
    padding: var(--gutter-md) var(--gutter-lg);
    border-top: 1px solid var(--border-static);
  }

  .foot .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
