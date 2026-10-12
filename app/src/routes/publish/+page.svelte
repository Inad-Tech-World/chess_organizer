<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { api } from '$lib/api';
  import { t } from '$lib/i18n/index.svelte';
  import { app, notify, run } from '$lib/stores/app.svelte';

  const KEY = 'publish-server';
  let server = $state(localStorage.getItem(KEY) ?? '');
  // The token is kept for this session only, never written to disk.
  let token = $state(sessionStorage.getItem('publish-token') ?? '');
  let url = $state<string | null>(null);

  async function upload() {
    localStorage.setItem(KEY, server.trim());
    sessionStorage.setItem('publish-token', token);
    const r = await run(() => api.publishUpload(server, token));
    if (r) {
      url = r;
      notify({ kind: 'info', message: t('publish-done') });
    }
  }
</script>

<div class="page">
  <header><h1>{t('nav-publish')}</h1></header>
  <section class="card">
    <p class="muted">{t('publish-help')}</p>
    <div class="form">
      <label>{t('publish-server')}<input bind:value={server} placeholder="https://results.example.org" /></label>
      <label>{t('publish-token')}<input type="password" bind:value={token} autocomplete="off" /></label>
    </div>
    <div class="toolbar actions">
      <button class="primary" onclick={upload} disabled={!app.view || !server.trim() || !token.trim()}>
        {t('publish-upload')}
      </button>
    </div>
    {#if url}
      <p class="link">
        {t('publish-live-at')}
        <button class="linklike" onclick={() => openUrl(url!)}>{url}</button>
      </p>
    {/if}
  </section>
  <section class="card">
    <h2>{t('publish-server-title')}</h2>
    <p class="muted">{t('publish-server-help')}</p>
    <pre>CO_PUBLISH_TOKEN=… CO_PUBLISH_DATA=/var/lib/co-publish co-publish</pre>
  </section>
</div>

<style>
  p {
    margin: 0 0 10px;
  }
  .actions {
    margin-top: 12px;
  }
  .link {
    margin-top: 12px;
  }
  .linklike {
    border: none;
    background: none;
    color: var(--link);
    text-decoration: underline;
    padding: 0;
  }
  pre {
    background: var(--selected);
    padding: 8px 10px;
    border-radius: 6px;
    overflow: auto;
  }
</style>
