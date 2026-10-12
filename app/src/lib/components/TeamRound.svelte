<script lang="ts">
  import { api } from '$lib/api';
  import type { GameResult, Player, PlayerId, Round, Tournament } from '$lib/api/types';
  import type { TeamMatch } from '$lib/bindings/TeamMatch';
  import DataGrid from '$lib/components/DataGrid.svelte';
  import Dialog from '$lib/components/Dialog.svelte';
  import type { Column } from '$lib/components/grid';
  import { fmtPoints, playerRating } from '$lib/format';
  import { t } from '$lib/i18n/index.svelte';
  import { resultCode, resultForKey } from '$lib/keyboard/results';
  import { run } from '$lib/stores/app.svelte';

  let { tour, round }: { tour: Tournament; round: Round } = $props();

  type Row =
    | { kind: 'match'; m: TeamMatch; home: string; away: string; score: string }
    | { kind: 'game'; m: TeamMatch; board: number; white: Player; black: Player; result: GameResult | null };

  const byId = $derived(new Map(tour.players.map((p) => [p.id, p])));
  const teamName = (id: number | null) => (id === null ? '' : (tour.teams.find((x) => x.id === id)?.name ?? `#${id}`));

  /** Game points of each side of a match, from the board results. */
  function matchScore(m: TeamMatch): string {
    if (m.away === null) return t('bye-pairing-allocated');
    let home = 0;
    let away = 0;
    let missing = false;
    const ps = tour.settings.points;
    const pts = (s: string) =>
      s === 'Win' || s === 'UnratedWin' ? ps.win : s === 'ForfeitWin' ? ps.forfeit_win
        : s === 'Draw' || s === 'UnratedDraw' ? ps.draw : s === 'ForfeitLoss' ? ps.forfeit_loss : ps.loss;
    for (const g of m.games) {
      if (g.type !== 'Game') continue;
      if (!g.result) {
        missing = true;
        continue;
      }
      const homeWhite = m.home_lineup.includes(g.white);
      const w = pts(g.result.white);
      const b = pts(g.result.black);
      home += homeWhite ? w : b;
      away += homeWhite ? b : w;
    }
    const s = `${fmtPoints(home)} – ${fmtPoints(away)}`;
    return missing ? `${s} …` : s;
  }

  const rows: Row[] = $derived.by(() => {
    const out: Row[] = [];
    for (const m of round.matches) {
      out.push({ kind: 'match', m, home: teamName(m.home), away: teamName(m.away), score: matchScore(m) });
      for (const g of m.games) {
        if (g.type !== 'Game') continue;
        const white = byId.get(g.white);
        const black = byId.get(g.black);
        if (white && black) out.push({ kind: 'game', m, board: g.board, white, black, result: g.result });
      }
    }
    return out;
  });

  let selected = $state(0);
  let grid: DataGrid<Row> | undefined = $state();

  const rating = (p: Player) => playerRating(p, tour.settings.rating_type);

  const columns: Column<Row>[] = $derived([
    { label: t('col-board'), width: '56px', align: 'right', value: (r) => (r.kind === 'match' ? r.m.number : `${r.m.number}.${r.board}`) },
    { label: t('col-white'), width: 'minmax(180px, 2fr)', value: (r) => (r.kind === 'match' ? r.home : r.white.name) },
    { label: t('col-rating'), width: '56px', align: 'right', value: (r) => (r.kind === 'match' ? '' : rating(r.white)) },
    { label: t('col-result'), width: '96px', align: 'center', value: (r) => (r.kind === 'match' ? r.score : resultCode(r.result)) },
    { label: t('col-black'), width: 'minmax(180px, 2fr)', value: (r) => (r.kind === 'match' ? r.away : r.black.name) },
    { label: t('col-rating'), width: '56px', align: 'right', value: (r) => (r.kind === 'match' ? '' : rating(r.black)) }
  ]);

  async function enter(row: Row | undefined, result: GameResult | null) {
    if (!row || row.kind !== 'game') return;
    const ok = await run(() => api.setTeamResult(round.number, row.m.number, row.board, result));
    if (ok && result) grid?.advance();
  }

  function onkey(e: KeyboardEvent, row: Row | undefined): boolean {
    if (e.metaKey || e.ctrlKey || e.altKey) return false;
    const result = resultForKey(e.key);
    if (result === undefined) return false;
    enter(row, result);
    return true;
  }

  // Line-up editor for the selected match.
  let lineupOpen = $state(false);
  let editing = $state<TeamMatch | null>(null);
  let homeLineup = $state<(PlayerId | null)[]>([]);
  let awayLineup = $state<(PlayerId | null)[]>([]);
  const boards = $derived(tour.settings.team.boards);

  function openLineups() {
    const row = rows[selected];
    if (!row || row.m.away === null) return;
    editing = row.m;
    const fill = (l: PlayerId[]) => Array.from({ length: boards }, (_, i) => l[i] ?? null);
    homeLineup = fill(row.m.home_lineup);
    awayLineup = fill(row.m.away_lineup);
    lineupOpen = true;
  }

  function members(team: number | null) {
    const tm = tour.teams.find((x) => x.id === team);
    return (tm?.members ?? []).map((id) => byId.get(id)).filter((p): p is Player => !!p);
  }

  async function saveLineups() {
    if (!editing) return;
    const m = editing;
    const clean = (l: (PlayerId | null)[]) => l.filter((x): x is PlayerId => x !== null);
    lineupOpen = false;
    await run(() => api.setLineup(round.number, m.number, m.home, clean(homeLineup)));
    if (m.away !== null) await run(() => api.setLineup(round.number, m.number, m.away!, clean(awayLineup)));
  }
</script>

<div class="toolbar">
  <span class="muted">{t('rounds-matches', { count: round.matches.length })}</span>
  <div class="spacer"></div>
  <button onclick={openLineups} disabled={!rows[selected] || rows[selected].m.away === null}>
    {t('rounds-lineups')}
  </button>
</div>
<DataGrid
  bind:this={grid}
  {rows}
  {columns}
  bind:selected
  label={t('round-n', { round: round.number })}
  {onkey}
  rowClass={(r) => (r.kind === 'match' ? 'match-row' : '')}
  ltr
/>
<p class="muted hint">{t('rounds-team-keys-hint')}</p>

<Dialog bind:open={lineupOpen} title={t('rounds-lineups-title')} width="640px">
  {#if editing}
    <div class="lineups">
      {#each [{ team: editing.home, list: homeLineup }, { team: editing.away, list: awayLineup }] as side}
        <div>
          <h3>{teamName(side.team)}</h3>
          {#each side.list as _, i}
            <label class="board">
              <span class="mono">{i + 1}</span>
              <select bind:value={side.list[i]}>
                <option value={null}>—</option>
                {#each members(side.team) as p}
                  <option value={p.id}>{p.name}</option>
                {/each}
              </select>
            </label>
          {/each}
        </div>
      {/each}
    </div>
    <p class="muted">{t('rounds-lineups-help')}</p>
  {/if}
  {#snippet footer()}
    <button onclick={() => (lineupOpen = false)}>{t('action-cancel')}</button>
    <button class="primary" onclick={saveLineups}>{t('action-save')}</button>
  {/snippet}
</Dialog>

<style>
  .hint {
    margin: 0;
    font-size: 12px;
  }
  :global(.match-row) {
    font-weight: 600;
    background: var(--selected) !important;
  }
  .lineups {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }
  .board {
    display: grid;
    grid-template-columns: 24px 1fr;
    gap: 6px;
    align-items: center;
    margin-bottom: 4px;
  }
</style>
