<script lang="ts">
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import type { NormReport, RatingChangeRow } from '$lib/api/types';
  import { exportTrf } from '$lib/commands';
  import { t } from '$lib/i18n/index.svelte';
  import { app, run } from '$lib/stores/app.svelte';

  const tour = $derived(app.view?.tournament);
  let player = $state<number | null>(null);
  let changes = $state<RatingChangeRow[] | null>(null);
  let norms = $state<NormReport[] | null>(null);

  const byId = $derived(new Map((tour?.players ?? []).map((p) => [p.id, p])));

  async function calculate() {
    const r = await run(() => api.ratingReport());
    if (r) changes = r;
  }

  async function checkNorms() {
    if (player === null) return;
    const id = player;
    const r = await run(() => api.normCheck(id));
    if (r) norms = r;
  }

  function sign(n: number) {
    return (n > 0 ? '+' : '') + n.toFixed(1);
  }
</script>

{#if tour}
  <div class="page">
    <header>
      <h1>{t('nav-ratings')}</h1>
      <div class="spacer"></div>
      <button onclick={() => goto('/lists?list=RatingReport')}>{t('list-rating-report')}</button>
    </header>
    <section class="card">
      <h2>{t('ratings-trf-title')}</h2>
      <p class="muted">{t('ratings-trf-help')}</p>
      <button class="primary" onclick={exportTrf}>{t('action-export-trf')}</button>
    </section>
    <section class="card">
      <h2>{t('ratings-changes-title')}</h2>
      <p class="muted">{t('ratings-changes-help')}</p>
      <button onclick={calculate} disabled={tour.rounds.length === 0}>{t('ratings-calculate')}</button>
      {#if changes}
        <table class="simple">
          <thead>
            <tr>
              <th>{t('col-name')}</th><th>{t('col-rating')}</th><th>{t('col-k')}</th><th>{t('col-games')}</th>
              <th>{t('col-score')}</th><th>{t('col-expected')}</th><th>{t('col-change')}</th>
              <th>{t('col-aro')}</th><th>{t('col-performance')}</th>
            </tr>
          </thead>
          <tbody dir="ltr">
            {#each changes as c}
              <tr>
                <td>{byId.get(c.player)?.name ?? c.player}</td>
                <td>{c.rating ?? ''}</td>
                <td>{c.k ?? ''}</td>
                <td>{c.games}</td>
                <td>{c.score}</td>
                <td>{c.rating !== null ? c.expected.toFixed(2) : ''}</td>
                <td class:up={c.change > 0} class:down={c.change < 0}>
                  {c.rating !== null ? sign(c.change) : c.initial_rating_estimate !== null ? t('ratings-first-rating', { rating: c.initial_rating_estimate }) : ''}
                </td>
                <td>{c.average_opponent_rating ?? ''}</td>
                <td>{c.performance ?? ''}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>
    <section class="card">
      <h2>{t('ratings-norms-title')}</h2>
      <p class="muted">{t('ratings-norms-help')}</p>
      <div class="toolbar">
        <select bind:value={player}>
          <option value={null}>—</option>
          {#each tour.players as p}<option value={p.id}>{p.name}</option>{/each}
        </select>
        <button disabled={player === null} onclick={checkNorms}>{t('ratings-check-norms')}</button>
      </div>
      {#if norms}
        {#if norms.length === 0}
          <p class="muted">{t('ratings-norms-none')}</p>
        {/if}
        <ul class="norms">
          {#each norms as n}
            <li class:ok={n.achieved}>
              <strong>{n.title}</strong>
              {n.achieved ? t('ratings-norm-achieved') : t('ratings-norm-missed')}
              · {t('ratings-norm-stats', { games: n.games, aro: n.average_opponent_rating, perf: n.performance })}
              {#if n.shortfalls.length}
                <ul>{#each n.shortfalls as s}<li class="muted">{s}</li>{/each}</ul>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  </div>
{/if}

<style>
  section p {
    margin: 0 0 10px;
  }
  table {
    margin-top: 12px;
  }
  .up {
    color: var(--ok, #2e7d32);
  }
  .down {
    color: var(--danger);
  }
  .norms {
    padding-inline-start: 18px;
  }
  .norms li.ok strong {
    color: var(--ok, #2e7d32);
  }
</style>
