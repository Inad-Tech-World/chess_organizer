<script lang="ts">
  import { api } from '$lib/api';
  import type { Settings, TiebreakKind, TournamentInfo } from '$lib/api/types';
  import PhaseBanner from '$lib/components/PhaseBanner.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { isTeamKind, KINDS, kindId } from '$lib/kinds';
  import { clone } from '$lib/players';
  import { app, notify, run } from '$lib/stores/app.svelte';
  import { plainSpec, tiebreakInfo, tiebreakLabel, TIEBREAKS } from '$lib/tiebreaks';

  const tour = $derived(app.view?.tournament);
  const locked = $derived((tour?.rounds.length ?? 0) > 0);

  let info = $state<TournamentInfo | null>(null);
  let settings = $state<Settings | null>(null);
  let kind = $state('swiss-dutch');
  let deputies = $state('');
  let loadedFrom: unknown = null;

  // Reload the form when the tournament changes underneath (undo, other edits).
  $effect(() => {
    if (!tour || tour === loadedFrom) return;
    loadedFrom = tour;
    info = clone(tour.info);
    settings = clone(tour.settings);
    kind = kindId(tour.kind);
    deputies = tour.info.deputy_arbiters.join(', ');
  });

  const kindDef = $derived(KINDS.find((k) => k.id === kind) ?? KINDS[0]);
  const team = $derived(isTeamKind(kindDef.kind));
  let addTb = $state<TiebreakKind>('Buchholz');
  let groupName = $state('');
  let groupRanks = $state('');

  const offered = $derived(
    TIEBREAKS.filter((tb) => (team ? tb.scope !== 'individual' : tb.scope !== 'team'))
  );

  const POINT_FIELDS: [keyof Settings['points'], string][] = [
    ['win', 'points-win'],
    ['draw', 'points-draw'],
    ['loss', 'points-loss'],
    ['forfeit_win', 'points-forfeit-win'],
    ['forfeit_loss', 'points-forfeit-loss'],
    ['pairing_bye', 'points-pab'],
    ['half_bye', 'points-half-bye'],
    ['full_bye', 'points-full-bye'],
    ['zero_bye', 'points-zero-bye']
  ];

  function moveTb(i: number, d: number) {
    if (!settings) return;
    const list = settings.tiebreaks;
    const j = i + d;
    if (j < 0 || j >= list.length) return;
    [list[i], list[j]] = [list[j], list[i]];
  }

  function setCut(i: number, value: string) {
    if (!settings) return;
    const m = settings.tiebreaks[i].modifiers;
    const n = Number(value.slice(1)) || 0;
    m.cut_lowest = n;
    m.cut_highest = value.startsWith('M') ? n : 0;
  }

  function cutValue(i: number): string {
    const m = settings!.tiebreaks[i].modifiers;
    if (m.cut_lowest === 0) return '';
    return (m.cut_lowest === m.cut_highest ? 'M' : 'C') + m.cut_lowest;
  }

  function addGroup() {
    if (!settings || !tour) return;
    const ranks = groupRanks
      .split(/[\s,;]+/)
      .map(Number)
      .filter((n) => n > 0);
    const members = tour.players.filter((p) => ranks.includes(p.starting_rank)).map((p) => p.id);
    if (members.length < 2) {
      notify({ kind: 'error', message: t('setup-group-too-small') });
      return;
    }
    settings.forbidden_groups.push({ name: groupName.trim() || t('setup-group-default'), members });
    groupName = '';
    groupRanks = '';
  }

  function memberNames(ids: number[]) {
    return ids.map((id) => tour?.players.find((p) => p.id === id)?.name ?? `#${id}`).join(', ');
  }

  async function save() {
    if (!tour || !info || !settings) return;
    info.deputy_arbiters = deputies
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean);
    const newKind = kindDef.kind;
    if (JSON.stringify(newKind) !== JSON.stringify(tour.kind)) {
      if (!(await run(() => api.changeKind(newKind)))) return;
    }
    if (JSON.stringify(info) !== JSON.stringify(tour.info)) {
      const i = info;
      if (!(await run(() => api.updateInfo(i)))) return;
    }
    if (JSON.stringify(settings) !== JSON.stringify(tour.settings)) {
      const s = settings;
      if (!(await run(() => api.updateSettings(s)))) return;
    }
    notify({ kind: 'info', message: t('setup-saved') });
  }
</script>

{#if info && settings}
  <div class="page">
    <header>
      <h1>{t('nav-setup')}</h1>
      <div class="spacer"></div>
      <button class="primary" onclick={save} disabled={app.busy}>{t('action-save')}</button>
    </header>

    <section class="card">
      <h2>{t('setup-info')}</h2>
      <div class="form">
        <label class="wide">{t('info-name')}<input bind:value={info.name} /></label>
        <label>{t('info-city')}<input bind:value={info.city} /></label>
        <label>{t('col-fed')}<input bind:value={info.federation} maxlength="3" /></label>
        <label>{t('setup-venue')}<input bind:value={info.venue} /></label>
        <label>
          {t('setup-start-date')}
          <input
            type="date"
            value={info.start_date ?? ''}
            onchange={(e) => (info!.start_date = e.currentTarget.value || null)}
          />
        </label>
        <label>
          {t('setup-end-date')}
          <input
            type="date"
            value={info.end_date ?? ''}
            onchange={(e) => (info!.end_date = e.currentTarget.value || null)}
          />
        </label>
        <label>{t('info-chief-arbiter')}<input bind:value={info.chief_arbiter} /></label>
        <label>{t('setup-deputies')}<input bind:value={deputies} /></label>
        <label>{t('setup-organizer')}<input bind:value={info.organizer} /></label>
        <label>
          {t('info-time-control')}<input bind:value={info.time_control} placeholder="90 min + 30 s" />
        </label>
      </div>
    </section>

    <section class="card">
      <h2>{t('setup-system-title')}</h2>
      <div class="form">
        <label>
          {t('setup-system')}
          <select bind:value={kind} disabled={locked}>
            {#each KINDS as k}<option value={k.id}>{t(`kind-${k.id}`)}</option>{/each}
          </select>
        </label>
        <label>
          {t('setup-rounds')}<input type="number" min="1" max="150" bind:value={settings.rounds} />
        </label>
        <label>
          {t('setup-rating-type')}
          <select bind:value={settings.rating_type}>
            <option value="Standard">{t('rating-standard')}</option>
            <option value="Rapid">{t('rating-rapid')}</option>
            <option value="Blitz">{t('rating-blitz')}</option>
          </select>
        </label>
        <label>
          {t('setup-initial-color')}
          <select bind:value={settings.initial_color}>
            <option value="White">{t('color-white')}</option>
            <option value="Black">{t('color-black')}</option>
          </select>
        </label>
        <label>
          {t('setup-acceleration')}
          <select
            value={settings.acceleration ?? ''}
            onchange={(e) =>
              (settings!.acceleration = (e.currentTarget.value || null) as Settings['acceleration'])}
          >
            <option value="">{t('setup-acceleration-none')}</option>
            <option value="Baku">{t('setup-acceleration-baku')}</option>
            <option value="Manual">{t('setup-acceleration-manual')}</option>
          </select>
        </label>
        <label class="check">
          <input type="checkbox" bind:checked={settings.fide_rated} />{t('setup-fide-rated')}
        </label>
      </div>
      {#if locked}<p class="muted">{t('setup-kind-locked')}</p>{/if}
      {#if kindDef.phase > 1}
        <PhaseBanner phase={kindDef.phase} feature={t('setup-auto-pairing', { system: t(`kind-${kind}`) })} />
      {/if}
      {#if settings.acceleration === 'Baku'}
        <PhaseBanner phase={2} feature={t('setup-acceleration-baku')} />
      {/if}
    </section>

    <section class="card">
      <h2>{t('setup-points')}</h2>
      <div class="form points">
        {#each POINT_FIELDS as [field, key]}
          <label>
            {t(key)}
            <input type="number" step="0.5" bind:value={settings.points[field]} />
          </label>
        {/each}
      </div>
    </section>

    <section class="card">
      <h2>{t('setup-tiebreaks')}</h2>
      <table class="simple">
        <thead>
          <tr>
            <th>#</th>
            <th>{t('setup-tiebreak')}</th>
            <th>{t('setup-tb-cut')}</th>
            <th>{t('setup-tb-forfeits')}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each settings.tiebreaks as tb, i}
            {@const meta = tiebreakInfo(tb.kind)}
            <tr>
              <td>{i + 1}</td>
              <td>
                <strong class="mono">{tiebreakLabel(tb)}</strong>
                <span class="muted">{t(`tb-${meta.acronym}`)}</span>
                <span class="badge planned">{t('planned-badge', { phase: meta.phase })}</span>
              </td>
              <td>
                <select value={cutValue(i)} onchange={(e) => setCut(i, e.currentTarget.value)}>
                  <option value="">—</option>
                  <option value="C1">C1</option>
                  <option value="C2">C2</option>
                  <option value="M1">M1</option>
                  <option value="M2">M2</option>
                </select>
              </td>
              <td>
                <input type="checkbox" bind:checked={tb.modifiers.forfeits_as_played} aria-label="/P" />
              </td>
              <td class="toolbar">
                <button onclick={() => moveTb(i, -1)} disabled={i === 0} aria-label={t('action-move-up')}>↑</button>
                <button
                  onclick={() => moveTb(i, 1)}
                  disabled={i === settings.tiebreaks.length - 1}
                  aria-label={t('action-move-down')}>↓</button
                >
                <button onclick={() => settings!.tiebreaks.splice(i, 1)} aria-label={t('action-remove')}>×</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      <div class="toolbar add">
        <select bind:value={addTb}>
          {#each offered as tb}
            <option value={tb.kind}>{tb.acronym} — {t(`tb-${tb.acronym}`)}</option>
          {/each}
        </select>
        <button onclick={() => settings!.tiebreaks.push(plainSpec(addTb))}>{t('setup-tb-add')}</button>
      </div>
      <p class="muted">{t('setup-tb-help')}</p>
    </section>

    <section class="card">
      <h2>{t('setup-forbidden')}</h2>
      <p class="muted">{t('setup-forbidden-help')}</p>
      <ul>
        {#each settings.forbidden_groups as g, i}
          <li>
            <strong>{g.name}</strong>: {memberNames(g.members)}
            <button class="link" onclick={() => settings!.forbidden_groups.splice(i, 1)}>
              {t('action-remove')}
            </button>
          </li>
        {/each}
      </ul>
      <div class="toolbar">
        <input placeholder={t('setup-group-name')} bind:value={groupName} />
        <input placeholder={t('setup-group-ranks')} bind:value={groupRanks} />
        <button onclick={addGroup}>{t('setup-group-add')}</button>
      </div>
    </section>

    {#if team}
      <section class="card">
        <h2>{t('setup-team')}</h2>
        <div class="form">
          <label>
            {t('setup-boards')}<input type="number" min="1" max="20" bind:value={settings.team.boards} />
          </label>
          <label>
            {t('setup-reserves')}<input type="number" min="0" max="20" bind:value={settings.team.reserves} />
          </label>
          <label>
            {t('setup-match-win')}<input type="number" step="0.5" bind:value={settings.team.match_win} />
          </label>
          <label>
            {t('setup-match-draw')}<input type="number" step="0.5" bind:value={settings.team.match_draw} />
          </label>
          <label>
            {t('setup-match-loss')}<input type="number" step="0.5" bind:value={settings.team.match_loss} />
          </label>
          <label>
            {t('setup-primary-score')}
            <select bind:value={settings.team.primary_score}>
              <option value="MatchPoints">{t('score-match-points')}</option>
              <option value="GamePoints">{t('score-game-points')}</option>
            </select>
          </label>
          <label>
            {t('setup-color-preference')}
            <select bind:value={settings.team.color_preference}>
              <option value="TypeA">{t('color-pref-a')}</option>
              <option value="TypeB">{t('color-pref-b')}</option>
              <option value="None">{t('color-pref-none')}</option>
            </select>
          </label>
        </div>
      </section>
    {/if}
  </div>
{/if}

<style>
  .points {
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  }
  .add {
    margin-top: 8px;
  }
  section p {
    margin: 8px 0 0;
  }
  td .muted,
  td .badge {
    margin-inline-start: 6px;
  }
  ul {
    margin: 8px 0;
    padding-inline-start: 18px;
  }
  .card :global(.banner) {
    margin-top: 10px;
  }
</style>
