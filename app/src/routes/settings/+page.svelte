<script lang="ts">
  import { i18n, LANGUAGES, setLang, t } from '$lib/i18n/index.svelte';
  import { applyTheme, getTheme, type Theme } from '$lib/stores/theme';

  let theme = $state<Theme>(getTheme());
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
</div>

<style>
  p {
    margin: 10px 0 0;
  }
</style>
