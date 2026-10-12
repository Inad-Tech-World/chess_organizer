<script lang="ts">
  import { api } from '$lib/api';
  import PhaseBanner from '$lib/components/PhaseBanner.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { run } from '$lib/stores/app.svelte';

  let server = $state('');
  let token = $state('');
</script>

<div class="page">
  <header><h1>{t('nav-publish')}</h1></header>
  <PhaseBanner phase={5} feature={t('publish-feature')} />
  <section class="card">
    <p class="muted">{t('publish-help')}</p>
    <div class="form">
      <label>{t('publish-server')}<input bind:value={server} placeholder="https://results.example.org" /></label>
      <label>{t('publish-token')}<input type="password" bind:value={token} /></label>
    </div>
    <div class="toolbar actions">
      <button class="primary" onclick={() => run(() => api.publishUpload(server, token))}>{t('publish-upload')}</button>
    </div>
  </section>
</div>

<style>
  p {
    margin: 0 0 10px;
  }
  .actions {
    margin-top: 12px;
  }
</style>
