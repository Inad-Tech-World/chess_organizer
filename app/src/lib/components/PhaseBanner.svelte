<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';

  let { phase, feature }: { phase: number; feature: string } = $props();
</script>

<div class="banner" role="note">
  <span class="badge planned">{t('planned-badge', { phase })}</span>
  <span><strong>{feature}</strong> — {t('planned-body', { phase })}</span>
</div>

<style>
  .banner {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 8px 12px;
    border: 1px solid var(--warn-border);
    background: var(--warn-soft);
    border-radius: 8px;
  }
</style>
