<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { PlayerRecord, Source } from '$lib/api/types';
  import DataGrid from '$lib/components/DataGrid.svelte';
  import type { Column } from '$lib/components/grid';
  import { CSV, FIDE_LIST, pickOpen } from '$lib/files';
  import { t } from '$lib/i18n/index.svelte';
  import { run, showError } from '$lib/stores/app.svelte';

  let query = $state('');
  let results = $state<PlayerRecord[]>([]);
  let sources = $state<Source[]>([]);
  let timer: ReturnType<typeof setTimeout> | undefined;

  onMount(async () => {
    try {
      sources = await api.playerDbSources();
    } catch (e) {
      showError(e);
    }
  });

  function search() {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      try {
        results = query.trim().length >= 2 ? await api.playerDbSearch(query, 200) : [];
      } catch (e) {
        showError(e);
      }
    }, 150);
  }

  async function importFide() {
    const path = await pickOpen(FIDE_LIST);
    if (path) await run(() => api.playerDbImportFide(path));
  }

  async function importNational() {
    const path = await pickOpen(CSV);
    if (path) await run(() => api.playerDbImportNational(path));
  }

  const columns: Column<PlayerRecord>[] = $derived([
    { label: t('col-title'), width: '52px', value: (r) => r.title ?? '' },
    { label: t('col-name'), width: 'minmax(220px, 2fr)', value: (r) => r.name },
    { label: t('col-fed'), width: '52px', align: 'center', value: (r) => r.federation ?? '' },
    { label: t('rating-standard'), width: '72px', align: 'right', value: (r) => r.rating_standard },
    { label: t('rating-rapid'), width: '64px', align: 'right', value: (r) => r.rating_rapid },
    { label: t('rating-blitz'), width: '64px', align: 'right', value: (r) => r.rating_blitz },
    { label: t('rating-national'), width: '72px', align: 'right', value: (r) => r.national_rating },
    { label: t('col-fide-id'), width: '96px', align: 'right', value: (r) => r.fide_id },
    { label: t('col-birth'), width: '56px', align: 'right', value: (r) => r.birth_year }
  ]);
</script>

<div class="page fill">
  <header>
    <h1>{t('nav-player-db')}</h1>
    <div class="spacer"></div>
    <div class="toolbar">
      <button class="primary" onclick={importFide}>{t('player-db-import-fide')}</button>
      <button onclick={importNational}>{t('player-db-import-national')}</button>
    </div>
  </header>
  <p class="muted">{t('player-db-help')}</p>
  {#if sources.length}
    <div class="toolbar">
      {#each sources as s}
        <span class="badge">{s.name} {s.period ?? ''} · {s.players}</span>
      {/each}
    </div>
  {/if}
  <input
    type="search"
    placeholder={t('player-db-search-placeholder')}
    bind:value={query}
    oninput={search}
    aria-label={t('player-db-search-placeholder')}
  />
  <DataGrid
    rows={results}
    {columns}
    label={t('nav-player-db')}
    empty={query.trim().length >= 2 ? t('player-db-no-results') : t('player-db-type-to-search')}
  />
</div>

<style>
  p {
    margin: 0;
  }
</style>
