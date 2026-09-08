<script lang="ts">
  import { campaign, openCampaign } from "../stores/campaign.svelte";
  import { inspector } from "../stores/inspector.svelte";
  import StatblockView from "./StatblockView.svelte";

  const counts = $derived(campaign.info?.counts);
</script>

<aside class="rail inspector">
  <div class="rail-header">
    <span class="rail-title t-headline-sm">{inspector.statblock ? "Creature" : "Inspector"}</span>
  </div>

  {#if inspector.statblock}
    <div class="sb-wrap">
      <StatblockView doc={inspector.statblock} />
    </div>
  {:else if campaign.info && counts}
    <div class="insp-body">
      <section class="card">
        <h2 class="card-title t-headline-sm">{campaign.info.name}</h2>
        <p class="card-sub t-mono-sm">{campaign.info.path}</p>
      </section>

      <section class="card">
        <h3 class="counts-title t-label-lg">Library</h3>
        <div class="counts-grid">
          {#each [["Scenes", counts.scenes], ["Encounters", counts.encounters], ["NPCs", counts.npcs], ["Statblocks", counts.statblocks], ["Items", counts.items], ["Files", counts.total]] as [label, value] (label)}
            <div class="count-cell">
              <span class="count-value t-mono-lg">{value}</span>
              <span class="count-label t-label-lg">{label}</span>
            </div>
          {/each}
        </div>
      </section>

      <button class="btn btn-secondary t-label-lg" type="button" onclick={openCampaign}>
        Switch Campaign…
      </button>
    </div>
  {:else}
    <div class="insp-empty">
      <p class="t-body-sm">
        {inspector.loading
          ? "Loading statblock…"
          : "The statblock drawer lives here — click any creature in an encounter to open its reference."}
      </p>
    </div>
  {/if}
</aside>

<style>
  .sb-wrap {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    padding: var(--gutter-sm) var(--inset-panel);
  }

  .insp-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--gutter-md);
    padding: var(--gutter-sm) var(--inset-panel);
  }

  .card {
    background: var(--surface-raised);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded);
    box-shadow:
      inset 0 1px 0 var(--border-bevel-light),
      0 1px 0 var(--border-bevel-dark);
    padding: var(--gutter-md);
  }

  .card-title {
    color: var(--text-primary);
    margin: 0;
  }

  .card-sub {
    color: var(--text-tertiary);
    margin: var(--gutter-xs) 0 0;
    word-break: break-all;
  }

  .counts-title {
    color: var(--text-tertiary);
    text-transform: uppercase;
    margin: 0 0 var(--gutter-sm);
  }

  .counts-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--gutter-sm);
  }

  .count-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    background: var(--surface-base);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded-sm);
    padding: var(--gutter-xs) var(--gutter-xs);
  }

  .count-value {
    color: var(--amber);
  }

  .count-label {
    color: var(--text-tertiary);
    font-size: 10px;
    text-transform: uppercase;
  }

  .insp-empty {
    padding: var(--gutter-md);
  }

  .insp-empty p {
    color: var(--text-tertiary);
    margin: 0;
  }
</style>
