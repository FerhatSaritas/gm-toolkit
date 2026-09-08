<script lang="ts">
  import type { SearchHit } from "../commands";
  import {
    chooseSelected,
    closePalette,
    moveSelection,
    palette,
    setQuery
  } from "../stores/search.svelte";

  let inputEl: HTMLInputElement | undefined = $state();

  const KIND_LABEL: Record<SearchHit["kind"], string> = {
    scene: "Scene",
    encounter: "Encounter",
    statblock: "Creature",
    item: "Item"
  };

  interface Group {
    kind: SearchHit["kind"];
    label: string;
    hits: SearchHit[];
  }

  const KIND_ORDER: SearchHit["kind"][] = ["scene", "encounter", "statblock", "item"];

  const groups = $derived.by(() => {
    const out: Group[] = [];
    for (const kind of KIND_ORDER) {
      const hits = palette.results.filter((r) => r.kind === kind);
      if (hits.length > 0) {
        out.push({ kind, label: KIND_LABEL[kind], hits });
      }
    }
    return out;
  });

  const flat = $derived(groups.flatMap((g) => g.hits));

  const selectedTitle = $derived(flat[palette.selected]?.title ?? "");

  // Keep the selected row visible.
  $effect(() => {
    const title = selectedTitle;
    if (!title) return;
    const el = document.querySelector(`[data-hit-title="${CSS.escape(title)}"]`);
    el?.scrollIntoView({ block: "nearest" });
  });

  $effect(() => {
    if (palette.open) {
      inputEl?.focus();
    }
  });

  function kindClass(kind: SearchHit["kind"]): string {
    return `kind-${kind}`;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="overlay" onclick={(e) => e.target === e.currentTarget && closePalette()}>
  <div class="panel" role="dialog" aria-label="Global search">
    <input
      bind:this={inputEl}
      class="query t-mono-lg"
      type="text"
      placeholder="Search scenes, encounters, creatures, items…"
      spellcheck="false"
      value={palette.query}
      oninput={(e) => setQuery(e.currentTarget.value)}
      onkeydown={(e) => {
        if (e.key === "ArrowDown" || (e.key === "n" && e.ctrlKey)) {
          e.preventDefault();
          moveSelection(1);
        } else if (e.key === "ArrowUp" || (e.key === "p" && e.ctrlKey)) {
          e.preventDefault();
          moveSelection(-1);
        } else if (e.key === "Enter") {
          e.preventDefault();
          chooseSelected();
        } else if (e.key === "Escape") {
          e.preventDefault();
          closePalette();
        }
      }}
    />

    <div class="results">
      {#if !palette.query.trim()}
        <p class="hint t-body-sm">
          Type to search titles, tags, creature names and body text.
        </p>
      {:else if palette.results.length === 0}
        <p class="hint t-body-sm">No matches.</p>
      {:else}
        {#each groups as group (group.kind)}
          <div class="group-label t-label-lg">{group.label}</div>
          {#each group.hits as hit (hit.relPath)}
            {@const idx = flat.indexOf(hit)}
            <button
              class="hit"
              type="button"
              data-hit-title={hit.title}
              class:selected={idx === palette.selected}
              onmouseenter={() => (palette.selected = idx)}
              onclick={chooseSelected}
            >
              <span class="kind kind-{kindClass(hit.kind)} t-mono-sm">{group.label}</span>
              <span class="hit-main">
                <span class="hit-title t-body-lg">{hit.title}</span>
                {#if hit.snippet}
                  <span class="hit-snippet t-mono-sm">{hit.snippet}</span>
                {/if}
              </span>
              <span class="hit-path t-mono-sm">{hit.relPath}</span>
            </button>
          {/each}
        {/each}
      {/if}
    </div>

    <footer class="foot t-mono-sm">
      <span>↑↓ navigate</span>
      <span>↵ open</span>
      <span>esc close</span>
    </footer>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(8, 10, 13, 0.6);
    z-index: 110;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 8vh;
  }

  .panel {
    width: 38rem;
    max-width: 92vw;
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    background: var(--surface-overlay);
    border: 1px solid var(--border-focus);
    border-radius: var(--rounded-lg);
    box-shadow:
      0 24px 60px rgba(0, 0, 0, 0.65),
      var(--glow-amber);
    overflow: hidden;
  }

  .query {
    flex: none;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border-static);
    color: var(--text-primary);
    padding: 10px 14px;
    outline: none;
  }

  .query::placeholder {
    color: var(--text-tertiary);
  }

  .results {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--gutter-xs);
  }

  .hint {
    color: var(--text-tertiary);
    text-align: center;
    padding: var(--gutter-lg);
  }

  .group-label {
    color: var(--text-tertiary);
    text-transform: uppercase;
    font-size: 9px;
    letter-spacing: 0.08em;
    padding: var(--gutter-xs) var(--gutter-sm) 2px;
  }

  .hit {
    display: grid;
    grid-template-columns: 4.5rem 1fr auto;
    align-items: baseline;
    gap: var(--gutter-sm);
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-radius: var(--rounded-sm);
    padding: 4px 8px;
    cursor: pointer;
  }

  .hit.selected {
    background: var(--surface-highlight);
    box-shadow: inset 2px 0 0 var(--amber);
  }

  .kind {
    font-size: 9px;
    text-transform: uppercase;
    text-align: center;
    border: 1px solid;
    border-radius: var(--rounded-sm);
    padding: 1px 0;
  }

  .kind.kind-scene {
    color: var(--amber);
    border-color: var(--border-focus);
  }

  .kind.kind-encounter {
    color: var(--crimson-bright);
    border-color: rgba(225, 29, 72, 0.4);
  }

  .kind.kind-statblock {
    color: var(--sapphire);
    border-color: rgba(56, 189, 248, 0.35);
  }

  .kind.kind-item {
    color: var(--emerald);
    border-color: rgba(16, 185, 129, 0.35);
  }

  .hit-main {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .hit-title {
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .hit-snippet {
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .hit-path {
    color: var(--text-tertiary);
    max-width: 10rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .foot {
    flex: none;
    display: flex;
    gap: var(--gutter-md);
    padding: 4px 14px;
    border-top: 1px solid var(--border-static);
    color: var(--text-tertiary);
  }
</style>
