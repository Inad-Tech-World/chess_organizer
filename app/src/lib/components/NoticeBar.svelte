<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { t } from '$lib/i18n/index.svelte';
</script>

{#if app.notice}
  <div class="notice {app.notice.kind}" role={app.notice.kind === 'error' ? 'alert' : 'status'}>
    <div class="text">
      {#if app.notice.kind === 'planned' && app.notice.phase}
        <span class="badge planned">{t('planned-badge', { phase: app.notice.phase })}</span>
      {/if}
      <span>{app.notice.message}</span>
      {#if app.notice.details?.length}
        <ul>
          {#each app.notice.details as d}<li>{d}</li>{/each}
        </ul>
      {/if}
    </div>
    <button class="close" aria-label={t('action-dismiss')} onclick={() => (app.notice = null)}>×</button>
  </div>
{/if}

<style>
  .notice {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    padding: 8px 12px;
    margin: 10px 20px 0;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface);
  }
  .text {
    flex: 1;
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    align-items: center;
  }
  ul {
    margin: 4px 0 0;
    padding-inline-start: 18px;
    width: 100%;
  }
  .error {
    border-color: var(--danger);
    background: var(--danger-soft);
  }
  .planned {
    border-color: var(--warn-border);
    background: var(--warn-soft);
  }
  .info {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .close {
    border: none;
    background: none;
    font-size: 16px;
    line-height: 1;
    padding: 0 4px;
  }
</style>
