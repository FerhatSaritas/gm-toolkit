<script lang="ts">
  import { campaign, openCampaign } from "../stores/campaign.svelte";
</script>

<div class="empty">
  <div class="empty-mark t-mono-lg">◆</div>
  <h1 class="t-display-lg empty-title">GM Toolkit</h1>
  <div class="divider-diamond empty-divider" role="separator"></div>
  <p class="t-body-lg empty-tagline">
    A private grimoire for running campaigns.<br />
    Scenes, encounters and statblocks — all on your disk.
  </p>

  {#if campaign.info}
    <p class="t-body-md empty-open">
      Campaign “<span class="empty-campaign">{campaign.info.name}</span>” is
      loaded — pick a file from the library.
    </p>
  {:else}
    <button class="btn btn-primary t-label-lg btn-open" type="button" onclick={openCampaign}>
      Open Campaign…
    </button>
    <p class="t-mono-sm empty-hint">
      campaign/&nbsp;&nbsp;scenes/&nbsp;&nbsp;encounters/&nbsp;&nbsp;npcs/&nbsp;&nbsp;items/
    </p>
  {/if}
</div>

<style>
  .empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--gutter-md);
    text-align: center;
    padding: var(--gutter-xl);
  }

  .empty-mark {
    color: var(--amber);
    font-size: 20px;
    text-shadow: 0 0 12px rgba(224, 169, 83, 0.45);
  }

  .empty-title {
    color: var(--text-primary);
    margin: 0;
  }

  .empty-divider {
    width: 14rem;
    margin: var(--gutter-xs) 0;
  }

  .empty-tagline {
    color: var(--text-secondary);
    margin: 0 0 var(--gutter-md);
  }

  .empty-open {
    color: var(--text-secondary);
    margin: 0;
  }

  .empty-campaign {
    color: var(--amber);
  }

  .btn-open {
    min-width: 12rem;
  }

  .empty-hint {
    color: var(--text-tertiary);
    letter-spacing: 0.04em;
  }
</style>
