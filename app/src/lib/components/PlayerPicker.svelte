<!-- Search the local player database (FIDE and national lists). -->
<script lang="ts">
  import { api } from '$lib/api';
  import type { PlayerRecord } from '$lib/api/types';
  import { t } from '$lib/i18n/index.svelte';
  import { showError } from '$lib/stores/app.svelte';

  let { onpick }: { onpick: (r: PlayerRecord) => void } = $props();

  let query = $state('');
  let results = $state<PlayerRecord[]>([]);
  let active = $state(0);
  let timer: ReturnType<typeof setTimeout> | undefined;

  function search() {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      if (query.trim().length < 2) {
        results = [];
        return;
      }
      try {
        results = await api.playerDbSearch(query, 20);
        active = 0;
      } catch (e) {
        showError(e);
      }
    }, 150);
  }

  function keydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') active = Math.min(results.length - 1, active + 1);
    else if (e.key === 'ArrowUp') active = Math.max(0, active - 1);
    else if (e.key === 'Enter' && results[active]) onpick(results[active]);
    else return;
    e.preventDefault();
  }
</script>

<div class="picker">
  <input
    type="search"
    placeholder={t('player-db-search-placeholder')}
    bind:value={query}
    oninput={search}
    onkeydown={keydown}
    aria-label={t('player-db-search-placeholder')}
  />
  {#if results.length}
    <ul role="listbox">
      {#each results as r, i}
        <li role="option" aria-selected={i === active}>
          <button class:active={i === active} onclick={() => onpick(r)}>
            <span class="name">{r.title ? `${r.title} ` : ''}{r.name}</span>
            <span class="muted">{r.federation ?? ''}</span>
            <span class="muted">{r.rating_standard ?? r.national_rating ?? ''}</span>
            <span class="muted mono">{r.fide_id ?? r.national_id ?? ''}</span>
          </button>
        </li>
      {/each}
    </ul>
  {:else if query.trim().length >= 2}
    <p class="muted">{t('player-db-no-results')}</p>
  {/if}
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    max-height: 220px;
    overflow: auto;
  }
  li button {
    display: grid;
    grid-template-columns: 1fr 48px 52px 90px;
    gap: 8px;
    width: 100%;
    border: none;
    border-radius: 0;
    text-align: start;
    background: none;
  }
  li button.active {
    background: var(--selected);
  }
  p {
    margin: 0;
  }
</style>
