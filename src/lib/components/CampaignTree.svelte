<script lang="ts">
  import type { TreeNode } from "../commands";
  import { campaign, selectFile } from "../stores/campaign.svelte";
  import Self from "./CampaignTree.svelte";

  let {
    nodes,
    depth = 0
  }: { nodes: TreeNode[]; depth?: number } = $props();

  function extClass(ext: string | null): string {
    return ext === "json" ? "ext-json" : "ext-md";
  }
</script>

<ul class="tree" class:root={depth === 0}>
  {#each nodes as node (node.relPath)}
    <li>
      {#if node.kind === "dir"}
        <button
          class="tree-dir t-label-lg"
          type="button"
          onclick={() => (node.open = !node.open)}
        >
          <span class="twist t-mono-sm">{node.open ? "▾" : "▸"}</span>
          <span class="dir-name">{node.name}</span>
        </button>
        {#if node.open && node.children.length > 0}
          <Self nodes={node.children} depth={depth + 1} />
        {/if}
      {:else}
        <button
          class="tree-file t-body-md"
          type="button"
          class:selected={campaign.doc?.relPath === node.relPath}
          onclick={() => selectFile(node)}
        >
          <span class="file-ext t-mono-sm {extClass(node.ext)}">{node.ext}</span>
          <span class="file-name">{node.name}</span>
        </button>
      {/if}
    </li>
  {/each}
</ul>

<style>
  .tree {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .tree.root {
    padding: 0 var(--gutter-xs);
  }

  .tree :global(li + li) {
    margin-top: 1px;
  }

  .tree .tree {
    padding-left: 0.875rem;
    border-left: 1px solid var(--border-static);
    margin-left: 0.625rem;
  }

  .tree-dir,
  .tree-file {
    display: flex;
    align-items: center;
    gap: var(--gutter-xs);
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-radius: var(--rounded-sm);
    padding: 3px 6px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background 90ms ease, color 90ms ease;
  }

  .tree-dir {
    color: var(--text-tertiary);
    text-transform: capitalize;
  }

  .tree-dir:hover,
  .tree-file:hover {
    background: var(--surface-highlight);
    color: var(--text-primary);
  }

  .tree-file.selected {
    background: var(--surface-overlay);
    color: var(--amber);
    box-shadow: inset 2px 0 0 var(--amber);
  }

  .twist {
    width: 0.75rem;
    color: var(--text-tertiary);
    flex: none;
  }

  .dir-name,
  .file-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-ext {
    flex: none;
    width: 2rem;
    text-align: right;
    text-transform: uppercase;
    color: var(--text-tertiary);
    font-size: 9px;
  }

  .file-ext.ext-md {
    color: var(--amber);
    opacity: 0.75;
  }

  .file-ext.ext-json {
    color: var(--sapphire);
    opacity: 0.75;
  }
</style>
