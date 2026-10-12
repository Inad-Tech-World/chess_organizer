<script lang="ts">
  import { page } from '$app/state';
  import { api } from '$lib/api';
  import type { ExportFormat, ListKind } from '$lib/api/types';
  import { pickSave, suggestName } from '$lib/files';
  import { i18n, LANGUAGES, t } from '$lib/i18n/index.svelte';
  import { app, notify, run, showError } from '$lib/stores/app.svelte';

  type ListDef = { id: ListKind['type']; key: string; round: boolean; team?: boolean };

  const ALL_LISTS: ListDef[] = [
    { id: 'StartingRank', key: 'list-starting-rank', round: false },
    { id: 'Alphabetical', key: 'list-alphabetical', round: false },
    { id: 'Pairings', key: 'list-name-pairings', round: true },
    { id: 'Standings', key: 'list-name-standings', round: true },
    { id: 'Crosstable', key: 'list-name-crosstable', round: true },
    { id: 'Scorecards', key: 'list-scorecards', round: false },
    { id: 'BoardCards', key: 'list-board-cards', round: true },
    { id: 'RatingReport', key: 'list-rating-report', round: false },
    { id: 'Prizes', key: 'list-prizes', round: false },
    { id: 'TeamRosters', key: 'list-team-roster', round: false, team: true },
    { id: 'TeamPairings', key: 'list-name-team-pairings', round: true, team: true },
    { id: 'TeamStandings', key: 'list-name-team-standings', round: true, team: true }
  ];

  const isTeam = $derived(app.view?.tournament.kind.type.startsWith('Team') ?? false);
  const LISTS = $derived(ALL_LISTS.filter((l) => !l.team || isTeam));

  const FORMATS: { id: ExportFormat; label: string }[] = [
    { id: 'html', label: 'HTML' },
    { id: 'pdf', label: 'PDF' },
    { id: 'csv', label: 'CSV' },
    { id: 'xlsx', label: 'Excel' }
  ];

  const paired = $derived(app.view?.tournament.rounds.length ?? 0);
  let listId = $state<ListKind['type']>('StartingRank');
  let round = $state(1);
  let lang = $state(i18n.lang);
  let format = $state<ExportFormat>('html');
  let html = $state('');

  const def = $derived(ALL_LISTS.find((l) => l.id === listId) ?? ALL_LISTS[0]);

  // Follow ?list=…&round=… when arriving from a menu or another screen.
  $effect(() => {
    const q = page.url.searchParams;
    const l = q.get('list');
    if (l && ALL_LISTS.some((d) => d.id === l)) listId = l as ListKind['type'];
    round = Number(q.get('round')) || Math.max(paired, 1);
  });

  const kind: ListKind = $derived(
    (def.round ? { type: def.id, round } : { type: def.id }) as ListKind
  );

  $effect(() => {
    const k = kind;
    const l = lang;
    void app.view;
    if (def.round && paired === 0) {
      html = '';
      return;
    }
    api
      .listHtml(k, l)
      .then((h) => {
        html = h;
      })
      .catch((e) => {
        html = '';
        showError(e);
      });
  });

  async function print() {
    await run(() => api.listPrint(kind, lang));
  }

  async function exportList() {
    const name = `${app.view?.tournament.info.name ?? 'list'} - ${t(def.key)}`;
    const path = await pickSave([{ name: format.toUpperCase(), extensions: [format] }], suggestName(name, format));
    if (!path) return;
    if ((await run(() => api.listExport(kind, format, lang, path))) !== undefined) {
      notify({ kind: 'info', message: t('export-done', { path }) });
    }
  }
</script>

<div class="page fill">
  <header>
    <h1>{t('nav-lists')}</h1>
    <div class="spacer"></div>
    <div class="toolbar">
      <label class="inline">
        {t('lists-language')}
        <select bind:value={lang}>
          {#each LANGUAGES as l}<option value={l.code}>{l.name}</option>{/each}
        </select>
      </label>
      <button class="primary" onclick={print} disabled={!html}>{t('lists-print')}</button>
      <select bind:value={format} aria-label={t('lists-format')}>
        {#each FORMATS as f}
          <option value={f.id}>{f.label}</option>
        {/each}
      </select>
      <button onclick={exportList} disabled={!html}>{t('lists-export')}</button>
    </div>
  </header>

  <div class="split">
    <nav class="card">
      {#each LISTS as l}
        <button class:active={l.id === listId} onclick={() => (listId = l.id)}>
          <span>{t(l.key)}</span>
        </button>
      {/each}
      {#if def.round}
        <label class="field round">
          {t('lists-round')}
          <input type="number" min="1" max={Math.max(paired, 1)} bind:value={round} />
        </label>
      {/if}
    </nav>
    <div class="preview">
      {#if def.round && paired === 0}
        <p class="muted">{t('lists-no-rounds')}</p>
      {:else}
        <iframe title={t(def.key)} srcdoc={html} sandbox=""></iframe>
      {/if}
    </div>
  </div>
  <p class="muted hint">{t('lists-print-hint')}</p>
</div>

<style>
  .split {
    display: grid;
    grid-template-columns: 210px 1fr;
    gap: 12px;
    flex: 1;
    min-height: 0;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px;
    overflow: auto;
  }
  nav button {
    display: flex;
    justify-content: space-between;
    gap: 6px;
    border: none;
    text-align: start;
    background: none;
  }
  nav button.active {
    background: var(--selected);
  }
  .round {
    margin-top: 10px;
    padding: 0 8px;
  }
  .preview {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  iframe {
    flex: 1;
    width: 100%;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: #fff;
  }
  .inline {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .hint {
    margin: 0;
    font-size: 12px;
  }
</style>
