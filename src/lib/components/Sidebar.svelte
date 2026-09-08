<script lang="ts">
  import { campaign, openCampaign } from "../stores/campaign.svelte";
  import { showBestiary, showForge, showGallery, showLibrary, showHelp, ui } from "../stores/ui.svelte";
  import CampaignTree from "./CampaignTree.svelte";
</script>

<aside class="rail sidebar">
  <div class="rail-header nav-header">
    <div class="nav-tabs">
      <button
        class="nav-tab t-label-lg"
        type="button"
        class:active={ui.view === "library"}
        onclick={showLibrary}
        title="File library"
      >
        Library
      </button>
      <button
        class="nav-tab t-label-lg"
        type="button"
        class:active={ui.view === "creatures" && ui.creatureFilter === "all"}
        onclick={showBestiary}
        title="All creatures"
      >
        Bestiary
      </button>
      <button
        class="nav-tab t-label-lg"
        type="button"
        class:active={ui.view === "creatures" && ui.creatureFilter === "npc"}
        onclick={showGallery}
        title="NPC gallery"
      >
        NPCs
      </button>
      <button
        class="nav-tab t-label-lg"
        type="button"
        class:active={ui.view === "generators"}
        onclick={showForge}
        title="Generators"
      >
        Forge
      </button>
    </div>
  </div>

  {#if campaign.info}
    <div class="side-campaign">
      <span class="t-mono-sm count-pill" title="Markdown files">
        {campaign.info.counts.total}&nbsp;files
      </span>
      <span class="t-mono-sm count-pill" title="Scenes">
        {campaign.info.counts.scenes}&nbsp;scenes
      </span>
      <span class="t-mono-sm count-pill" title="Encounters">
        {campaign.info.counts.encounters}&nbsp;encounters
      </span>
    </div>
    <div class="tree-scroll">
      {#if campaign.tree.length > 0}
        <CampaignTree nodes={campaign.tree} />
      {:else}
        <p class="side-hint t-body-sm">
          No markdown found yet. Add files to
          <code>scenes/</code>, <code>encounters/</code>,
          <code>npcs/</code> or <code>items/</code>.
        </p>
      {/if}
    </div>
  {:else}
    <div class="side-empty">
      <p class="t-body-md side-empty-text">No campaign open.</p>
      <button class="btn btn-primary t-label-lg" type="button" onclick={openCampaign}>
        Open Campaign…
      </button>
    </div>
  {/if}
  <div class="rail-footer">
    <button class="help-btn t-mono-sm" type="button" title="Keyboard shortcuts" onclick={() => (showHelp.open = true)}>
      ? shortcuts
    </button>
  </div>
</aside>

<style>
  .nav-header {
    padding: var(--gutter-xs) var(--gutter-sm);
  }

  .nav-tabs {
    display: flex;
    gap: 2px;
  }

  .nav-tab {
    flex: 1;
    background: transparent;
    border: none;
    border-radius: var(--rounded-sm);
    color: var(--text-tertiary);
    padding: 4px 6px;
    cursor: pointer;
    text-align: center;
    transition: all 100ms ease;
  }

  .nav-tab:hover {
    color: var(--text-primary);
    background: var(--surface-highlight);
  }

  .nav-tab.active {
    color: var(--amber);
    background: var(--surface-overlay);
    box-shadow: inset 0 -2px 0 var(--amber);
  }

  .help-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 0;
  }

  .help-btn:hover {
    color: var(--amber);
  }

  .side-campaign {
    display: flex;
    flex-wrap: wrap;
    gap: var(--gutter-xs);
    padding: 0 var(--inset-panel) var(--gutter-sm);
  }

  .count-pill {
    color: var(--text-secondary);
    background: var(--surface-base);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: 2px 6px;
  }

  .tree-scroll {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
    padding-bottom: var(--gutter-sm);
  }

  .side-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--gutter-md);
    padding: var(--gutter-xl) var(--gutter-lg);
  }

  .side-empty-text {
    color: var(--text-secondary);
    text-align: center;
  }

  .side-hint {
    color: var(--text-tertiary);
    padding: var(--gutter-sm) var(--gutter-md);
  }

  .side-hint code {
    font-family: var(--font-mono);
    color: var(--sapphire);
    font-size: 11px;
  }
</style>
