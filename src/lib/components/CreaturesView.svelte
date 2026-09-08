<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { deleteFile, duplicateFile, addToEncounter, createEncounter, createStatblock, pickImportFile, requestImportPreview } from "../commands";
  import type { CreatureKind, ImportPreview, StatblockSummary } from "../commands";
  import { catalog, refreshCatalog } from "../stores/catalog.svelte";
  import { ui, showLibrary } from "../stores/ui.svelte";
  import { showStatblock } from "../stores/inspector.svelte";
  import { notify } from "../stores/campaign.svelte";
  import ImportDialog from "./ImportDialog.svelte";

  let query = $state("");
  let showNew = $state(false);
  let newName = $state("");
  let newKind = $state<CreatureKind>("monster");
  let newHp = $state("");
  let newAc = $state("");
  let openAddFor = $state<string | null>(null);
  let importPreviewData = $state<ImportPreview | null>(null);
  let dragActive = $state(false);

  const FILTERS: { key: "all" | "monster" | "npc" | "player"; label: string }[] = [
    { key: "all", label: "All" },
    { key: "monster", label: "Monsters" },
    { key: "npc", label: "NPCs" },
    { key: "player", label: "Players" }
  ];

  onMount(() => {
    void refreshCatalog();
    let unlisten: (() => void) | undefined;
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        if (ui.view !== "creatures") return;
        const p = event.payload;
        if (p.type === "enter" || p.type === "over") {
          dragActive = true;
        } else if (p.type === "leave") {
          dragActive = false;
        } else if (p.type === "drop") {
          dragActive = false;
          void handleDrop(p.paths[0]);
        }
      })
      .then((fn) => (unlisten = fn));
    return () => unlisten?.();
  });

  async function handleDrop(path: string | undefined) {
    if (!path) return;
    const lower = path.toLowerCase();
    if (!lower.endsWith(".md") && !lower.endsWith(".json")) {
      notify("Drop a .md or .json statblock file", "error");
      return;
    }
    try {
      importPreviewData = await requestImportPreview(path);
    } catch (e) {
      notify(String(e), "error");
    }
  }

  async function startImportDialog() {
    try {
      const path = await pickImportFile();
      if (!path) return;
      importPreviewData = await requestImportPreview(path);
    } catch (e) {
      notify(String(e), "error");
    }
  }

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return catalog.creatures.filter((c) => {
      if (ui.creatureFilter !== "all" && c.kind !== ui.creatureFilter) return false;
      if (!q) return true;
      return (
        c.name.toLowerCase().includes(q) ||
        c.tags.some((t) => t.toLowerCase().includes(q))
      );
    });
  });

  function kindLabel(kind: CreatureKind): string {
    return kind === "npc" ? "NPC" : kind === "player" ? "Player" : "Monster";
  }

  async function submitNew() {
    if (!newName.trim()) return;
    try {
      const hp = Number(newHp);
      const ac = Number(newAc);
      const rel = await createStatblock(
        newName,
        newKind,
        newHp.trim() && !Number.isNaN(hp) ? hp : undefined,
        newAc.trim() && !Number.isNaN(ac) ? ac : undefined
      );
      notify(`Created ${rel}`);
      newName = "";
      newHp = "";
      newAc = "";
      showNew = false;
      await refreshCatalog();
    } catch (e) {
      notify(String(e), "error");
    }
  }

  async function addCreatureToEncounter(c: StatblockSummary, encounterPath: string | null) {
    openAddFor = null;
    try {
      const input = {
        name: c.name,
        kind: c.kind,
        hp: c.hp,
        ac: c.ac,
        initiative: c.initiative,
        statblock: c.relPath,
        tags: c.tags
      };
      if (encounterPath) {
        const s = await addToEncounter(encounterPath, input);
        notify(`Added ${c.name} to “${s.title}” (${s.combatants} combatants)`);
      } else {
        const rel = await createEncounter(c.name, input);
        notify(`New encounter: ${rel}`);
      }
      await refreshCatalog();
    } catch (e) {
      notify(String(e), "error");
    }
  }

  async function copy(c: StatblockSummary) {
    try {
      const rel = await duplicateFile(c.relPath);
      notify(`Duplicated → ${rel}`);
      await refreshCatalog();
    } catch (e) {
      notify(String(e), "error");
    }
  }

  async function remove(c: StatblockSummary) {
    if (!confirm(`Delete “${c.name}” (${c.relPath})? This cannot be undone.`)) return;
    try {
      await deleteFile(c.relPath);
      notify(`Deleted ${c.name}`);
      await refreshCatalog();
    } catch (e) {
      notify(String(e), "error");
    }
  }
</script>

<div class="creatures">
  <div class="toolbar">
    <div class="pills">
      {#each FILTERS as f (f.key)}
        <button
          class="pill t-label-lg"
          type="button"
          class:active={ui.creatureFilter === f.key}
          onclick={() => (ui.creatureFilter = f.key)}
        >
          {f.label}
        </button>
      {/each}
    </div>
    <input
      class="search t-body-md"
      type="text"
      placeholder="Filter by name or tag…"
      bind:value={query}
      spellcheck="false"
    />
    <button class="btn btn-secondary t-label-lg" type="button" title="Import .md / .json" onclick={startImportDialog}>
      Import
    </button>
    <button class="btn btn-primary t-label-lg" type="button" onclick={() => (showNew = !showNew)}>
      + New
    </button>
    <button class="btn btn-secondary t-label-lg" type="button" title="Back to library" onclick={showLibrary}>
      Library
    </button>
  </div>

  {#if showNew}
    <div class="new-form">
      <input class="t-body-md field name" type="text" placeholder="Name" bind:value={newName} onkeydown={(e) => e.key === "Enter" && submitNew()} />
      <select class="t-body-md field" bind:value={newKind}>
        <option value="monster">Monster</option>
        <option value="npc">NPC</option>
        <option value="player">Player</option>
      </select>
      <input class="t-body-md field num" type="text" inputmode="numeric" placeholder="HP" bind:value={newHp} onkeydown={(e) => e.key === "Enter" && submitNew()} />
      <input class="t-body-md field num" type="text" inputmode="numeric" placeholder="AC" bind:value={newAc} onkeydown={(e) => e.key === "Enter" && submitNew()} />
      <button class="btn btn-primary t-label-lg" type="button" onclick={submitNew}>Create</button>
    </div>
  {/if}

  <div class="grid">
    {#if filtered.length === 0}
      <div class="empty t-body-md">
        {catalog.loading
          ? "Indexing campaign…"
          : "No creatures match. Create one with “+ New”, or import .md / .json files."}
      </div>
    {/if}
    {#each filtered as c (c.relPath)}
      <article class="card">
        <button class="card-main" type="button" onclick={() => showStatblock(null, c.relPath)}>
          <div class="card-head">
            <span class="name t-headline-sm">{c.name}</span>
            <span class="kind kind-{c.kind} t-mono-sm">{kindLabel(c.kind)}</span>
          </div>
          <div class="stats">
            {#if c.hp != null}<span class="stat t-mono-lg hp">{c.hp}<span class="stat-label">hp</span></span>{/if}
            {#if c.ac != null}<span class="stat t-mono-lg ac">{c.ac}<span class="stat-label">ac</span></span>{/if}
            {#if c.initiative != null}<span class="stat t-mono-lg init">{c.initiative >= 0 ? "+" : ""}{c.initiative}<span class="stat-label">init</span></span>{/if}
            {#if c.power != null}<span class="stat t-mono-lg pwr">{c.power}<span class="stat-label">pwr</span></span>{/if}
          </div>
          {#if c.tags.length > 0}
            <div class="tags">
              {#each c.tags.slice(0, 3) as t (t)}
                <span class="tag t-mono-sm">{t}</span>
              {/each}
            </div>
          {/if}
        </button>
        <div class="card-actions">
          <div class="add-wrap">
            <button
              class="act act-primary t-label-lg"
              type="button"
              onclick={() => (openAddFor = openAddFor === c.relPath ? null : c.relPath)}
            >
              + Encounter
            </button>
            {#if openAddFor === c.relPath}
              <div class="add-menu">
                {#if catalog.encounters.length === 0}
                  <div class="add-hint t-body-sm">No encounters yet.</div>
                {/if}
                {#each catalog.encounters as enc (enc.relPath)}
                  <button
                    class="add-item t-body-md"
                    type="button"
                    onclick={() => addCreatureToEncounter(c, enc.relPath)}
                  >
                    {enc.title}
                    <span class="add-count t-mono-sm">{enc.combatants}</span>
                  </button>
                {/each}
                <button
                  class="add-item new t-body-md"
                  type="button"
                  onclick={() => addCreatureToEncounter(c, null)}
                >
                  + New encounter…
                </button>
              </div>
            {/if}
          </div>
          <button class="act t-label-lg" type="button" title="Duplicate" onclick={() => copy(c)}>Copy</button>
          <button class="act act-danger t-label-lg" type="button" title="Delete" onclick={() => remove(c)}>×</button>
        </div>
      </article>
    {/each}
  </div>

  {#if dragActive}
    <div class="drop-overlay">
      <span class="drop-label t-headline-sm">Drop to import statblock</span>
    </div>
  {/if}

  {#if importPreviewData}
    <ImportDialog preview={importPreviewData} onClose={() => (importPreviewData = null)} />
  {/if}
</div>

<style>
  .creatures {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: var(--gutter-sm);
    background: var(--surface-base);
    padding: var(--inset-panel) var(--gutter-xl);
    overflow-y: auto;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--gutter-sm);
    flex: none;
  }

  .pills {
    display: flex;
    gap: var(--gutter-xs);
  }

  .pill {
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-full);
    color: var(--text-secondary);
    padding: 3px 12px;
    cursor: pointer;
    transition: all 100ms ease;
  }

  .pill:hover {
    color: var(--text-primary);
    border-color: var(--border-focus);
  }

  .pill.active {
    color: var(--surface-canvas);
    background: var(--amber);
    border-color: var(--amber);
  }

  .search {
    flex: 1;
    min-width: 8rem;
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded);
    color: var(--text-primary);
    padding: 5px 10px;
    outline: none;
  }

  .search:focus {
    border-color: var(--amber);
  }

  .search::placeholder {
    color: var(--text-tertiary);
  }

  .new-form {
    display: flex;
    gap: var(--gutter-xs);
    align-items: center;
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded);
    padding: var(--gutter-xs) var(--gutter-sm);
    flex: none;
  }

  .field {
    background: var(--surface-base);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    color: var(--text-primary);
    padding: 4px 8px;
    outline: none;
  }

  .field:focus {
    border-color: var(--amber);
  }

  .field.name {
    flex: 1;
    min-width: 8rem;
  }

  .field.num {
    width: 4.5rem;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: var(--gutter-md);
    align-content: start;
  }

  .card {
    display: flex;
    flex-direction: column;
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded);
    box-shadow:
      inset 0 1px 0 var(--border-bevel-light),
      0 1px 0 var(--border-bevel-dark);
    overflow: visible;
    transition: border-color 120ms ease, box-shadow 120ms ease;
  }

  .card:hover {
    border-color: var(--border-focus);
    box-shadow:
      inset 0 1px 0 var(--border-bevel-light),
      0 1px 0 var(--border-bevel-dark),
      var(--glow-amber);
  }

  .card-main {
    display: flex;
    flex-direction: column;
    gap: var(--gutter-sm);
    background: transparent;
    border: none;
    padding: var(--gutter-md);
    cursor: pointer;
    text-align: left;
    border-radius: var(--rounded);
  }

  .card-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--gutter-xs);
  }

  .name {
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .kind {
    flex: none;
    font-size: 9px;
    text-transform: uppercase;
    border-radius: var(--rounded-sm);
    padding: 1px 6px;
    border: 1px solid;
  }

  .kind-monster {
    color: var(--crimson-bright);
    border-color: rgba(225, 29, 72, 0.35);
  }

  .kind-npc {
    color: var(--amber);
    border-color: var(--border-focus);
  }

  .kind-player {
    color: var(--emerald);
    border-color: rgba(16, 185, 129, 0.35);
  }

  .stats {
    display: flex;
    gap: var(--gutter-sm);
  }

  .stat {
    display: flex;
    align-items: baseline;
    gap: 3px;
  }

  .stat-label {
    font-size: 9px;
    color: var(--text-tertiary);
    text-transform: uppercase;
  }

  .hp {
    color: var(--emerald);
  }

  .ac {
    color: var(--sapphire);
  }

  .init {
    color: var(--amber);
  }

  .pwr {
    color: var(--text-secondary);
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--gutter-xs);
  }

  .tag {
    color: var(--text-tertiary);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: 0 5px;
    font-size: 9px;
  }

  .card-actions {
    display: flex;
    gap: var(--gutter-xs);
    padding: 0 var(--gutter-sm) var(--gutter-sm);
    border-top: 1px solid var(--border-static);
    padding-top: var(--gutter-xs);
  }

  .act {
    background: transparent;
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    color: var(--text-secondary);
    padding: 2px 8px;
    cursor: pointer;
    transition: all 100ms ease;
  }

  .act:hover {
    color: var(--text-primary);
    border-color: var(--border-focus);
  }

  .act-primary {
    color: var(--amber);
    border-color: var(--border-focus);
  }

  .act-primary:hover {
    box-shadow: var(--glow-amber);
  }

  .act-danger:hover {
    color: var(--crimson-bright);
    border-color: var(--crimson);
  }

  .add-wrap {
    position: relative;
    margin-right: auto;
  }

  .add-menu {
    position: absolute;
    z-index: 40;
    top: calc(100% + 4px);
    left: 0;
    min-width: 14rem;
    max-height: 16rem;
    overflow-y: auto;
    background: var(--surface-overlay);
    border: 1px solid var(--border-focus);
    border-radius: var(--rounded);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    padding: var(--gutter-xs);
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .add-hint {
    color: var(--text-tertiary);
    padding: 4px 8px;
  }

  .add-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--gutter-sm);
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-radius: var(--rounded-sm);
    padding: 4px 8px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .add-item:hover {
    background: var(--surface-highlight);
    color: var(--text-primary);
  }

  .add-item.new {
    color: var(--amber);
    border-top: 1px solid var(--border-static);
    margin-top: 2px;
    border-radius: 0;
  }

  .add-count {
    color: var(--text-tertiary);
  }

  .empty {
    grid-column: 1 / -1;
    color: var(--text-tertiary);
    text-align: center;
    padding: var(--gutter-xl);
  }

  .drop-overlay {
    position: fixed;
    inset: 0;
    z-index: 90;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(13, 15, 18, 0.75);
    border: 2px dashed var(--amber);
    box-shadow: var(--glow-amber);
  }

  .drop-label {
    color: var(--amber);
    text-shadow: 0 0 12px rgba(224, 169, 83, 0.45);
  }
</style>
