<script lang="ts">
  import { showHelp } from "../stores/ui.svelte";

  const SHORTCUTS: [string, string][] = [
    ["Ctrl+K", "Global search palette"],
    ["Ctrl+N / Ctrl+P", "Move palette selection (or ↑ ↓)"],
    ["Enter", "Open selection / apply edit"],
    ["Space", "Advance turn (encounter view)"],
    ["J / K", "Select next / previous combatant"],
    ["?", "Toggle this help"],
    ["Esc", "Close any overlay"]
  ];
</script>

{#if showHelp.open}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="overlay" onclick={(e) => e.target === e.currentTarget && (showHelp.open = false)}>
    <div class="panel" role="dialog" aria-label="Keyboard shortcuts">
      <header class="head">
        <h2 class="t-headline-sm">Keyboard Shortcuts</h2>
        <button class="close t-mono-sm" type="button" onclick={() => (showHelp.open = false)}>×</button>
      </header>
      <ul class="list">
        {#each SHORTCUTS as [key, desc] (key)}
          <li>
            <span class="key t-mono-sm">{key}</span>
            <span class="desc t-body-md">{desc}</span>
          </li>
        {/each}
      </ul>
      <footer class="foot t-mono-sm">Esc closes</footer>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(8, 10, 13, 0.6);
    z-index: 105;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .panel {
    width: 24rem;
    max-width: 92vw;
    background: var(--surface-overlay);
    border: 1px solid var(--border-focus);
    border-radius: var(--rounded-lg);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--gutter-md) var(--gutter-lg);
    border-bottom: 1px solid var(--border-static);
  }

  .head h2 {
    margin: 0;
    color: var(--text-primary);
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

  .list {
    list-style: none;
    margin: 0;
    padding: var(--gutter-md) var(--gutter-lg);
    display: flex;
    flex-direction: column;
    gap: var(--gutter-sm);
  }

  .list li {
    display: flex;
    align-items: baseline;
    gap: var(--gutter-md);
  }

  .key {
    flex: none;
    min-width: 7rem;
    text-align: center;
    color: var(--amber);
    background: var(--surface-base);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: 2px 6px;
  }

  .desc {
    color: var(--text-secondary);
  }

  .foot {
    padding: var(--gutter-xs) var(--gutter-lg) var(--gutter-md);
    color: var(--text-tertiary);
  }
</style>
