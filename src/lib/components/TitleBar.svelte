<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { campaign, notify } from "../stores/campaign.svelte";
  import { breakdown, dice, roll } from "../stores/dice.svelte";

  let rollExpr = $state("");
  let flash = $state<string | null>(null);
  let logOpen = $state(false);
  let flashTimer: ReturnType<typeof setTimeout> | undefined;
  const win = getCurrentWindow();

  async function submitRoll() {
    const expr = rollExpr.trim();
    if (!expr) return;
    try {
      const r = await roll(expr);
      rollExpr = "";
      flash = `${r.notation} = ${r.total}`;
      if (flashTimer) clearTimeout(flashTimer);
      flashTimer = setTimeout(() => (flash = null), 2600);
    } catch (e) {
      notify(String(e), "error");
    }
  }
</script>

<header class="titlebar">
  <div class="tb-left" data-tauri-drag-region>
    <span class="brand t-mono-sm" data-tauri-drag-region>GM</span>
    <span class="crumb t-label-lg" data-tauri-drag-region>
      {campaign.info ? campaign.info.name : "No Campaign"}
    </span>
  </div>

  <div class="tb-center" data-tauri-drag-region>
    <input
      class="roll-input t-mono-sm"
      type="text"
      placeholder="/roll 2d6+4"
      spellcheck="false"
      bind:value={rollExpr}
      onkeydown={(e) => {
        if (e.key === "Enter") submitRoll();
      }}
    />
    {#if flash}
      <span class="roll-flash t-mono-sm">{flash}</span>
    {/if}
    {#if dice.history.length > 0}
      <button
        class="dice-log-btn t-mono-sm"
        type="button"
        title="Dice log"
        onclick={() => (logOpen = !logOpen)}
      >
        log {dice.history.length}
      </button>
      {#if logOpen}
        <div class="dice-log">
          {#each dice.history as r, i (r.at + "-" + i)}
            <div class="log-row" class:first={i === 0}>
              <span class="log-total t-mono-lg">{r.total}</span>
              <div class="log-detail">
                <span class="log-notation t-mono-sm">{r.notation}</span>
                <span class="log-breakdown t-mono-sm">{breakdown(r)}</span>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    {/if}
  </div>

  <div class="tb-actions">
    <button
      class="win-btn t-mono-sm"
      type="button"
      title="Minimize"
      onclick={() => win.minimize()}
    >
      &#x2500;
    </button>
    <button
      class="win-btn t-mono-sm"
      type="button"
      title="Maximize"
      onclick={() => win.toggleMaximize()}
    >
      &#x25A1;
    </button>
    <button
      class="win-btn win-close t-mono-sm"
      type="button"
      title="Close"
      onclick={() => win.close()}
    >
      &#x00D7;
    </button>
  </div>
</header>

<style>
  .titlebar {
    height: var(--titlebar-h);
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    background: var(--surface-canvas);
    border-bottom: 1px solid var(--border-static);
    box-shadow:
      inset 0 1px 0 var(--border-bevel-light),
      0 1px 0 var(--border-bevel-dark);
    user-select: none;
  }

  .tb-left {
    display: flex;
    align-items: center;
    gap: var(--gutter-sm);
    padding-left: var(--gutter-md);
    min-width: 0;
  }

  .brand {
    color: var(--surface-canvas);
    background: var(--amber);
    border-radius: var(--rounded-sm);
    padding: 1px 5px;
    box-shadow: var(--glow-amber);
    letter-spacing: 0.06em;
  }

  .crumb {
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tb-center {
    display: flex;
    justify-content: center;
    align-items: center;
    position: relative;
  }

  .roll-input {
    width: 15rem;
    text-align: center;
    color: var(--text-primary);
    background: var(--surface-base);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded);
    padding: 2px 8px;
    outline: none;
    transition: border-color 120ms ease, box-shadow 120ms ease;
  }

  .roll-input::placeholder {
    color: var(--text-tertiary);
  }

  .roll-input:focus {
    border-color: var(--amber);
    box-shadow: 0 0 0 1px var(--border-focus);
  }

  .roll-flash {
    position: absolute;
    top: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
    color: var(--amber);
    background: var(--surface-overlay);
    border: 1px solid var(--border-focus);
    border-radius: var(--rounded);
    padding: 3px 10px;
    box-shadow: var(--glow-amber);
    white-space: nowrap;
    z-index: 60;
  }

  .dice-log-btn {
    margin-left: var(--gutter-xs);
    color: var(--text-tertiary);
    background: transparent;
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: 2px 7px;
    cursor: pointer;
    transition: all 100ms ease;
  }

  .dice-log-btn:hover {
    color: var(--amber);
    border-color: var(--border-focus);
  }

  .dice-log {
    position: absolute;
    top: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
    width: 20rem;
    max-height: 40vh;
    overflow-y: auto;
    background: var(--surface-overlay);
    border: 1px solid var(--border-focus);
    border-radius: var(--rounded);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.55);
    padding: var(--gutter-xs);
    z-index: 60;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .log-row {
    display: flex;
    align-items: center;
    gap: var(--gutter-sm);
    padding: 3px 6px;
    border-radius: var(--rounded-sm);
  }

  .log-row:hover {
    background: var(--surface-highlight);
  }

  .log-row.first {
    background: rgba(224, 169, 83, 0.08);
    border: 1px solid var(--border-focus);
  }

  .log-total {
    color: var(--amber);
    width: 3rem;
    text-align: right;
    flex: none;
  }

  .log-detail {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .log-notation {
    color: var(--text-primary);
  }

  .log-breakdown {
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tb-actions {
    display: flex;
    justify-content: flex-end;
    align-self: stretch;
  }

  .win-btn {
    width: 2.75rem;
    align-self: stretch;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: default;
    transition: background 100ms ease, color 100ms ease;
  }

  .win-btn:hover {
    background: var(--surface-highlight);
    color: var(--text-secondary);
  }

  .win-close:hover {
    background: var(--crimson);
    color: #ffffff;
  }
</style>
