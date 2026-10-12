<!-- Start screen when no tournament is open; overview otherwise. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import { importTrf, openTournament, pairNextRound } from '$lib/commands';
  import { t } from '$lib/i18n/index.svelte';
  import { kindKey } from '$lib/kinds';
  import { app } from '$lib/stores/app.svelte';

  let recent = $state<string[]>([]);

  onMount(async () => {
    try {
      recent = await api.recentFiles();
    } catch {
      recent = [];
    }
  });

  const tour = $derived(app.view?.tournament);
  const paired = $derived(tour?.rounds.length ?? 0);
  const current = $derived(tour?.rounds[paired - 1]);
  const pending = $derived(
    current ? current.pairings.filter((p) => p.type === 'Game' && !p.result).length : 0
  );
  const active = $derived(tour?.players.filter((p) => p.status.type === 'Active').length ?? 0);

  function fileName(path: string) {
    return path.split(/[\\/]/).pop() ?? path;
  }
</script>

{#if !tour}
  <div class="start">
    <div class="hero">
      <img src="/logo.png" alt="Chess Organiser logo" width="112" height="112" />
      <h1>Chess Organiser</h1>
      <p class="muted">{t('start-tagline')}</p>
    </div>
    <div class="actions">
      <button class="primary big" onclick={() => (app.newDialog = true)}>{t('new-tournament')}</button>
      <button class="big" onclick={() => openTournament()}>{t('action-open')}</button>
      <button class="big" onclick={importTrf}>{t('action-import-trf')}</button>
    </div>
    <div class="links">
      <a href="/player-db">{t('nav-player-db')}</a>
      <a href="/settings">{t('nav-settings')}</a>
    </div>
    {#if recent.length}
      <section class="card recent">
        <h2>{t('start-recent')}</h2>
        <ul>
          {#each recent as path}
            <li>
              <button class="link" onclick={() => openTournament(path)}>{fileName(path)}</button>
              <span class="muted path">{path}</span>
            </li>
          {/each}
        </ul>
      </section>
    {/if}
  </div>
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
  .start {
    max-width: 640px;
    margin: 8vh auto 0;
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 0 20px;
  }
  .hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    text-align: center;
  }
  .hero h1 {
    font-size: 26px;
  }
  .hero p {
    margin: 0;
    max-width: 440px;
  }
  .actions {
    display: flex;
    gap: 10px;
    justify-content: center;
    flex-wrap: wrap;
  }
  .links {
    display: flex;
    gap: 16px;
    justify-content: center;
  }
  .links a {
    color: var(--accent);
  }
  .big {
    padding: 9px 18px;
    font-size: 14px;
  }
  .recent ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .recent li {
    display: flex;
    gap: 10px;
    align-items: baseline;
    min-width: 0;
  }
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
