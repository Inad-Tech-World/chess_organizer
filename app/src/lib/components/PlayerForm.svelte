<script lang="ts">
  import type { Player, Sex, Title } from '$lib/api/types';
  import { t } from '$lib/i18n/index.svelte';

  let { player = $bindable() }: { player: Player } = $props();

  const TITLES: Title[] = ['GM', 'IM', 'WGM', 'FM', 'WIM', 'CM', 'WFM', 'WCM'];

  function num(v: string): number | null {
    const n = parseInt(v, 10);
    return Number.isFinite(n) && n > 0 ? n : null;
  }
  function text(v: string): string | null {
    const s = v.trim();
    return s === '' ? null : s;
  }
</script>

<div class="form">
  <label class="wide">
    {t('player-name')}
    <input bind:value={player.name} placeholder={t('player-name-placeholder')} required />
  </label>
  <label>
    {t('col-title')}
    <select
      value={player.title ?? ''}
      onchange={(e) => (player.title = (e.currentTarget.value || null) as Title | null)}
    >
      <option value="">—</option>
      {#each TITLES as tl}<option value={tl}>{tl}</option>{/each}
    </select>
  </label>
  <label>
    {t('col-sex')}
    <select
      value={player.sex ?? ''}
      onchange={(e) => (player.sex = (e.currentTarget.value || null) as Sex | null)}
    >
      <option value="">—</option>
      <option value="Male">{t('sex-male')}</option>
      <option value="Female">{t('sex-female')}</option>
    </select>
  </label>
  <label>
    {t('col-fed')}
    <input
      value={player.federation ?? ''}
      maxlength="3"
      oninput={(e) => (player.federation = text(e.currentTarget.value.toUpperCase()))}
    />
  </label>
  <label>
    {t('col-fide-id')}
    <input
      inputmode="numeric"
      value={player.fide_id ?? ''}
      oninput={(e) => (player.fide_id = num(e.currentTarget.value))}
    />
  </label>
  <label>
    {t('player-national-id')}
    <input
      value={player.national_id ?? ''}
      oninput={(e) => (player.national_id = text(e.currentTarget.value))}
    />
  </label>
  <label>
    {t('col-birth')}
    <input
      value={player.birth_date ?? ''}
      placeholder="YYYY or YYYY/MM/DD"
      oninput={(e) => (player.birth_date = text(e.currentTarget.value))}
    />
  </label>
  <label>
    {t('col-club')}
    <input value={player.club ?? ''} oninput={(e) => (player.club = text(e.currentTarget.value))} />
  </label>
  <label>
    {t('rating-standard')}
    <input
      inputmode="numeric"
      value={player.ratings.fide_standard ?? ''}
      oninput={(e) => (player.ratings.fide_standard = num(e.currentTarget.value))}
    />
  </label>
  <label>
    {t('rating-rapid')}
    <input
      inputmode="numeric"
      value={player.ratings.fide_rapid ?? ''}
      oninput={(e) => (player.ratings.fide_rapid = num(e.currentTarget.value))}
    />
  </label>
  <label>
    {t('rating-blitz')}
    <input
      inputmode="numeric"
      value={player.ratings.fide_blitz ?? ''}
      oninput={(e) => (player.ratings.fide_blitz = num(e.currentTarget.value))}
    />
  </label>
  <label>
    {t('rating-national')}
    <input
      inputmode="numeric"
      value={player.ratings.national ?? ''}
      oninput={(e) => (player.ratings.national = num(e.currentTarget.value))}
    />
  </label>
  <label>
    {t('player-k-factor')}
    <input
      inputmode="numeric"
      value={player.k_factor ?? ''}
      oninput={(e) => (player.k_factor = num(e.currentTarget.value))}
    />
  </label>
</div>
