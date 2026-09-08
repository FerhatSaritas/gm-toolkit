<script lang="ts">
  import type { Doc } from "../commands";
  import type { CombatantState } from "../commands";
  import CombatantRow from "./CombatantRow.svelte";
  import {
    addCombatant,
    advanceTurn,
    adjustHp,
    encounter,
    moveSelection,
    removeCombatant,
    resetEncounter,
    rewindTurn,
    rollInitiative,
    setHp,
    setInitiative,
    setTempHp,
    setTurn,
    startEncounter,
    toggleCondition
  } from "../stores/encounter.svelte";
  import { showStatblock } from "../stores/inspector.svelte";
  import { palette } from "../stores/search.svelte";
  import { catalog, refreshCatalog } from "../stores/catalog.svelte";

  let { doc }: { doc: Doc } = $props();

  let showAdd = $state(false);
  let addName = $state("");
  let addKind = $state<CombatantState["kind"]>("monster");
  let addHp = $state("");
  let addAc = $state("");
  let partyOpen = $state(false);

  const party = $derived(catalog.creatures.filter((c) => c.kind === "player"));

  $effect(() => {
    if (mine && catalog.creatures.length === 0) void refreshCatalog();
  });

  const encState = $derived(encounter.state);
  const mine = $derived(
    encState && encState.encounterPath === doc.relPath ? encState : null
  );
  const rows = $derived(mine ? [...mine.combatants].sort(
    (a, b) =>
      (b.initiative ?? Number.NEGATIVE_INFINITY) -
        (a.initiative ?? Number.NEGATIVE_INFINITY) ||
      (b.initiativeMod ?? 0) - (a.initiativeMod ?? 0) ||
      a.name.localeCompare(b.name)
  ) : []);
  const activeName = $derived(
    mine?.combatants.find((c) => c.id === mine.turnId)?.name ?? null
  );

  function submitAdd() {
    if (!addName.trim()) return;
    const hp = Number(addHp);
    const ac = Number(addAc);
    addCombatant(
      addName,
      addKind,
      addHp.trim() && !Number.isNaN(hp) ? hp : null,
      addAc.trim() && !Number.isNaN(ac) ? ac : null
    );
    addName = "";
    addHp = "";
    addAc = "";
    showAdd = false;
  }

  function addFromParty(c: (typeof catalog.creatures)[number]) {
    partyOpen = false;
    addCombatant(c.name, "player", c.hp ?? null, c.ac ?? null, c.relPath);
  }

  function onKey(e: KeyboardEvent) {
    if (!mine || palette.open) return;
    const t = e.target as HTMLElement;
    if (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || t.isContentEditable) {
      return;
    }
    switch (e.key) {
      case " ":
        e.preventDefault();
        advanceTurn();
        break;
      case "j":
      case "ArrowDown":
        e.preventDefault();
        moveSelection(1);
        break;
      case "k":
      case "ArrowUp":
        e.preventDefault();
        moveSelection(-1);
        break;
      case "Enter":
        if (encounter.selectedId) setTurn(encounter.selectedId);
        break;
    }
  }
</script>

<svelte:window onkeydown={onKey} />

{#if !mine}
  <div class="not-started">
    <p class="t-body-lg ns-title">Not in combat.</p>
    <p class="t-body-sm ns-sub">
      Live state (initiative, HP, conditions) is kept in
      <code>.state/</code> — your blueprint stays untouched.
    </p>
    <button class="btn btn-primary t-label-lg" type="button" onclick={() => startEncounter(doc.relPath)}>
      Start Encounter
    </button>
  </div>
{:else}
  <div class="tracker">
    <div class="toolbar">
      <span class="round-pill t-mono-lg" title="Round">R{mine.round}</span>
      <span class="turn-label t-label-lg">
        {activeName ? `${activeName}'s turn` : "Roll initiative to begin"}
      </span>
      <div class="toolbar-actions">
        <button class="btn btn-secondary t-label-lg" type="button" title="Previous turn" onclick={rewindTurn}>
          ‹
        </button>
        <button class="btn btn-secondary t-label-lg" type="button" title="Next turn (Space)" onclick={advanceTurn}>
          ›
        </button>
        <button class="btn btn-primary t-label-lg" type="button" onclick={rollInitiative}>
          Roll Initiative
        </button>
        <button class="btn btn-secondary t-label-lg" type="button" onclick={() => (showAdd = !showAdd)}>
          + Add
        </button>
        <div class="party-wrap">
          <button
            class="btn btn-secondary t-label-lg"
            type="button"
            title="Add players from party/"
            onclick={() => {
              partyOpen = !partyOpen;
              if (partyOpen && catalog.creatures.length === 0) void refreshCatalog();
            }}
          >
            + Party
          </button>
          {#if partyOpen}
            <div class="party-menu">
              {#if party.length === 0}
                <div class="party-empty t-body-sm">No player statblocks in <code>party/</code> yet.</div>
              {/if}
              {#each party as p (p.relPath)}
                <button class="party-item t-body-md" type="button" onclick={() => addFromParty(p)}>
                  <span class="party-name">{p.name}</span>
                  {#if p.hp != null}<span class="party-hp t-mono-sm">{p.hp} hp</span>{/if}
                  {#if p.ac != null}<span class="party-ac t-mono-sm">ac {p.ac}</span>{/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>
        <button class="btn btn-danger t-label-lg" type="button" onclick={resetEncounter}>
          Reset
        </button>
      </div>
    </div>

    {#if showAdd}
      <div class="add-form">
        <input class="t-body-md field name" type="text" placeholder="Name" bind:value={addName} onkeydown={(e) => e.key === "Enter" && submitAdd()} />
        <select class="t-body-md field" bind:value={addKind}>
          <option value="monster">Monster</option>
          <option value="npc">NPC</option>
          <option value="player">Player</option>
        </select>
        <input class="t-body-md field num" type="text" inputmode="numeric" placeholder="HP" bind:value={addHp} onkeydown={(e) => e.key === "Enter" && submitAdd()} />
        <input class="t-body-md field num" type="text" inputmode="numeric" placeholder="AC" bind:value={addAc} onkeydown={(e) => e.key === "Enter" && submitAdd()} />
        <button class="btn btn-primary t-label-lg" type="button" onclick={submitAdd}>Add</button>
      </div>
    {/if}

    <div class="rows">
      {#if rows.length === 0}
        <p class="t-body-md empty-rows">No combatants. Add them to the blueprint or use “+ Add”.</p>
      {/if}
      {#each rows as c, i (c.id)}
        <CombatantRow
          combatant={c}
          active={mine.turnId === c.id}
          selected={encounter.selectedId === c.id}
          onAdjust={adjustHp}
          onSetHp={setHp}
          onSetTempHp={setTempHp}
          onSetInitiative={setInitiative}
          onToggleCondition={toggleCondition}
          onSelect={(id) => (encounter.selectedId = id)}
          onSetTurn={setTurn}
          onRemove={removeCombatant}
          onOpenStatblock={(id) => {
            const c = mine.combatants.find((x) => x.id === id);
            if (c) showStatblock(mine.encounterPath, c.statblock ?? c.name);
          }}
        />
        {#if i === 0 && rows.length > 1 && rows[1].initiative == null && c.initiative != null}
          <div class="no-init-divider t-mono-sm">no initiative</div>
        {/if}
      {/each}
    </div>
  </div>
{/if}

<style>
  .not-started {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--gutter-sm);
    text-align: center;
  }

  .ns-title {
    color: var(--text-primary);
    margin: 0;
  }

  .ns-sub {
    color: var(--text-tertiary);
    margin: 0 0 var(--gutter-md);
    max-width: 26rem;
  }

  .ns-sub code {
    font-family: var(--font-mono);
    color: var(--sapphire);
  }

  .tracker {
    display: flex;
    flex-direction: column;
    gap: var(--gutter-sm);
    min-height: 0;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--gutter-md);
  }

  .round-pill {
    color: var(--crimson-bright);
    background: var(--surface-raised);
    border: 1px solid rgba(225, 29, 72, 0.35);
    border-radius: var(--rounded);
    padding: 3px 10px;
  }

  .turn-label {
    color: var(--text-secondary);
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .toolbar-actions {
    display: flex;
    gap: var(--gutter-xs);
    flex: none;
  }

  .toolbar-actions .btn {
    padding: 4px 10px;
  }

  .party-wrap {
    position: relative;
  }

  .party-menu {
    position: absolute;
    z-index: 30;
    top: calc(100% + 4px);
    right: 0;
    min-width: 13rem;
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

  .party-empty {
    color: var(--text-tertiary);
    padding: 4px 8px;
  }

  .party-item {
    display: flex;
    align-items: baseline;
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

  .party-item:hover {
    background: var(--surface-highlight);
    color: var(--text-primary);
  }

  .party-name {
    flex: 1;
  }

  .party-hp {
    color: var(--emerald);
  }

  .party-ac {
    color: var(--sapphire);
  }

  .party-wrap {
    position: relative;
  }

  .party-menu {
    position: absolute;
    z-index: 30;
    top: calc(100% + 4px);
    right: 0;
    min-width: 13rem;
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

  .party-empty {
    color: var(--text-tertiary);
    padding: 4px 8px;
  }

  .party-item {
    display: flex;
    align-items: baseline;
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

  .party-item:hover {
    background: var(--surface-highlight);
    color: var(--text-primary);
  }

  .party-name {
    flex: 1;
  }

  .party-hp {
    color: var(--emerald);
  }

  .party-ac {
    color: var(--sapphire);
  }

  .add-form {
    display: flex;
    gap: var(--gutter-xs);
    align-items: center;
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded);
    padding: var(--gutter-xs) var(--gutter-sm);
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

  .rows {
    display: flex;
    flex-direction: column;
    gap: var(--gutter-xs);
    overflow-y: auto;
    min-height: 0;
    padding-bottom: var(--gutter-sm);
  }

  .no-init-divider {
    color: var(--text-tertiary);
    text-transform: uppercase;
    font-size: 9px;
    text-align: center;
    padding: 2px 0;
    letter-spacing: 0.08em;
    border-top: 1px dashed var(--border-static);
  }

  .empty-rows {
    color: var(--text-tertiary);
    text-align: center;
    padding: var(--gutter-lg);
  }
</style>
