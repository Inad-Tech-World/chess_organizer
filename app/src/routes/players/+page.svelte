<script lang="ts">
  import { goto } from '$app/navigation';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { api } from '$lib/api';
  import type { ByeKind, Player, PlayerRecord } from '$lib/api/types';
  import DataGrid from '$lib/components/DataGrid.svelte';
  import type { Column } from '$lib/components/grid';
  import Dialog from '$lib/components/Dialog.svelte';
  import PlayerForm from '$lib/components/PlayerForm.svelte';
  import PlayerPicker from '$lib/components/PlayerPicker.svelte';
  import { fmtPoints, playerRating } from '$lib/format';
  import { t } from '$lib/i18n/index.svelte';
  import { blankPlayer, clone, playerFromRecord } from '$lib/players';
  import { app, run, takeAction } from '$lib/stores/app.svelte';

  const tour = $derived(app.view?.tournament);
  const rows = $derived(
    tour ? [...tour.players].sort((a, b) => a.starting_rank - b.starting_rank || a.id - b.id) : []
  );
  let selected = $state(0);
  const current = $derived(rows[selected] as Player | undefined);
  const nextRound = $derived(app.view?.next_round ?? 1);

  let editing = $state<Player | null>(null);
  let editOpen = $state(false);
  let isNew = $state(false);
  let withdrawOpen = $state(false);
  let withdrawRound = $state(1);
  let byesOpen = $state(false);
  let byeDraft = $state<Record<number, ByeKind | ''>>({});

  function status(p: Player): string {
    if (p.status.type === 'Withdrawn') return t('status-withdrawn', { round: p.status.from_round });
    const byes = Object.entries(p.byes).map(([r, k]) => `R${r} ${byeShort(k)}`);
    const late = p.joined_round > 1 ? [t('status-late', { round: p.joined_round })] : [];
    return [...late, ...byes].join(', ');
  }

  function byeShort(k: ByeKind): string {
    return k === 'Half' ? '½' : k === 'Full' ? '1' : k === 'Zero' ? '0' : 'PAB';
  }

  const columns: Column<Player>[] = $derived([
    { label: t('col-start-rank'), width: '52px', align: 'right', value: (p) => p.starting_rank },
    { label: t('col-title'), width: '52px', value: (p) => p.title ?? '' },
    { label: t('col-name'), width: 'minmax(200px, 2fr)', value: (p) => p.name },
    { label: t('col-fed'), width: '52px', align: 'center', value: (p) => p.federation ?? '' },
    {
      label: t('col-rating'),
      width: '60px',
      align: 'right',
      value: (p) => (tour ? playerRating(p, tour.settings.rating_type) : null)
    },
    { label: t('col-fide-id'), width: '96px', align: 'right', value: (p) => p.fide_id },
    { label: t('col-club'), width: 'minmax(120px, 1fr)', value: (p) => p.club ?? '' },
    {
      label: t('col-points'),
      width: '52px',
      align: 'right',
      value: (p) => fmtPoints(app.view?.scores[p.id])
    },
    { label: t('col-status'), width: 'minmax(140px, 1fr)', value: status }
  ]);

  $effect(() => {
    if (takeAction('players.add')) openAdd();
    if (takeAction('players.sort')) sort();
  });

  function openAdd() {
    editing = blankPlayer();
    isNew = true;
    editOpen = true;
  }

  function openEdit(p: Player | undefined) {
    if (!p) return;
    editing = clone(p);
    isNew = false;
    editOpen = true;
  }

  function pick(r: PlayerRecord) {
    const late = editing?.late_entry_bye ?? 'Zero';
    editing = { ...playerFromRecord(r), late_entry_bye: late };
  }

  async function save() {
    if (!editing || !editing.name.trim()) return;
    const p = editing;
    const ok = await run(() => (isNew ? api.addPlayer(p) : api.updatePlayer(p)));
    if (!ok) return;
    if (isNew) {
      // Stay open for fast entry of the next player.
      editing = { ...blankPlayer(), late_entry_bye: p.late_entry_bye };
    } else {
      editOpen = false;
    }
  }

  async function remove() {
    const p = current;
    if (!p) return;
    if (await ask(t('players-remove-confirm', { name: p.name }), { kind: 'warning' })) {
      await run(() => api.removePlayer(p.id));
    }
  }

  function openWithdraw() {
    withdrawRound = nextRound;
    withdrawOpen = true;
  }

  async function withdraw() {
    const p = current;
    if (p && (await run(() => api.withdrawPlayer(p.id, withdrawRound)))) withdrawOpen = false;
  }

  async function reenter() {
    const p = current;
    if (p) await run(() => api.reenterPlayer(p.id));
  }

  function openByes() {
    const p = current;
    if (!p || !tour) return;
    const draft: Record<number, ByeKind | ''> = {};
    for (let r = nextRound; r <= tour.settings.rounds; r++) draft[r] = p.byes[r] ?? '';
    byeDraft = draft;
    byesOpen = true;
  }

  async function saveByes() {
    const p = current;
    if (!p) return;
    for (const [r, kind] of Object.entries(byeDraft)) {
      const round = Number(r);
      const before = p.byes[round] ?? '';
      if (before === kind) continue;
      const ok = await run(() => api.setBye(p.id, round, kind === '' ? null : kind));
      if (!ok) return;
    }
    byesOpen = false;
  }

  async function sort() {
    await run(() => api.sortPlayers());
  }

  function onkey(e: KeyboardEvent): boolean {
    if (e.key === 'Delete' || e.key === 'Backspace') {
      remove();
      return true;
    }
    if (e.key === 'Insert') {
      openAdd();
      return true;
    }
    return false;
  }
</script>

<div class="page fill">
  <header>
    <h1>{t('nav-players')}</h1>
    <span class="muted">{t('players-count', { count: rows.length })}</span>
    <div class="spacer"></div>
    <div class="toolbar">
      <button class="primary" onclick={openAdd}>{t('players-add')}</button>
      <button onclick={() => openEdit(current)} disabled={!current}>{t('action-edit')}</button>
      <button onclick={remove} disabled={!current}>{t('action-remove')}</button>
      {#if current?.status.type === 'Withdrawn'}
        <button onclick={reenter}>{t('players-reenter')}</button>
      {:else}
        <button onclick={openWithdraw} disabled={!current}>{t('players-withdraw')}</button>
      {/if}
      <button onclick={openByes} disabled={!current || nextRound > (tour?.settings.rounds ?? 0)}>
        {t('players-byes')}
      </button>
      <button onclick={sort} disabled={rows.length < 2}>{t('players-sort')}</button>
      <button onclick={() => goto('/player-db')}>{t('nav-player-db')}</button>
    </div>
  </header>

  <DataGrid
    {rows}
    {columns}
    bind:selected
    label={t('nav-players')}
    empty={t('players-empty')}
    onactivate={(p) => openEdit(p)}
    {onkey}
    rowClass={(p) => (p.status.type === 'Withdrawn' ? 'dim' : '')}
  />
  <p class="muted hint">{t('players-hint')}</p>
</div>

<Dialog bind:open={editOpen} title={isNew ? t('players-add') : t('players-edit')} width="680px">
  {#if editing}
    {#if isNew}
      <PlayerPicker onpick={pick} />
      <hr />
    {/if}
    <PlayerForm bind:player={editing} />
    {#if isNew && nextRound > 1}
      <div class="late">
        <span>{t('players-late-entry', { round: nextRound })}</span>
        <select bind:value={editing.late_entry_bye}>
          <option value="Zero">{t('bye-zero-points')}</option>
          <option value="Half">{t('bye-half-points')}</option>
        </select>
      </div>
    {/if}
  {/if}
  {#snippet footer()}
    <button onclick={() => (editOpen = false)}>{t('action-close')}</button>
    <button class="primary" disabled={!editing?.name.trim() || app.busy} onclick={save}>
      {isNew ? t('players-add-another') : t('action-save')}
    </button>
  {/snippet}
</Dialog>

<Dialog bind:open={withdrawOpen} title={t('players-withdraw')} width="380px">
  <p>{current?.name}</p>
  <label class="field">
    {t('players-withdraw-from')}
    <input type="number" min="1" max={tour?.settings.rounds} bind:value={withdrawRound} />
  </label>
  {#snippet footer()}
    <button onclick={() => (withdrawOpen = false)}>{t('action-cancel')}</button>
    <button class="primary" onclick={withdraw}>{t('players-withdraw')}</button>
  {/snippet}
</Dialog>

<Dialog bind:open={byesOpen} title={t('players-byes-title', { name: current?.name ?? '' })} width="420px">
  <p class="muted">{t('players-byes-help')}</p>
  <table class="simple">
    <tbody>
      {#each Object.keys(byeDraft) as r}
        <tr>
          <td>{t('round-n', { round: Number(r) })}</td>
          <td>
            <select bind:value={byeDraft[Number(r)]}>
              <option value="">{t('bye-none')}</option>
              <option value="Half">{t('bye-half-points')}</option>
              <option value="Zero">{t('bye-zero-points')}</option>
              <option value="Full">{t('bye-full-points')}</option>
            </select>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#snippet footer()}
    <button onclick={() => (byesOpen = false)}>{t('action-cancel')}</button>
    <button class="primary" onclick={saveByes}>{t('action-save')}</button>
  {/snippet}
</Dialog>

<style>
  .hint {
    margin: 0;
    font-size: 12px;
  }
  hr {
    border: none;
    border-top: 1px solid var(--border);
    margin: 12px 0;
  }
  .late {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-top: 12px;
    padding: 8px 10px;
    border-radius: 6px;
    background: var(--surface-2);
  }
</style>
