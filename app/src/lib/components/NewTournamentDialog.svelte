<script lang="ts">
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import type { TournamentKind } from '$lib/api/types';
  import Dialog from '$lib/components/Dialog.svelte';
  import { COT, pickSave, suggestName } from '$lib/files';
  import { t } from '$lib/i18n/index.svelte';
  import { app, run } from '$lib/stores/app.svelte';
  import { KINDS, kindKey } from '$lib/kinds';

  let name = $state('');
  let kindId = $state('swiss-dutch');
  let rounds = $state(7);

  async function create() {
    const kind: TournamentKind = KINDS.find((k) => k.id === kindId)!.kind;
    const path = await pickSave(COT, suggestName(name, 'cot'));
    if (!path) return;
    const view = await run(async () => {
      const d = await api.defaults();
      return api.newTournament({
        path,
        info: { ...d.info, name: name.trim() },
        settings: { ...d.settings, rounds },
        kind
      });
    });
    if (view) {
      app.newDialog = false;
      name = '';
      goto('/setup');
    }
  }
</script>

<Dialog bind:open={app.newDialog} title={t('new-tournament')}>
  <div class="form">
    <label class="wide">
      {t('info-name')}
      <input bind:value={name} placeholder={t('info-name-placeholder')} />
    </label>
    <label>
      {t('setup-system')}
      <select bind:value={kindId}>
        {#each KINDS as k}<option value={k.id}>{t(kindKey(k.kind))}</option>{/each}
      </select>
    </label>
    <label>
      {t('setup-rounds')}
      <input type="number" min="1" max="150" bind:value={rounds} />
    </label>
  </div>
  {#snippet footer()}
    <button onclick={() => (app.newDialog = false)}>{t('action-cancel')}</button>
    <button class="primary" disabled={!name.trim() || rounds < 1} onclick={create}>
      {t('action-create')}
    </button>
  {/snippet}
</Dialog>
