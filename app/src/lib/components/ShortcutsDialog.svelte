<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { app } from '$lib/stores/app.svelte';

  const mac = navigator.platform.toLowerCase().includes('mac');
  const mod = mac ? '⌘' : 'Ctrl+';
  const alt = mac ? '⌥' : 'Alt+';
  const rows: [string, string][] = [
    ['1', 'shortcut-white-wins'],
    ['0', 'shortcut-black-wins'],
    ['5  =', 'shortcut-draw'],
    ['+', 'shortcut-white-forfeit'],
    ['-', 'shortcut-black-forfeit'],
    ['*', 'shortcut-double-forfeit'],
    ['Delete', 'shortcut-clear-result'],
    ['↑ ↓  PgUp PgDn', 'shortcut-move'],
    ['Enter', 'shortcut-edit'],
    [`${mod}${alt}Z`, 'shortcut-undo'],
    [`${mod}${alt}⇧Z`, 'shortcut-redo'],
    [`${mod}⇧P`, 'shortcut-pair'],
    [`${mod}1 … ${mod}7`, 'shortcut-sections']
  ];
</script>

<Dialog bind:open={app.shortcuts} title={t('shortcuts-title')} width="440px">
  <table class="simple">
    <tbody>
      {#each rows as [keys, id]}
        <tr><td class="mono">{keys}</td><td>{t(id)}</td></tr>
      {/each}
    </tbody>
  </table>
  {#snippet footer()}
    <button class="primary" onclick={() => (app.shortcuts = false)}>{t('action-close')}</button>
  {/snippet}
</Dialog>
