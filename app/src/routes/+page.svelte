<!-- Start screen when no tournament is open; overview otherwise. -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import Welcome from '$lib/components/Welcome.svelte';
  import { pairNextRound } from '$lib/commands';
  import { t } from '$lib/i18n/index.svelte';
  import { kindKey } from '$lib/kinds';
  import { app } from '$lib/stores/app.svelte';

  const tour = $derived(app.view?.tournament);
  const paired = $derived(tour?.rounds.length ?? 0);
  const current = $derived(tour?.rounds[paired - 1]);
  const pending = $derived(
    current ? current.pairings.filter((p) => p.type === 'Game' && !p.result).length : 0
  );
  const active = $derived(tour?.players.filter((p) => p.status.type === 'Active').length ?? 0);
</script>

{#if !tour}
  <Welcome />
{:else}
  <div class="page">
    <header><h1>{t('nav-overview')}</h1></header>
    <div class="stats">
      <div class="card stat">
        <span class="muted">{t('overview-type')}</span>
        <strong>{t(kindKey(tour.kind))}</strong>
      </div>
      <div class="card stat">
        <span class="muted">{t('overview-players')}</span>
        <strong>{tour.players.length}</strong>
        <span class="muted">{t('overview-active', { count: active })}</span>
      </div>
      <div class="card stat">
        <span class="muted">{t('overview-rounds')}</span>
        <strong>{paired} / {tour.settings.rounds}</strong>
      </div>
      <div class="card stat">
        <span class="muted">{t('overview-pending')}</span>
        <strong>{pending}</strong>
      </div>
    </div>
    <section class="card">
      <h2>{t('overview-next')}</h2>
      <div class="toolbar">
        {#if tour.players.length < 2}
          <button class="primary" onclick={() => goto('/players')}>{t('overview-add-players')}</button>
        {:else if pending > 0}
          <button class="primary" onclick={() => goto('/rounds')}>{t('overview-enter-results', { round: paired })}</button>
        {:else if paired < tour.settings.rounds}
          <button class="primary" onclick={pairNextRound}>{t('overview-pair', { round: paired + 1 })}</button>
          <button onclick={() => goto('/rounds')}>{t('rounds-manual')}</button>
        {:else}
          <button class="primary" onclick={() => goto('/lists?list=Standings')}>{t('overview-final')}</button>
        {/if}
        <button onclick={() => goto('/setup')}>{t('nav-setup')}</button>
      </div>
      <p class="muted path">{app.view?.path}</p>
    </section>
  </div>
{/if}

<style>
  .path {
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 12px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .stat strong {
    font-size: 20px;
  }
  .card .toolbar {
    margin-bottom: 8px;
  }
</style>
