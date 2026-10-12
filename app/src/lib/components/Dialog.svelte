<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    open = $bindable(false),
    title,
    width = '480px',
    children,
    footer,
    onclose
  }: {
    open: boolean;
    title: string;
    width?: string;
    children: Snippet;
    footer?: Snippet;
    onclose?: () => void;
  } = $props();

  let el: HTMLDialogElement;

  $effect(() => {
    if (open && !el.open) el.showModal();
    else if (!open && el.open) el.close();
  });
</script>

<dialog
  bind:this={el}
  style:width
  onclose={() => {
    open = false;
    onclose?.();
  }}
>
  <header><h2>{title}</h2></header>
  <div class="body">{@render children()}</div>
  {#if footer}
    <footer class="toolbar">{@render footer()}</footer>
  {/if}
</dialog>

<style>
  dialog {
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 0;
    background: var(--surface);
    color: var(--text);
    max-width: calc(100vw - 32px);
    max-height: calc(100vh - 48px);
    box-shadow: 0 12px 40px rgb(0 0 0 / 0.25);
  }
  dialog::backdrop {
    background: rgb(0 0 0 / 0.35);
  }
  header {
    padding: 14px 18px 0;
  }
  .body {
    padding: 12px 18px;
    overflow: auto;
    max-height: calc(100vh - 180px);
  }
  footer {
    justify-content: flex-end;
    padding: 10px 18px 14px;
    border-top: 1px solid var(--border);
  }
</style>
