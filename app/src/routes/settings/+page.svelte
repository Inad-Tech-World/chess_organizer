<script lang="ts">
  import { i18n, LANGUAGES, setLang, t } from '$lib/i18n/index.svelte';
  import { api } from '$lib/api';
  import { applyTheme, getTheme, type Theme } from '$lib/stores/theme';
  import { run } from '$lib/stores/app.svelte';

  let theme = $state<Theme>(getTheme());
  let update = $state<{ enabled: boolean; current: string; available: string | null } | null>(null);

  async function check() {
    const r = await run(() => api.updateCheck());
    if (r) update = r;
  }
</script>

<div class="page">
  <header><h1>{t('nav-settings')}</h1></header>
  <section class="card">
    <div class="form">
      <label>
        {t('settings-language')}
        <select value={i18n.lang} onchange={(e) => setLang(e.currentTarget.value)}>
          {#each LANGUAGES as l}<option value={l.code}>{l.name}</option>{/each}
        </select>
      </label>
      <label>
        {t('settings-theme')}
        <select bind:value={theme} onchange={() => applyTheme(theme)}>
          <option value="system">{t('theme-system')}</option>
          <option value="light">{t('theme-light')}</option>
          <option value="dark">{t('theme-dark')}</option>
        </select>
      </label>
    </div>
    <p class="muted">{t('settings-translations-note')}</p>
  </section>
  <section class="card">
    <h2>{t('settings-updates')}</h2>
    <div class="toolbar">
      <button onclick={check}>{t('settings-check-updates')}</button>
      {#if update?.available}
        <button class="primary" onclick={() => run(() => api.updateInstall())}>
          {t('settings-install-update', { version: update.available })}
        </button>
      {/if}
    </div>
    {#if update}
      <p class="muted">
        {#if !update.enabled}
          {t('settings-updates-disabled', { version: update.current })}
        {:else if !update.available}
          {t('settings-up-to-date', { version: update.current })}
        {/if}
      </p>
    {/if}
  </section>
</div>

<style>
  p {
    margin: 10px 0 0;
  }
</style>
