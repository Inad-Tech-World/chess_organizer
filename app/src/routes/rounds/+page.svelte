<script lang="ts">
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import type { ByeKind, GameResult, Player, PlayerId } from '$lib/api/types';
  import DataGrid from '$lib/components/DataGrid.svelte';
  import type { Column } from '$lib/components/grid';
  import Dialog from '$lib/components/Dialog.svelte';
  import TeamRound from '$lib/components/TeamRound.svelte';
  import { deleteLastRound, pairNextRound } from '$lib/commands';
  import { fmtPoints, playerRating } from '$lib/format';
  import { t } from '$lib/i18n/index.svelte';
  import { resultCode, resultForKey } from '$lib/keyboard/results';
  import { isTeamKind, kindKey } from '$lib/kinds';
  import { app, notify, run, takeAction } from '$lib/stores/app.svelte';

  type Row =
    | { kind: 'game'; board: number; white: Player; black: Player; result: GameResult | null }
    | { kind: 'bye'; player: Player; bye: ByeKind };

  const tour = $derived(app.view?.tournament);
  const paired = $derived(tour?.rounds.length ?? 0);
  const total = $derived(tour?.settings.rounds ?? 0);
  let chosen = $state<number | null>(null);
  const roundNo = $derived(
    chosen !== null && chosen >= 1 && chosen <= Math.min(paired + 1, total) ? chosen : Math.max(paired, 1)
  );
  const isNext = $derived(roundNo === paired + 1 && paired < total);
  const round = $derived(tour?.rounds[roundNo - 1]);
  const byId = $derived(new Map((tour?.players ?? []).map((p) => [p.id, p])));
  const before = $derived(roundNo >= 2 ? (app.view?.round_scores[roundNo - 2] ?? {}) : {});
  const team = $derived(tour ? isTeamKind(tour.kind) : false);

  const rows: Row[] = $derived.by(() => {
    if (!round) return [];
    const games: Row[] = [];
    const byes: Row[] = [];
    for (const p of round.pairings) {
      if (p.type === 'Game') {
        const white = byId.get(p.white);
        const black = byId.get(p.black);
        if (white && black) games.push({ kind: 'game', board: p.board, white, black, result: p.result });
      } else {
        const player = byId.get(p.player);
        if (player) byes.push({ kind: 'bye', player, bye: p.kind });
      }
    }
    games.sort((a, b) => (a.kind === 'game' && b.kind === 'game' ? a.board - b.board : 0));
    return [...games, ...byes];
  });

  const entered = $derived(rows.filter((r) => r.kind === 'game' && r.result).length);
  const gameCount = $derived(rows.filter((r) => r.kind === 'game').length);

  let selected = $state(0);
  let grid: DataGrid<Row> | undefined = $state();

  function rating(p: Player) {
    return tour ? playerRating(p, tour.settings.rating_type) : null;
  }
  function byePoints(k: ByeKind) {
    const ps = tour?.settings.points;
    if (!ps) return '';
    const p = { PairingAllocated: ps.pairing_bye, Half: ps.half_bye, Full: ps.full_bye, Zero: ps.zero_bye }[k];
    return fmtPoints(p);
  }
  function byeLabel(k: ByeKind) {
    return t(
      k === 'PairingAllocated' ? 'bye-pairing-allocated' : k === 'Half' ? 'bye-half' : k === 'Full' ? 'bye-full' : 'bye-zero'
    );
  }

  const columns: Column<Row>[] = $derived([
    { label: t('col-board'), width: '44px', align: 'right', value: (r) => (r.kind === 'game' ? r.board : '') },
    { label: t('col-start-rank'), width: '48px', align: 'right', value: (r) => (r.kind === 'game' ? r.white : r.player).starting_rank },
    { label: t('col-white'), width: 'minmax(160px, 2fr)', value: (r) => (r.kind === 'game' ? r.white : r.player).name },
    { label: t('col-rating'), width: '56px', align: 'right', value: (r) => rating(r.kind === 'game' ? r.white : r.player) },
    { label: t('col-points'), width: '44px', align: 'center', value: (r) => fmtPoints(before[(r.kind === 'game' ? r.white : r.player).id] ?? 0) },
    { label: t('col-result'), width: '64px', align: 'center', value: (r) => (r.kind === 'game' ? resultCode(r.result) : byePoints(r.bye)) },
    { label: t('col-points'), width: '44px', align: 'center', value: (r) => (r.kind === 'game' ? fmtPoints(before[r.black.id] ?? 0) : '') },
    { label: t('col-start-rank'), width: '48px', align: 'right', value: (r) => (r.kind === 'game' ? r.black.starting_rank : '') },
    { label: t('col-black'), width: 'minmax(160px, 2fr)', value: (r) => (r.kind === 'game' ? r.black.name : byeLabel(r.bye)) },
    { label: t('col-rating'), width: '56px', align: 'right', value: (r) => (r.kind === 'game' ? rating(r.black) : '') }
  ]);

  async function enter(row: Row | undefined, result: GameResult | null) {
    if (!row || row.kind !== 'game') return;
    const ok = await run(() => api.setResult(roundNo, row.board, result));
    if (ok && result) grid?.advance();
  }

  function onkey(e: KeyboardEvent, row: Row | undefined): boolean {
    if (e.metaKey || e.ctrlKey || e.altKey) return false;
    const result = resultForKey(e.key);
    if (result === undefined) return false;
    enter(row, result);
    return true;
  }

  async function swapColors() {
    const row = rows[selected];
    if (row?.kind === 'game') await run(() => api.swapColors(roundNo, row.board));
  }

  async function check() {
    const problems = await run(() => api.checkRound(roundNo));
    if (!problems) return;
    notify(
      problems.length === 0
        ? { kind: 'info', message: t('rounds-check-ok', { round: roundNo }) }
        : { kind: 'error', message: t('rounds-check-problems', { round: roundNo }), details: problems }
    );
  }

  // Manual pairing of the next round.
  let games = $state<[PlayerId, PlayerId][]>([]);
  let firstPick = $state<PlayerId | null>(null);
  let pab = $state<PlayerId | null>(null);
  let violations = $state<string[]>([]);
  let violationsOpen = $state(false);

  const nextRound = $derived(paired + 1);
  const scores = $derived(app.view?.scores ?? {});
  const available = $derived(
    (tour?.players ?? [])
      .filter(
        (p) =>
          p.status.type === 'Active' &&
          p.joined_round <= nextRound &&
          p.byes[nextRound] === undefined
      )
      .sort((a, b) => (scores[b.id] ?? 0) - (scores[a.id] ?? 0) || a.starting_rank - b.starting_rank)
  );
  const requested = $derived((tour?.players ?? []).filter((p) => p.byes[nextRound] !== undefined));
  const used = $derived(new Set([...games.flat(), ...(pab ? [pab] : [])]));
  const unpaired = $derived(available.filter((p) => !used.has(p.id)));

  function pick(p: Player) {
    if (firstPick === null) {
      firstPick = p.id;
    } else if (firstPick === p.id) {
      firstPick = null;
    } else {
      games = [...games, [firstPick, p.id]];
      firstPick = null;
    }
  }

  function swapGame(i: number) {
    games = games.map((g, j) => (j === i ? [g[1], g[0]] : g));
  }

  function removeGame(i: number) {
    games = games.filter((_, j) => j !== i);
  }

  function resetManual() {
    games = [];
    firstPick = null;
    pab = null;
  }

  async function commit(force: boolean) {
    const outcome = await run(() => api.pairManual(games, pab, force));
    if (!outcome) return;
    app.view = outcome.view;
    if (outcome.applied) {
      violationsOpen = false;
      resetManual();
      chosen = outcome.view.tournament.rounds.length;
    } else {
      violations = outcome.violations;
      violationsOpen = true;
    }
  }

  $effect(() => {
    if (takeAction('rounds.manual')) chosen = paired + 1;
    if (takeAction('rounds.check') && paired > 0) check();
  });

  function name(id: PlayerId) {
    return byId.get(id)?.name ?? `#${id}`;
  }
</script>

{#if tour}
  <div class="page fill">
    <header>
      <h1>{t('nav-rounds')}</h1>
      <div class="tabs" role="tablist">
        {#each Array.from({ length: paired }, (_, i) => i + 1) as r}
          <button role="tab" aria-selected={roundNo === r} class:active={roundNo === r} onclick={() => (chosen = r)}>
            {t('round-short', { round: r })}
          </button>
        {/each}
        {#if paired < total}
          <button role="tab" aria-selected={isNext} class:active={isNext} onclick={() => (chosen = paired + 1)}>
            {t('rounds-next-tab', { round: paired + 1 })}
          </button>
        {/if}
      </div>
    </header>

    {#if round && team}
      <div class="toolbar">
        <div class="spacer"></div>
        <button onclick={() => goto(`/lists?list=TeamPairings&round=${roundNo}`)}>{t('rounds-print')}</button>
        {#if roundNo === paired}
          <button class="danger" onclick={deleteLastRound}>{t('rounds-delete')}</button>
        {/if}
      </div>
      <TeamRound {tour} {round} />
    {:else if round}
      <div class="toolbar">
        <span class="muted">{t('rounds-results-entered', { entered, total: gameCount })}</span>
        <div class="spacer"></div>
        <button onclick={swapColors} disabled={rows[selected]?.kind !== 'game'}>{t('rounds-swap')}</button>
        <button onclick={check}>{t('rounds-check')}</button>
        <button onclick={() => goto(`/lists?list=Pairings&round=${roundNo}`)}>{t('rounds-print')}</button>
        {#if roundNo === paired}
          <button class="danger" onclick={deleteLastRound}>{t('rounds-delete')}</button>
        {/if}
      </div>
      <DataGrid
        bind:this={grid}
        {rows}
        {columns}
        bind:selected
        label={t('round-n', { round: roundNo })}
        {onkey}
        ltr
      />
      <p class="muted hint">{t('rounds-keys-hint')}</p>
    {:else if isNext}
      <section class="card auto">
        <div>
          <h2>{t('rounds-pair-title', { round: nextRound })}</h2>
          <p class="muted">{t('rounds-auto-help', { system: t(kindKey(tour.kind)) })}</p>
        </div>
        <button
          class="primary"
          onclick={pairNextRound}
          disabled={app.busy || (team ? (tour.teams.length < 2) : available.length < 2)}
        >
          {t('rounds-pair-auto')}
        </button>
      </section>

      {#if !team}
      <section class="manual">
        <div class="card column">
          <h2>{t('rounds-manual')}</h2>
          <p class="muted">{t('rounds-manual-help')}</p>
          <ul class="players">
            {#each unpaired as p (p.id)}
              <li>
                <button class:picked={firstPick === p.id} onclick={() => pick(p)}>
                  <span class="mono">{p.starting_rank}</span>
                  <span class="name">{p.name}</span>
                  <span class="muted">{fmtPoints(scores[p.id] ?? 0)}</span>
                </button>
              </li>
            {/each}
          </ul>
          {#if unpaired.length === 1}
            <button onclick={() => (pab = unpaired[0].id)}>{t('rounds-give-pab', { name: unpaired[0].name })}</button>
          {/if}
          {#if requested.length}
            <h3>{t('rounds-requested-byes')}</h3>
            <ul class="plain">
              {#each requested as p}<li>{p.name} — {byeLabel(p.byes[nextRound])}</li>{/each}
            </ul>
          {/if}
        </div>
        <div class="card column">
          <h2>{t('rounds-boards', { count: games.length })}</h2>
          <table class="simple">
            <thead>
              <tr><th>{t('col-board')}</th><th>{t('col-white')}</th><th>{t('col-black')}</th><th></th></tr>
            </thead>
            <tbody>
              {#each games as g, i}
                <tr>
                  <td>{i + 1}</td>
                  <td>{name(g[0])}</td>
                  <td>{name(g[1])}</td>
                  <td class="toolbar">
                    <button onclick={() => swapGame(i)} title={t('rounds-swap')}>⇄</button>
                    <button onclick={() => removeGame(i)} title={t('action-remove')}>×</button>
                  </td>
                </tr>
              {/each}
              {#if pab}
                <tr>
                  <td></td><td>{name(pab)}</td><td>{t('bye-pairing-allocated')}</td>
                  <td><button onclick={() => (pab = null)}>×</button></td>
                </tr>
              {/if}
            </tbody>
          </table>
          <div class="toolbar">
            <button onclick={resetManual} disabled={games.length === 0 && !pab}>{t('action-clear')}</button>
            <div class="spacer"></div>
            <button class="primary" disabled={games.length === 0 || unpaired.length > 0 || app.busy} onclick={() => commit(false)}>
              {t('rounds-commit', { round: nextRound })}
            </button>
          </div>
          {#if unpaired.length > 0 && games.length > 0}
            <p class="muted">{t('rounds-unpaired-left', { count: unpaired.length })}</p>
          {/if}
        </div>
      </section>
      {/if}
    {:else}
      <p class="muted">{t('rounds-none')}</p>
    {/if}
  </div>

  <Dialog bind:open={violationsOpen} title={t('rounds-violations-title')} width="520px">
    <p>{t('rounds-violations-help')}</p>
    <ul>
      {#each violations as v}<li>{v}</li>{/each}
    </ul>
    {#snippet footer()}
      <button onclick={() => (violationsOpen = false)}>{t('rounds-violations-fix')}</button>
      <button class="primary danger-fill" onclick={() => commit(true)}>{t('rounds-violations-force')}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .tabs {
    display: flex;
    gap: 2px;
    flex-wrap: wrap;
  }
  .tabs button {
    border-radius: 6px;
    padding: 3px 9px;
  }
  .tabs button.active {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-contrast);
  }
  .hint {
    margin: 0;
    font-size: 12px;
  }
  .auto {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .auto > div {
    flex: 1;
  }
  .auto p {
    margin: 4px 0 0;
  }
  .manual {
    display: grid;
    grid-template-columns: minmax(260px, 1fr) minmax(320px, 1.4fr);
    gap: 12px;
    min-height: 0;
    flex: 1;
  }
  .column {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
    overflow: auto;
  }
  .column p {
    margin: 0;
  }
  .players {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .players button {
    display: grid;
    grid-template-columns: 36px 1fr 36px;
    gap: 8px;
    width: 100%;
    text-align: start;
  }
  .players button.picked {
    background: var(--selected);
    border-color: var(--accent);
  }
  .plain {
    margin: 0;
    padding-inline-start: 18px;
  }
  .danger-fill {
    background: var(--danger);
    border-color: var(--danger);
  }
</style>
