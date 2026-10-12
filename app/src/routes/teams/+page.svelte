<script lang="ts">
  import { ask } from '@tauri-apps/plugin-dialog';
  import { api } from '$lib/api';
  import type { Team } from '$lib/api/types';
  import Dialog from '$lib/components/Dialog.svelte';
  import PhaseBanner from '$lib/components/PhaseBanner.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { isTeamKind } from '$lib/kinds';
  import { clone } from '$lib/players';
  import { app, run } from '$lib/stores/app.svelte';

  const tour = $derived(app.view?.tournament);
  let editing = $state<Team | null>(null);
  let open = $state(false);

  function blank(): Team {
    return { id: 0, name: '', short_name: null, federation: null, members: [], captain: null };
  }

  function edit(team: Team | null) {
    editing = team ? clone(team) : blank();
    open = true;
  }

  function toggle(id: number) {
    if (!editing) return;
    editing.members = editing.members.includes(id)
      ? editing.members.filter((m) => m !== id)
      : [...editing.members, id];
  }

  async function save() {
    const team = editing;
    if (!team || !team.name.trim()) return;
    if (await run(() => (team.id === 0 ? api.addTeam(team) : api.updateTeam(team)))) open = false;
  }

  async function remove(team: Team) {
    if (await ask(t('teams-remove-confirm', { name: team.name }), { kind: 'warning' })) {
      await run(() => api.removeTeam(team.id));
    }
  }

  function playerName(id: number) {
    return tour?.players.find((p) => p.id === id)?.name ?? `#${id}`;
  }
</script>

{#if tour}
  <div class="page">
    <header>
      <h1>{t('nav-teams')}</h1>
      <div class="spacer"></div>
      <button class="primary" onclick={() => edit(null)}>{t('teams-add')}</button>
    </header>
    <PhaseBanner phase={3} feature={t('teams-feature')} />
    {#if !isTeamKind(tour.kind)}
      <p class="muted">{t('teams-individual-note')}</p>
    {/if}
    <section class="card">
      <table class="simple">
        <thead>
          <tr><th>#</th><th>{t('teams-name')}</th><th>{t('col-fed')}</th><th>{t('teams-members')}</th><th></th></tr>
        </thead>
        <tbody>
          {#each tour.teams as team}
            <tr>
              <td>{team.id}</td>
              <td>{team.name}</td>
              <td>{team.federation ?? ''}</td>
              <td>{team.members.map(playerName).join(', ')}</td>
              <td class="toolbar">
                <button onclick={() => edit(team)}>{t('action-edit')}</button>
                <button onclick={() => remove(team)}>{t('action-remove')}</button>
              </td>
            </tr>
          {:else}
            <tr><td colspan="5" class="muted">{t('teams-empty')}</td></tr>
          {/each}
        </tbody>
      </table>
    </section>
  </div>

  <Dialog bind:open title={editing?.id ? t('teams-edit') : t('teams-add')} width="560px">
    {#if editing}
      <div class="form">
        <label class="wide">{t('teams-name')}<input bind:value={editing.name} /></label>
        <label>{t('col-fed')}
          <input value={editing.federation ?? ''} maxlength="3"
            oninput={(e) => (editing!.federation = e.currentTarget.value.toUpperCase() || null)} />
        </label>
        <label>{t('teams-captain')}
          <input value={editing.captain ?? ''} oninput={(e) => (editing!.captain = e.currentTarget.value || null)} />
        </label>
      </div>
      <h3 class="members">{t('teams-board-order')}</h3>
      <ol>
        {#each editing.members as id}<li>{playerName(id)}</li>{/each}
      </ol>
      <div class="pick">
        {#each tour.players as p}
          <label class="check"><input type="checkbox" checked={editing.members.includes(p.id)} onchange={() => toggle(p.id)} />{p.name}</label>
        {/each}
      </div>
    {/if}
    {#snippet footer()}
      <button onclick={() => (open = false)}>{t('action-cancel')}</button>
      <button class="primary" onclick={save} disabled={!editing?.name.trim()}>{t('action-save')}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  p {
    margin: 0;
  }
  .members {
    margin-top: 12px;
  }
  .pick {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 4px;
    max-height: 220px;
    overflow: auto;
    margin-top: 8px;
  }
</style>
