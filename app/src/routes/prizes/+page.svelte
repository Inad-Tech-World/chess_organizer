<script lang="ts">
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import type { PrizeCategory, PrizeWinner } from '$lib/api/types';
  import Dialog from '$lib/components/Dialog.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { clone } from '$lib/players';
  import { app, run, showError } from '$lib/stores/app.svelte';

  const tour = $derived(app.view?.tournament);
  let winners = $state<PrizeWinner[]>([]);

  $effect(() => {
    const t = app.view?.tournament;
    if (!t || t.rounds.length === 0 || t.prize_categories.length === 0) {
      winners = [];
      return;
    }
    api.prizeWinners().then((w) => (winners = w)).catch(showError);
  });

  function playerName(id: number) {
    return tour?.players.find((p) => p.id === id)?.name ?? `#${id}`;
  }
  let editing = $state<PrizeCategory | null>(null);
  let prizesText = $state('');
  let open = $state(false);

  function blank(): PrizeCategory {
    return {
      id: 0,
      name: '',
      filter: {
        min_rating: null,
        max_rating: null,
        sex: null,
        born_from: null,
        born_until: null,
        federation: null,
        club: null,
        unrated_only: false
      },
      prizes: []
    };
  }

  function edit(c: PrizeCategory | null) {
    editing = c ? clone(c) : blank();
    prizesText = editing.prizes.join('\n');
    open = true;
  }

  function num(v: string): number | null {
    const n = parseInt(v, 10);
    return Number.isFinite(n) ? n : null;
  }

  async function save() {
    const c = editing;
    if (!c || !c.name.trim()) return;
    c.prizes = prizesText.split('\n').map((s) => s.trim()).filter(Boolean);
    if (await run(() => api.savePrizeCategory(c))) open = false;
  }

  function describe(c: PrizeCategory): string {
    const f = c.filter;
    const parts: string[] = [];
    if (f.max_rating != null) parts.push(`≤ ${f.max_rating}`);
    if (f.min_rating != null) parts.push(`≥ ${f.min_rating}`);
    if (f.sex === 'Female') parts.push(t('sex-female'));
    if (f.born_from != null) parts.push(t('prizes-born-from', { year: f.born_from }));
    if (f.born_until != null) parts.push(t('prizes-born-until', { year: f.born_until }));
    if (f.federation) parts.push(f.federation);
    if (f.unrated_only) parts.push(t('prizes-unrated'));
    return parts.join(' · ');
  }
</script>

{#if tour}
  <div class="page">
    <header>
      <h1>{t('nav-prizes')}</h1>
      <div class="spacer"></div>
      <button onclick={() => goto('/lists?list=Prizes')}>{t('list-prizes')}</button>
      <button class="primary" onclick={() => edit(null)}>{t('prizes-add')}</button>
    </header>
    <section class="card">
      <table class="simple">
        <thead><tr><th>{t('prizes-category')}</th><th>{t('prizes-filter')}</th><th>{t('prizes-prizes')}</th><th></th></tr></thead>
        <tbody>
          {#each tour.prize_categories as c}
            <tr>
              <td>{c.name}</td>
              <td class="muted">{describe(c)}</td>
              <td>{c.prizes.join(', ')}</td>
              <td class="toolbar">
                <button onclick={() => edit(c)}>{t('action-edit')}</button>
                <button onclick={() => run(() => api.removePrizeCategory(c.id))}>{t('action-remove')}</button>
              </td>
            </tr>
          {:else}
            <tr><td colspan="4" class="muted">{t('prizes-empty')}</td></tr>
          {/each}
        </tbody>
      </table>
    </section>
    {#if winners.length}
      <section class="card">
        <h2>{t('prizes-winners', { round: tour.rounds.length })}</h2>
        <table class="simple">
          <thead><tr><th>{t('prizes-category')}</th><th>{t('col-place')}</th><th>{t('col-prize')}</th><th>{t('col-name')}</th></tr></thead>
          <tbody>
            {#each winners as w}
              <tr><td>{w.category}</td><td>{w.place}</td><td>{w.prize}</td><td>{playerName(w.player)}</td></tr>
            {/each}
          </tbody>
        </table>
        <p class="muted">{t('prizes-one-each')}</p>
      </section>
    {/if}
  </div>

  <Dialog bind:open title={t('prizes-category')} width="560px">
    {#if editing}
      <div class="form">
        <label class="wide">{t('prizes-category')}<input bind:value={editing.name} placeholder="U1800, {t('prizes-women')}, U16…" /></label>
        <label>{t('prizes-max-rating')}<input inputmode="numeric" value={editing.filter.max_rating ?? ''} oninput={(e) => (editing!.filter.max_rating = num(e.currentTarget.value))} /></label>
        <label>{t('prizes-min-rating')}<input inputmode="numeric" value={editing.filter.min_rating ?? ''} oninput={(e) => (editing!.filter.min_rating = num(e.currentTarget.value))} /></label>
        <label>{t('prizes-born-from-label')}<input inputmode="numeric" value={editing.filter.born_from ?? ''} oninput={(e) => (editing!.filter.born_from = num(e.currentTarget.value))} /></label>
        <label>{t('prizes-born-until-label')}<input inputmode="numeric" value={editing.filter.born_until ?? ''} oninput={(e) => (editing!.filter.born_until = num(e.currentTarget.value))} /></label>
        <label>{t('col-sex')}
          <select value={editing.filter.sex ?? ''} onchange={(e) => (editing!.filter.sex = (e.currentTarget.value || null) as PrizeCategory['filter']['sex'])}>
            <option value="">—</option>
            <option value="Female">{t('sex-female')}</option>
            <option value="Male">{t('sex-male')}</option>
          </select>
        </label>
        <label>{t('col-fed')}<input value={editing.filter.federation ?? ''} maxlength="3" oninput={(e) => (editing!.filter.federation = e.currentTarget.value.toUpperCase() || null)} /></label>
        <label class="check"><input type="checkbox" bind:checked={editing.filter.unrated_only} />{t('prizes-unrated')}</label>
        <label class="wide">{t('prizes-prizes-help')}<textarea rows="4" bind:value={prizesText}></textarea></label>
      </div>
    {/if}
    {#snippet footer()}
      <button onclick={() => (open = false)}>{t('action-cancel')}</button>
      <button class="primary" onclick={save} disabled={!editing?.name.trim()}>{t('action-save')}</button>
    {/snippet}
  </Dialog>
{/if}
