<script lang="ts">
  import { api } from '$lib/api';
  import PhaseBanner from '$lib/components/PhaseBanner.svelte';
  import { exportTrf } from '$lib/commands';
  import { t } from '$lib/i18n/index.svelte';
  import { app, run } from '$lib/stores/app.svelte';

  const tour = $derived(app.view?.tournament);
  let player = $state<number | null>(null);
</script>

{#if tour}
  <div class="page">
    <header><h1>{t('nav-ratings')}</h1></header>
    <section class="card">
      <h2>{t('ratings-trf-title')}</h2>
      <p class="muted">{t('ratings-trf-help')}</p>
      <button class="primary" onclick={exportTrf}>{t('action-export-trf')}</button>
    </section>
    <section class="card">
      <h2>{t('ratings-changes-title')}</h2>
      <PhaseBanner phase={2} feature={t('ratings-changes-feature')} />
      <p class="muted">{t('ratings-changes-help')}</p>
      <button onclick={() => run(() => api.ratingReport())}>{t('ratings-calculate')}</button>
    </section>
    <section class="card">
      <h2>{t('ratings-norms-title')}</h2>
      <PhaseBanner phase={4} feature={t('ratings-norms-feature')} />
      <div class="toolbar">
        <select bind:value={player}>
          <option value={null}>—</option>
          {#each tour.players as p}<option value={p.id}>{p.name}</option>{/each}
        </select>
        <button disabled={player === null} onclick={() => player !== null && run(() => api.normCheck(player!))}>
          {t('ratings-check-norms')}
        </button>
      </div>
    </section>
  </div>
{/if}

<style>
  section p {
    margin: 0 0 10px;
  }
  section :global(.banner) {
    margin-bottom: 10px;
  }
</style>
