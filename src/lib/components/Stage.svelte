<script lang="ts">
  import EmptyState from "./EmptyState.svelte";
  import SceneBody from "./SceneBody.svelte";
  import EncounterView from "./EncounterView.svelte";
  import { campaign } from "../stores/campaign.svelte";
  import { showStatblock } from "../stores/inspector.svelte";
  import type { Doc } from "../commands";

  const doc = $derived(campaign.doc);

  const KIND_LABEL: Record<Doc["kind"], string> = {
    scene: "Scene",
    encounter: "Encounter",
    statblock: "Statblock",
    item: "Item"
  };

  const headerTitle = $derived(
    doc?.scene?.title ?? doc?.encounter?.title ?? doc?.statblock?.name ?? doc?.item?.name ?? doc?.relPath ?? ""
  );

  function kindClass(kind: Doc["kind"]): string {
    return `kind-${kind}`;
  }

  const chips = $derived.by(() => {
    if (!doc) return [];
    const out: { label: string; value: string; mono?: boolean }[] = [];
    if (doc.scene) {
      const s = doc.scene;
      if (s.status) out.push({ label: "status", value: s.status });
      if (s.readAloud) out.push({ label: "mode", value: "read-aloud" });
      for (const t of s.tags) out.push({ label: "tag", value: t });
      for (const n of s.npcs) out.push({ label: "npc", value: n });
    }
    if (doc.encounter) {
      const e = doc.encounter;
      if (e.partyLevel != null) out.push({ label: "party level", value: String(e.partyLevel), mono: true });
      for (const t of e.tags) out.push({ label: "tag", value: t });
    }
    if (doc.statblock) {
      const sb = doc.statblock;
      if (sb.hp != null) out.push({ label: "hp", value: String(sb.hp), mono: true });
      if (sb.ac != null) out.push({ label: "ac", value: String(sb.ac), mono: true });
      if (sb.initiative != null) out.push({ label: "init", value: `+${sb.initiative}`, mono: true });
      if (sb.speed) out.push({ label: "speed", value: sb.speed, mono: true });
      if (sb.power != null) out.push({ label: "power", value: String(sb.power), mono: true });
      for (const t of sb.tags) out.push({ label: "tag", value: t });
    }
    if (doc.item) {
      const it = doc.item;
      if (it.rarity) out.push({ label: "rarity", value: it.rarity });
      if (it.kind) out.push({ label: "kind", value: it.kind });
      if (it.value) out.push({ label: "value", value: it.value, mono: true });
      for (const t of it.tags) out.push({ label: "tag", value: t });
    }
    return out;
  });

  const combatants = $derived(doc?.encounter?.combatants ?? []);
</script>

<main class="stage">
  {#if !campaign.info}
    <EmptyState />
  {:else if doc}
    <article class="doc-view">
      <header class="doc-head">
        <div class="doc-title-row">
          <span class="kind-badge t-label-lg {kindClass(doc.kind)}">{KIND_LABEL[doc.kind]}</span>
          <h1 class="doc-title t-headline-lg">{headerTitle}</h1>
        </div>
        <span class="doc-path t-mono-sm">{doc.relPath}</span>
      </header>

      {#if chips.length > 0}
        <div class="chip-row">
          {#each chips as chip, i (i)}
            {#if chip.label === "npc"}
              <button
                class="chip chip-link t-mono-sm"
                type="button"
                title={`Open statblock: ${chip.value}`}
                onclick={() => showStatblock(doc.relPath, chip.value)}
              >
                <span class="chip-label">{chip.label}</span>
                {chip.value}
              </button>
            {:else}
              <span class="chip t-mono-sm" class:chip-mono={chip.mono}>
                <span class="chip-label">{chip.label}</span>
                {chip.value}
              </span>
            {/if}
          {/each}
        </div>
      {/if}

      {#if doc.kind === "encounter" && combatants.length > 0}
        <div class="combatant-strip">
          {#each combatants as c, i (i)}
            <button
              class="combatant t-mono-sm"
              type="button"
              title={`Open statblock: ${c.statblock ?? c.name}`}
              onclick={() => showStatblock(doc.relPath, c.statblock ?? c.name)}
            >
              <span class="c-name">{c.name}</span>
              <span class="c-kind">{c.kind}</span>
              {#if c.hp != null}<span class="c-hp">{c.hp} hp</span>{/if}
              {#if c.ac != null}<span class="c-ac">ac {c.ac}</span>{/if}
            </button>
          {/each}
        </div>
      {/if}

      <div class="divider-diamond doc-divider" role="separator"></div>
      <div class="body-scroll">
        {#if doc.kind === "encounter"}
          <EncounterView {doc} />
        {:else}
          <SceneBody {doc} headingToHide={headerTitle} />
        {/if}
      </div>
    </article>
  {:else}
    <div class="stage-hint">
      <p class="t-body-md">
        Select a scene, encounter or statblock from the library.
      </p>
      <p class="t-mono-sm stage-hint-sub">
        Structured scene &amp; encounter views land in M3–M4.
      </p>
    </div>
  {/if}
</main>

<style>
  .stage {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--surface-base);
  }

  .doc-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    padding: var(--inset-panel) var(--gutter-xl);
  }

  .doc-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--gutter-lg);
  }

  .doc-title-row {
    display: flex;
    align-items: center;
    gap: var(--gutter-sm);
    min-width: 0;
  }

  .kind-badge {
    flex: none;
    padding: 2px 8px;
    border-radius: var(--rounded-sm);
    border: 1px solid;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .kind-badge.kind-scene {
    color: var(--amber);
    border-color: var(--border-focus);
    background: rgba(224, 169, 83, 0.08);
  }

  .kind-badge.kind-encounter {
    color: var(--crimson-bright);
    border-color: rgba(225, 29, 72, 0.4);
    background: rgba(225, 29, 72, 0.08);
  }

  .kind-badge.kind-statblock {
    color: var(--sapphire);
    border-color: rgba(56, 189, 248, 0.35);
    background: rgba(56, 189, 248, 0.08);
  }

  .kind-badge.kind-item {
    color: var(--emerald);
    border-color: rgba(16, 185, 129, 0.35);
    background: rgba(16, 185, 129, 0.08);
  }

  .doc-title {
    color: var(--text-primary);
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .doc-path {
    color: var(--text-tertiary);
    flex: none;
  }

  .chip-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--gutter-xs);
    margin-top: var(--gutter-sm);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--text-secondary);
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: 2px 7px;
  }

  .chip-label {
    color: var(--text-tertiary);
    font-size: 9px;
    text-transform: uppercase;
  }

  .chip-mono {
    color: var(--text-primary);
  }

  .chip-link {
    cursor: pointer;
    transition: all 100ms ease;
  }

  .chip-link:hover {
    color: var(--amber);
    border-color: var(--border-focus);
    box-shadow: var(--glow-amber);
  }

  .combatant-strip {
    display: flex;
    flex-wrap: wrap;
    gap: var(--gutter-xs);
    margin-top: var(--gutter-sm);
  }

  .combatant {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-left: 2px solid var(--crimson);
    border-radius: var(--rounded-sm);
    padding: 3px 8px;
  }

  .c-name {
    color: var(--text-primary);
  }

  .c-kind {
    color: var(--text-tertiary);
    font-size: 9px;
    text-transform: uppercase;
  }

  .c-hp {
    color: var(--emerald);
  }

  .c-ac {
    color: var(--sapphire);
  }

  .doc-divider {
    margin: var(--gutter-md) 0;
  }

  .body-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .raw-body {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--text-primary);
    max-width: 60rem;
    user-select: text;
  }

  .stage-hint {
    margin: auto;
    text-align: center;
    color: var(--text-secondary);
  }

  .stage-hint-sub {
    color: var(--text-tertiary);
    margin-top: var(--gutter-xs);
  }
</style>
