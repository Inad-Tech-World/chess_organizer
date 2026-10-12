<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { api } from '$lib/api';
  import type { ColumnMapping, ListPreview, PlayerRecord, Source } from '$lib/api/types';
  import DataGrid from '$lib/components/DataGrid.svelte';
  import Dialog from '$lib/components/Dialog.svelte';
  import type { Column } from '$lib/components/grid';
  import { CSV, FIDE_LIST, pickOpen } from '$lib/files';
  import { t } from '$lib/i18n/index.svelte';
  import { notify, run, showError } from '$lib/stores/app.svelte';

  let query = $state('');
  let results = $state<PlayerRecord[]>([]);
  let sources = $state<Source[]>([]);
  let progress = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let unlisten: UnlistenFn | undefined;

  // National list mapping dialog.
  let mappingOpen = $state(false);
  let csvPath = $state('');
  let preview = $state<ListPreview | null>(null);
  let mapping = $state<ColumnMapping | null>(null);
  let sourceName = $state('National');

  const FIELDS: [keyof ColumnMapping, string][] = [
    ['name', 'player-db-field-name'],
    ['surname', 'player-db-field-surname'],
    ['firstName', 'player-db-field-first-name'],
    ['nationalId', 'player-db-field-national-id'],
    ['rating', 'player-db-field-rating'],
    ['fideId', 'col-fide-id'],
    ['federation', 'col-fed'],
    ['sex', 'col-sex'],
    ['birthYear', 'col-birth'],
    ['club', 'col-club'],
    ['title', 'col-title']
  ];

  function setField(m: ColumnMapping, key: keyof ColumnMapping, value: number | null) {
    (m as unknown as Record<string, number | null>)[key] = value;
  }

  async function refreshSources() {
    try {
      sources = await api.playerDbSources();
    } catch (e) {
      showError(e);
    }
  }

  onMount(async () => {
    await refreshSources();
    try {
      unlisten = await listen<{ stage: string; count: number }>('player-db-progress', (e) => {
        const { stage, count } = e.payload;
        progress =
          stage === 'download'
            ? t('player-db-downloading', { mb: Math.round(count / 1048576) })
            : t('player-db-importing', { count });
      });
    } catch {
      // Browser preview: no event bridge.
    }
  });
  onDestroy(() => unlisten?.());

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

  async function finished(count: number | undefined) {
    progress = null;
    if (count !== undefined) {
      notify({ kind: 'info', message: t('player-db-imported', { count }) });
      await refreshSources();
      if (query.trim().length >= 2) search();
    }
  }

  async function downloadFide() {
    progress = t('player-db-connecting');
    await finished(await run(() => api.playerDbDownloadFide()));
  }

  async function importFide() {
    const path = await pickOpen(FIDE_LIST);
    if (!path) return;
    progress = t('player-db-importing', { count: 0 });
    await finished(await run(() => api.playerDbImportFide(path)));
  }

  async function importNational() {
    const path = await pickOpen(CSV);
    if (!path) return;
    const p = await run(() => api.playerDbPreviewNational(path));
    if (!p) return;
    csvPath = path;
    preview = p;
    mapping = { ...p.mapping };
    mappingOpen = true;
  }

  async function confirmNational() {
    if (!mapping) return;
    mappingOpen = false;
    const m = $state.snapshot(mapping) as ColumnMapping;
    await finished(await run(() => api.playerDbImportNational(csvPath, m, sourceName)));
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
    { label: t('col-birth'), width: '56px', align: 'right', value: (r) => r.birth_year },
    { label: t('col-club'), width: 'minmax(120px, 1fr)', value: (r) => r.club ?? '' }
  ]);
</script>

<div class="page fill">
  <header>
    <h1>{t('nav-player-db')}</h1>
    <div class="spacer"></div>
    <div class="toolbar">
      <button class="primary" onclick={downloadFide} disabled={progress !== null}>
        {t('player-db-download-fide')}
      </button>
      <button onclick={importFide} disabled={progress !== null}>{t('player-db-import-fide')}</button>
      <button onclick={importNational} disabled={progress !== null}>
        {t('player-db-import-national')}
      </button>
    </div>
  </header>
  <p class="muted">{t('player-db-help')}</p>
  {#if progress}
    <p class="progress" role="status">{progress}</p>
  {/if}
  {#if sources.length}
    <div class="toolbar">
      {#each sources as s}
        <span class="badge">{s.name} {s.period ?? ''} · {s.players.toLocaleString()}</span>
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

<Dialog bind:open={mappingOpen} title={t('player-db-mapping-title')} width="760px">
  {#if preview && mapping}
    <p class="muted">{t('player-db-mapping-help')}</p>
    <div class="form-grid">
      <label>
        {t('player-db-source-name')}
        <input bind:value={sourceName} />
      </label>
      <label class="check">
        <input type="checkbox" bind:checked={mapping.hasHeader} />
        {t('player-db-has-header')}
      </label>
      {#each FIELDS as [key, label]}
        <label>
          {t(label)}
          <select
            value={mapping[key] ?? ''}
            onchange={(e) => {
              const v = (e.currentTarget as HTMLSelectElement).value;
              if (mapping) setField(mapping, key, v === '' ? null : Number(v));
            }}
          >
            <option value="">—</option>
            {#each preview.headers as h, i}
              <option value={i}>{i + 1}: {h}</option>
            {/each}
          </select>
        </label>
      {/each}
    </div>
    <div class="preview">
      <table>
        <thead>
          <tr>{#each preview.headers as h}<th>{h}</th>{/each}</tr>
        </thead>
        <tbody>
          {#each preview.rows as r}
            <tr>{#each r as c}<td>{c}</td>{/each}</tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
  {#snippet footer()}
    <button onclick={() => (mappingOpen = false)}>{t('action-cancel')}</button>
    <button
      class="primary"
      disabled={!mapping || (mapping.name === null && mapping.surname === null)}
      onclick={confirmNational}>{t('player-db-import')}</button
    >
  {/snippet}
</Dialog>

<style>
  p {
    margin: 0;
  }
  .progress {
    color: var(--link);
    font-weight: 600;
  }
  .form-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px 12px;
    margin: 10px 0;
  }
  .form-grid label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 0.85rem;
  }
  .form-grid label.check {
    flex-direction: row;
    align-items: center;
    gap: 6px;
  }
  .preview {
    overflow: auto;
    max-height: 220px;
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  table {
    border-collapse: collapse;
    font-size: 0.8rem;
    white-space: nowrap;
  }
  th,
  td {
    padding: 3px 8px;
    border-bottom: 1px solid var(--border);
    text-align: start;
  }
</style>
