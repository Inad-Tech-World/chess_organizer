<script lang="ts">
  import '../app.css';
  import { onMount, type Snippet } from 'svelte';
  import { page } from '$app/state';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { api } from '$lib/api';
  import NewTournamentDialog from '$lib/components/NewTournamentDialog.svelte';
  import NoticeBar from '$lib/components/NoticeBar.svelte';
  import ShortcutsDialog from '$lib/components/ShortcutsDialog.svelte';
  import { handleMenu, openTournament, redo, undo } from '$lib/commands';
  import { initLang, t } from '$lib/i18n/index.svelte';
  import { app, showError } from '$lib/stores/app.svelte';
  import { applyTheme, getTheme } from '$lib/stores/theme';

  let { children }: { children: Snippet } = $props();

  const nav = [
    { href: '/', id: 'nav-overview' },
    { href: '/setup', id: 'nav-setup' },
    { href: '/players', id: 'nav-players' },
    { href: '/rounds', id: 'nav-rounds' },
    { href: '/lists', id: 'nav-lists' },
    { href: '/teams', id: 'nav-teams' },
    { href: '/prizes', id: 'nav-prizes' },
    { href: '/ratings', id: 'nav-ratings' },
    { href: '/publish', id: 'nav-publish' }
  ];

  const t_ = $derived(app.view?.tournament);
  const roundsInfo = $derived(
    t_ ? t('status-rounds', { paired: t_.rounds.length, total: t_.settings.rounds }) : ''
  );

  async function openPending() {
    const files = await api.takePendingFiles();
    if (files.length === 0) return;
    await openTournament(files[0]);
    for (const f of files.slice(1)) await api.openWindow(f);
  }

  onMount(() => {
    initLang();
    applyTheme(getTheme());
    const unlisten: Promise<() => void>[] = [];
    (async () => {
      try {
        if (import.meta.env.DEV && !('__TAURI_INTERNALS__' in window)) {
          (await import('$lib/dev/preview')).installPreviewBackend();
        }
        unlisten.push(
          getCurrentWebviewWindow().listen<string>('menu', (e) => handleMenu(e.payload)),
          listen('files-opened', () => openPending().catch(showError))
        );
        app.view = await api.current();
        await openPending();
      } catch (e) {
        showError(e);
      }
    })();
    return () => unlisten.forEach((p) => p.then((f) => f()));
  });

  function keydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    const typing = target.closest('input, textarea, select, [contenteditable]');
    const mod = e.metaKey || e.ctrlKey;
    if (!mod || typing || e.key.toLowerCase() !== 'z') return;
    e.preventDefault();
    if (e.shiftKey) redo();
    else undo();
  }

  function isActive(href: string) {
    return href === '/' ? page.url.pathname === '/' : page.url.pathname.startsWith(href);
  }
</script>

<svelte:window onkeydown={keydown} />

<div class="shell" class:empty={!app.view}>
  {#if app.view}
    <aside>
      <div class="brand">
        <img src="/logo.png" alt="" width="24" height="24" />
        <span>Chess Organiser</span>
      </div>
      <nav aria-label={t('nav-label')}>
        {#each nav as item}
          <a href={item.href} class:active={isActive(item.href)}>{t(item.id)}</a>
        {/each}
      </nav>
      <div class="spacer"></div>
      <nav class="secondary">
        <a href="/player-db" class:active={isActive('/player-db')}>{t('nav-player-db')}</a>
        <a href="/settings" class:active={isActive('/settings')}>{t('nav-settings')}</a>
      </nav>
    </aside>
  {/if}
  <main>
    {#if app.view}
      <div class="topbar">
        <div class="title">
          <strong>{app.view.tournament.info.name || t('untitled')}</strong>
          <span class="muted">{roundsInfo}</span>
        </div>
        <div class="toolbar">
          <button
            onclick={undo}
            disabled={!app.view.can_undo || app.busy}
            title={app.view.undo_label ? t('undo-what', { what: app.view.undo_label }) : t('action-undo')}
          >
            ↶ {t('action-undo')}
          </button>
          <button onclick={redo} disabled={!app.view.can_redo || app.busy}>↷ {t('action-redo')}</button>
        </div>
      </div>
    {/if}
    <NoticeBar />
    <div class="content">
      {#if app.view || ['/', '/settings', '/player-db'].includes(page.url.pathname)}
        {@render children()}
      {:else}
        <div class="page">
          <p class="muted">{t('no-tournament-hint')}</p>
          <a href="/">{t('nav-overview')}</a>
        </div>
      {/if}
    </div>
  </main>
</div>

<NewTournamentDialog />
<ShortcutsDialog />

<style>
  .shell {
    display: grid;
    grid-template-columns: 190px 1fr;
    height: 100vh;
  }
  .shell.empty {
    grid-template-columns: 1fr;
  }
  aside {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px 8px;
    border-inline-end: 1px solid var(--border);
    background: var(--surface-2);
    min-height: 0;
  }
  .brand {
    display: flex;
    gap: 8px;
    align-items: center;
    font-weight: 600;
    padding: 4px 8px 12px;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  nav a {
    padding: 6px 10px;
    border-radius: 6px;
    color: var(--text);
    text-decoration: none;
  }
  nav a:hover {
    background: var(--surface);
  }
  nav a.active {
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .spacer {
    flex: 1;
  }
  main {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  .topbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 20px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .title {
    flex: 1;
    display: flex;
    gap: 12px;
    align-items: baseline;
    min-width: 0;
  }
  .title strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .content {
    flex: 1;
    min-height: 0;
  }
</style>
