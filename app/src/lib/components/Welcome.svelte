<!-- First screen when no tournament is open: actions, sample data, a tour of
     the app with real screenshots, and what each part of the program does. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { importTrf, loadSample, openTournament } from '$lib/commands';
  import { t } from '$lib/i18n/index.svelte';
  import { app } from '$lib/stores/app.svelte';

  let recent = $state<string[]>([]);
  let shot = $state(0);

  const SHOTS = [
    { src: '/welcome/rounds.jpg', title: 'welcome-shot-rounds', body: 'welcome-shot-rounds-body' },
    { src: '/welcome/players.jpg', title: 'welcome-shot-players', body: 'welcome-shot-players-body' },
    { src: '/welcome/lists.jpg', title: 'welcome-shot-lists', body: 'welcome-shot-lists-body' },
    { src: '/welcome/overview.jpg', title: 'welcome-shot-overview', body: 'welcome-shot-overview-body' }
  ];

  // Icons are simple strokes on a 24px grid.
  const FEATURES: { icon: string; title: string; body: string }[] = [
    { icon: 'M4 4h11l5 5v11H4zM15 4v5h5', title: 'feat-files', body: 'feat-files-body' },
    { icon: 'M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM2 21v-1a6 6 0 0 1 12 0v1M17 8v6M14 11h6', title: 'feat-players', body: 'feat-players-body' },
    { icon: 'M3 6h18M3 12h18M3 18h18M9 6v12', title: 'feat-results', body: 'feat-results-body' },
    { icon: 'M6 9V3h12v6M6 18H4v-7h16v7h-2M8 14h8v7H8z', title: 'feat-lists', body: 'feat-lists-body' },
    { icon: 'M12 3v12M7 10l5 5 5-5M4 21h16', title: 'feat-trf', body: 'feat-trf-body' },
    { icon: 'M7 4h3v3H7zM14 4h3v3h-3zM7 17h3v3H7zM14 17h3v3h-3zM8.5 7v10M15.5 7v10', title: 'feat-pairing', body: 'feat-pairing-body' },
    { icon: 'M8 21h8M12 17v4M7 4h10v5a5 5 0 0 1-10 0zM7 6H4a3 3 0 0 0 3 3M17 6h3a3 3 0 0 1-3 3', title: 'feat-teams', body: 'feat-teams-body' },
    { icon: 'M3 12h4l3-8 4 16 3-8h4', title: 'feat-ratings', body: 'feat-ratings-body' },
    { icon: 'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18', title: 'feat-languages', body: 'feat-languages-body' }
  ];

  onMount(async () => {
    try {
      recent = await api.recentFiles();
    } catch {
      recent = [];
    }
  });

  function fileName(path: string) {
    return path.split(/[\\/]/).pop() ?? path;
  }
</script>

<div class="welcome">
  <section class="hero">
    <div class="intro">
      <img class="logo" src="/logo.png" alt="Chess Organiser logo" width="96" height="96" />
      <h1>Chess Organiser</h1>
      <p class="tagline">{t('start-tagline')}</p>
      <div class="actions">
        <button class="primary big" onclick={() => (app.newDialog = true)}>{t('new-tournament')}</button>
        <button class="big" onclick={() => openTournament()}>{t('action-open')}</button>
        <button class="big" onclick={importTrf}>{t('action-import-trf')}</button>
      </div>

      <div class="sample">
        <div>
          <strong>{t('welcome-sample-title')}</strong>
          <p>{t('welcome-sample-body')}</p>
        </div>
        <button class="primary" onclick={loadSample} disabled={app.busy}>{t('welcome-sample-load')}</button>
      </div>

      {#if recent.length}
        <div class="recent">
          <h2>{t('start-recent')}</h2>
          <ul>
            {#each recent.slice(0, 5) as path}
              <li>
                <button class="link" onclick={() => openTournament(path)}>{fileName(path)}</button>
              </li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>

    <figure class="tour">
      <div class="tabs" role="tablist" aria-label={t('welcome-tour')}>
        {#each SHOTS as s, i}
          <button role="tab" aria-selected={shot === i} class:active={shot === i} onclick={() => (shot = i)}>
            {t(s.title)}
          </button>
        {/each}
      </div>
      <div class="frame">
        <div class="chrome"><span></span><span></span><span></span></div>
        <img src={SHOTS[shot].src} alt={t(SHOTS[shot].title)} width="1600" height="1000" />
      </div>
      <figcaption>{t(SHOTS[shot].body)}</figcaption>
    </figure>
  </section>

  <section class="features">
    <h2>{t('welcome-features')}</h2>
    <div class="grid">
      {#each FEATURES as f}
        <article class="feature">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d={f.icon} /></svg>
          <div>
            <h3>
              {t(f.title)}
            </h3>
            <p>{t(f.body)}</p>
          </div>
        </article>
      {/each}
    </div>
  </section>

  <footer>
    <a href="/player-db">{t('nav-player-db')}</a>
    <a href="/settings">{t('nav-settings')}</a>
    <button class="link" onclick={() => (app.shortcuts = true)}>{t('shortcuts-title')}</button>
    <span class="muted">GPL-3.0 · {t('welcome-free')}</span>
  </footer>
</div>

<style>
  .welcome {
    height: 100%;
    overflow: auto;
    padding: 32px clamp(20px, 4vw, 56px) 24px;
    display: flex;
    flex-direction: column;
    gap: 36px;
  }
  .hero {
    display: grid;
    grid-template-columns: minmax(300px, 400px) minmax(0, 1fr);
    gap: clamp(24px, 4vw, 56px);
    align-items: start;
  }
  .intro {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .logo {
    border-radius: 22px;
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.25);
  }
  h1 {
    font-size: 30px;
    letter-spacing: -0.01em;
  }
  .tagline {
    margin: 0;
    color: var(--muted);
    font-size: 15px;
    line-height: 1.5;
  }
  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .big {
    padding: 8px 16px;
    font-size: 14px;
  }
  .sample {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px 16px;
    border-radius: 10px;
    border: 1px solid var(--accent);
    background: var(--accent-soft);
  }
  .sample p {
    margin: 4px 0 0;
    color: var(--muted);
  }
  .sample button {
    align-self: flex-start;
  }
  .recent h2 {
    font-size: 13px;
    color: var(--muted);
    font-weight: 500;
    margin-bottom: 6px;
  }
  .recent ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .tour {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
  }
  .tabs {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }
  .tabs button {
    border-radius: 16px;
    padding: 3px 12px;
  }
  .tabs button.active {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-contrast);
  }
  .frame {
    border-radius: 10px;
    overflow: hidden;
    border: 1px solid var(--border);
    box-shadow: 0 18px 50px rgb(0 0 0 / 0.22);
    background: var(--surface);
  }
  .chrome {
    display: flex;
    gap: 6px;
    padding: 8px 10px;
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
  }
  .chrome span {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--border);
  }
  .frame img {
    display: block;
    width: 100%;
    height: auto;
  }
  figcaption {
    color: var(--muted);
    line-height: 1.5;
  }
  .features h2 {
    font-size: 18px;
    margin-bottom: 14px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 12px;
  }
  .feature {
    display: flex;
    gap: 12px;
    padding: 14px 16px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--surface);
  }
  .feature svg {
    flex: none;
    width: 26px;
    height: 26px;
    fill: none;
    stroke: var(--accent);
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .feature h3 {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    font-size: 14px;
  }
  .feature p {
    margin: 4px 0 0;
    color: var(--muted);
    line-height: 1.45;
  }
  footer {
    display: flex;
    gap: 18px;
    align-items: center;
    flex-wrap: wrap;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }
  footer .muted {
    margin-inline-start: auto;
  }
  @media (max-width: 860px) {
    .hero {
      grid-template-columns: 1fr;
    }
  }
</style>
