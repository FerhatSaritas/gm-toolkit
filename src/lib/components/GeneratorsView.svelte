<script lang="ts">
  import { onMount } from "svelte";
  import {
    generateEncounterDraft,
    generateFromPack,
    saveGeneratedEncounter,
    saveGeneratedItem,
    saveGeneratedStatblock
  } from "../commands";
  import type { Generated, PackInfo, StatblockSummary } from "../commands";
  import { forge, refreshPacks } from "../stores/generators.svelte";
  import { refreshCatalog } from "../stores/catalog.svelte";
  import { showStatblock } from "../stores/inspector.svelte";
  import { openRelPath, notify } from "../stores/campaign.svelte";

  let selectedId = $state<string | null>(null);
  let template = $state("");
  let result = $state<Generated | null>(null);

  let saveName = $state("");
  let saveRarity = $state("");
  let saveKind = $state("");
  let saveValue = $state("");
  let saveCreatureKind = $state<"npc" | "monster">("npc");

  let encTitle = $state("");
  let partyLevel = $state(3);
  let band = $state(1);
  let count = $state(3);
  let draft = $state<StatblockSummary[]>([]);
  let savedPath = $state<string | null>(null);
  let busy = $state(false);

  onMount(() => {
    void refreshPacks();
  });

  const selected = $derived(forge.packs.find((p) => p.id === selectedId) ?? null);

  function pickPack(p: PackInfo) {
    selectedId = p.id;
    result = null;
    savedPath = null;
    template = p.templates[0] ?? "";
    draft = [];
  }

  async function doGenerate() {
    if (!selected) return;
    busy = true;
    try {
      result = await generateFromPack(selected.id, template || undefined);
      saveName = result.fields["name"] ?? "";
      saveRarity = result.fields["rarity"] ?? "";
      saveKind = result.fields["kind"] ?? "";
      saveValue = result.fields["value"] ?? "";
      if (selected.kind === "encounter") {
        encTitle = "";
      }
    } catch (e) {
      notify(String(e), "error");
    } finally {
      busy = false;
    }
  }

  async function rollCreatures() {
    busy = true;
    try {
      draft = await generateEncounterDraft(partyLevel, band, count);
      if (draft.length === 0) notify("No creatures rolled", "error");
    } catch (e) {
      notify(String(e), "error");
    } finally {
      busy = false;
    }
  }

  const PACK_ORDER = ["npc", "item", "encounter"] as const;

  const grouped = $derived.by(() => {
    return PACK_ORDER.map((k) => ({
      kind: k,
      label: k === "npc" ? "NPC" : k === "item" ? "Item" : "Encounter",
      packs: forge.packs.filter((p) => p.kind === k)
    })).filter((g) => g.packs.length > 0);
  });

  async function saveNpc() {
    if (!result || !saveName.trim()) return;
    busy = true;
    try {
      savedPath = await saveGeneratedStatblock(
        {
          name: saveName.trim(),
          kind: saveCreatureKind,
          tags: [],
          traits: [],
          actions: [],
          reactions: []
        },
        result.text
      );
      notify(`Saved → ${savedPath}`);
      await refreshPacks();
      await refreshCatalog();
    } catch (e) {
      notify(String(e), "error");
    } finally {
      busy = false;
    }
  }

  async function saveItem() {
    if (!result || !saveName.trim()) return;
    busy = true;
    try {
      savedPath = await saveGeneratedItem(
        {
          name: saveName.trim(),
          rarity: saveRarity.trim() || undefined,
          kind: saveKind.trim() || undefined,
          value: saveValue.trim() || undefined,
          tags: []
        },
        result.text
      );
      notify(`Saved → ${savedPath}`);
      await refreshPacks();
      await refreshCatalog();
    } catch (e) {
      notify(String(e), "error");
    } finally {
      busy = false;
    }
  }

  async function saveEncounter() {
    if (!encTitle.trim()) {
      notify("Give the encounter a title", "error");
      return;
    }
    busy = true;
    try {
      savedPath = await saveGeneratedEncounter(
        encTitle.trim(),
        partyLevel,
        draft.map((c) => ({
          name: c.name,
          kind: c.kind,
          hp: c.hp,
          ac: c.ac,
          initiative: c.initiative,
          statblock: c.relPath,
          tags: c.tags
        })),
        result?.text ?? ""
      );
      notify(`Saved → ${savedPath}`);
      await refreshPacks();
      await refreshCatalog();
    } catch (e) {
      notify(String(e), "error");
    } finally {
      busy = false;
    }
  }
</script>

<div class="forge">
  <aside class="pack-list">
    <div class="pack-head t-label-lg">Packs</div>
    {#each grouped as group (group.kind)}
      <div class="pack-group-label t-label-lg">{group.label}</div>
      {#each group.packs as p (p.id)}
        <button
          class="pack t-body-md"
          type="button"
          class:selected={selectedId === p.id}
          class:broken={p.error}
          title={p.error ?? `${p.source} pack — ${p.tables.length} tables`}
          onclick={() => pickPack(p)}
        >
          <span class="pack-name">{p.name}</span>
          <span class="pack-src t-mono-sm" class:user-src={p.source === "user"}>{p.source}</span>
        </button>
      {/each}
    {/each}
    <p class="pack-hint t-mono-sm">Add packs: &lt;campaign&gt;/generators/*.yaml</p>
  </aside>

  <section class="panel">
    {#if !selected}
      <div class="placeholder">
        <p class="t-body-lg">The Forge</p>
        <p class="t-body-sm ph-sub">
          Pick a pack to generate NPCs, items, or encounters — then save the
          result straight into your campaign files.
        </p>
      </div>
    {:else if selected.error}
      <div class="placeholder">
        <p class="t-body-md pack-error">{selected.name} failed to load</p>
        <p class="t-mono-sm ph-sub">{selected.error}</p>
      </div>
    {:else}
      <div class="gen-head">
        <h2 class="t-headline-sm">{selected.name}</h2>
        {#if selected.templates.length > 1}
          <select class="field t-body-md" bind:value={template}>
            {#each selected.templates as t (t)}
              <option value={t}>{t}</option>
            {/each}
          </select>
        {/if}
        <button class="btn btn-primary t-label-lg" type="button" disabled={busy} onclick={doGenerate}>
          Generate
        </button>
      </div>

      {#if result}
        {#if selected.kind !== "encounter"}
          <div class="result md-body t-body-md">{result.text}</div>
        {/if}

        {#if selected.kind === "npc"}
          <div class="save-row">
            <input class="field t-body-md grow" type="text" placeholder="Name" bind:value={saveName} />
            <select class="field t-body-md" bind:value={saveCreatureKind}>
              <option value="npc">NPC</option>
              <option value="monster">Monster</option>
            </select>
            <button class="btn btn-primary t-label-lg" type="button" disabled={busy || !saveName.trim()} onclick={saveNpc}>
              Save
            </button>
          </div>
        {:else if selected.kind === "item"}
          <div class="save-row">
            <input class="field t-body-md grow" type="text" placeholder="Name" bind:value={saveName} />
            <input class="field t-body-md" type="text" placeholder="Rarity" bind:value={saveRarity} />
            <input class="field t-body-md" type="text" placeholder="Kind" bind:value={saveKind} />
            <input class="field t-body-md" type="text" placeholder="Value" bind:value={saveValue} />
            <button class="btn btn-primary t-label-lg" type="button" disabled={busy || !saveName.trim()} onclick={saveItem}>
              Save
            </button>
          </div>
        {:else if selected.kind === "encounter"}
          <div class="enc-controls">
            <label class="t-label-lg">Party
              <input class="field num t-mono-lg" type="number" min="1" max="20" bind:value={partyLevel} /></label>
            <label class="t-label-lg">Band ±
              <input class="field num t-mono-lg" type="number" min="0" max="10" bind:value={band} /></label>
            <label class="t-label-lg">Count
              <input class="field num t-mono-lg" type="number" min="1" max="50" bind:value={count} /></label>
            <button class="btn btn-secondary t-label-lg" type="button" disabled={busy} onclick={rollCreatures}>
              Roll Creatures
            </button>
          </div>

          {#if draft.length > 0}
            <div class="draft">
              {#each draft as c, i (c.relPath + i)}
                <span class="draft-chip t-mono-sm">
                  <button class="chip-x" type="button" title="Remove" onclick={() => (draft = draft.filter((_, x) => x !== i))}>×</button>
                  {c.name}
                  {#if c.power != null}<span class="pwr">p{c.power}</span>{/if}
                  {#if c.hp != null}<span class="hp">{c.hp}hp</span>{/if}
                </span>
              {/each}
            </div>
          {/if}

          <input class="field t-body-md" type="text" placeholder="Encounter title" bind:value={encTitle} />
          <div class="save-row">
            <button class="btn btn-primary t-label-lg" type="button" disabled={busy || !encTitle.trim()} onclick={saveEncounter}>
              Save Encounter
            </button>
          </div>
        {/if}

        {#if savedPath}
          <button class="saved t-mono-sm" type="button" title="Open in library" onclick={() => (selected?.kind === "encounter" ? openRelPath(savedPath!) : showStatblock(null, savedPath!))}>
            saved → {savedPath} (open)
          </button>
        {/if}
      {:else}
        <p class="hint t-body-sm">Press Generate to draw from the tables.</p>
      {/if}
    {/if}
  </section>
</div>

<style>
  .forge {
    min-width: 0;
    min-height: 0;
    display: grid;
    grid-template-columns: 14rem 1fr;
    gap: 1px;
    background: var(--border-static);
  }

  .pack-list {
    background: var(--surface-raised);
    display: flex;
    flex-direction: column;
    padding: var(--gutter-sm);
    gap: 2px;
    overflow-y: auto;
  }

  .pack-head,
  .pack-group-label {
    color: var(--text-tertiary);
    text-transform: uppercase;
    font-size: 9px;
    letter-spacing: 0.08em;
    padding: var(--gutter-xs) var(--gutter-xs) 2px;
  }

  .pack {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--gutter-xs);
    background: transparent;
    border: none;
    border-radius: var(--rounded-sm);
    padding: 4px 8px;
    color: var(--text-secondary);
    cursor: pointer;
    text-align: left;
  }

  .pack:hover {
    background: var(--surface-highlight);
    color: var(--text-primary);
  }

  .pack.selected {
    background: var(--surface-overlay);
    color: var(--amber);
    box-shadow: inset 2px 0 0 var(--amber);
  }

  .pack.broken {
    opacity: 0.45;
    text-decoration: line-through;
  }

  .pack-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pack-src {
    color: var(--text-tertiary);
    font-size: 9px;
    text-transform: uppercase;
  }

  .pack-src.user-src {
    color: var(--emerald);
  }

  .pack-hint {
    margin-top: auto;
    color: var(--text-tertiary);
    font-size: 9px;
    padding: var(--gutter-xs);
    word-break: break-all;
  }

  .panel {
    background: var(--surface-base);
    padding: var(--inset-panel) var(--gutter-xl);
    display: flex;
    flex-direction: column;
    gap: var(--gutter-md);
    min-height: 0;
    overflow-y: auto;
  }

  .placeholder {
    margin: auto;
    text-align: center;
    max-width: 24rem;
  }

  .placeholder p {
    margin: 0;
  }

  .ph-sub {
    color: var(--text-tertiary);
    margin-top: var(--gutter-xs);
  }

  .pack-error {
    color: var(--crimson-bright);
  }

  .gen-head {
    display: flex;
    align-items: center;
    gap: var(--gutter-md);
  }

  .gen-head h2 {
    margin: 0;
    color: var(--text-primary);
    flex: 1;
  }

  .field {
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    color: var(--text-primary);
    padding: 4px 8px;
    outline: none;
  }

  .field:focus {
    border-color: var(--amber);
  }

  .field.grow {
    flex: 1;
    min-width: 6rem;
  }

  .field.num {
    width: 4.5rem;
  }

  .result {
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded);
    padding: var(--gutter-md) var(--gutter-lg);
    color: var(--text-primary);
    white-space: pre-wrap;
    min-height: 6rem;
  }

  .save-row {
    display: flex;
    gap: var(--gutter-xs);
    flex-wrap: wrap;
    align-items: center;
  }

  .enc-controls {
    display: flex;
    gap: var(--gutter-md);
    align-items: center;
    flex-wrap: wrap;
  }

  .enc-controls label {
    display: flex;
    align-items: center;
    gap: var(--gutter-xs);
    color: var(--text-secondary);
  }

  .draft {
    display: flex;
    flex-wrap: wrap;
    gap: var(--gutter-xs);
  }

  .draft-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-left: 2px solid var(--crimson);
    border-radius: var(--rounded-sm);
    padding: 3px 8px;
    color: var(--text-primary);
  }

  .chip-x {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 0;
    font-size: 11px;
  }

  .chip-x:hover {
    color: var(--crimson-bright);
  }

  .pwr {
    color: var(--text-secondary);
  }

  .hp {
    color: var(--emerald);
  }

  .hint {
    color: var(--text-tertiary);
  }

  .saved {
    align-self: flex-start;
    background: transparent;
    border: 1px solid var(--border-focus);
    border-radius: var(--rounded-sm);
    color: var(--emerald);
    padding: 2px 8px;
    cursor: pointer;
  }

  .saved:hover {
    box-shadow: var(--glow-amber);
  }
</style>
