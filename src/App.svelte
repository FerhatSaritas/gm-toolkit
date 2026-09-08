<script lang="ts">
  import { onMount } from "svelte";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import Stage from "./lib/components/Stage.svelte";
  import CreaturesView from "./lib/components/CreaturesView.svelte";
  import GeneratorsView from "./lib/components/GeneratorsView.svelte";
  import Inspector from "./lib/components/Inspector.svelte";
  import CommandPalette from "./lib/components/CommandPalette.svelte";
  import ShortcutHelp from "./lib/components/ShortcutHelp.svelte";
  import { campaign, initCampaign, toast } from "./lib/stores/campaign.svelte";
  import { palette, togglePalette } from "./lib/stores/search.svelte";
  import { ui, toggleHelp } from "./lib/stores/ui.svelte";

  onMount(() => {
    void initCampaign();
  });

  function globalKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      togglePalette();
      return;
    }
    const t = e.target as HTMLElement;
    if (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || t.isContentEditable || t instanceof HTMLSelectElement) {
      return;
    }
    if (e.key === "?" && !e.ctrlKey && !e.metaKey) {
      e.preventDefault();
      toggleHelp();
    }
  }
</script>

<svelte:window onkeydown={globalKey} />

<div class="shell">
  <TitleBar />
  <div class="workspace">
    <Sidebar />
    {#if ui.view === "creatures"}
      <CreaturesView />
    {:else if ui.view === "generators"}
      <GeneratorsView />
    {:else}
      <Stage />
    {/if}
    <Inspector />
  </div>

  {#if toast.message}
    <div class="toast t-body-md" class:toast-error={toast.kind === "error"}>
      {toast.message}
    </div>
  {/if}

  {#if campaign.loading}
    <div class="loading t-mono-sm">…</div>
  {/if}

  {#if palette.open}
    <CommandPalette />
  {/if}

  <ShortcutHelp />
</div>

<style>
  .shell {
    height: 100vh;
    display: grid;
    grid-template-rows: var(--titlebar-h) 1fr;
    background: var(--surface-canvas);
    overflow: hidden;
  }

  .workspace {
    display: grid;
    grid-template-columns: var(--sidebar-w) 1fr var(--statblock-w);
    min-height: 0;
  }

  .toast {
    position: fixed;
    left: 50%;
    bottom: 1.25rem;
    transform: translateX(-50%);
    background: var(--surface-overlay);
    color: var(--text-primary);
    border: 1px solid var(--border-focus);
    border-radius: var(--rounded);
    padding: 6px 14px;
    box-shadow: var(--glow-amber);
    z-index: 50;
    max-width: 60vw;
  }

  .toast-error {
    border-color: var(--crimson);
    color: var(--crimson-bright);
  }

  .loading {
    position: fixed;
    top: calc(var(--titlebar-h) + 6px);
    right: 10px;
    color: var(--text-tertiary);
    z-index: 50;
  }
</style>
