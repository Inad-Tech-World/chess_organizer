<!--
  Virtualised, keyboard-driven table for lists of thousands of players.
  Arrow keys / PageUp / PageDown / Home / End move the selection, Enter
  activates the row, and `onkey` lets a page claim other keys (e.g. result
  entry on the pairings screen).
-->
<script lang="ts" generics="T">
  import type { Column } from './grid';

  let {
    rows,
    columns,
    selected = $bindable(0),
    rowHeight = 28,
    label,
    empty = '',
    onactivate,
    onkey,
    rowClass,
    ltr = false
  }: {
    rows: T[];
    columns: Column<T>[];
    selected?: number;
    rowHeight?: number;
    label: string;
    empty?: string;
    onactivate?: (row: T, index: number) => void;
    /** Return true if the key was handled. */
    onkey?: (e: KeyboardEvent, row: T | undefined, index: number) => boolean;
    rowClass?: (row: T, index: number) => string;
    /** Keep left-to-right even in RTL languages (pairings: White left, Black right). */
    ltr?: boolean;
  } = $props();

  let viewport: HTMLDivElement;
  let scrollTop = $state(0);
  let height = $state(400);
  const overscan = 10;

  const template = $derived(columns.map((c) => c.width ?? '1fr').join(' '));
  const start = $derived(Math.max(0, Math.floor(scrollTop / rowHeight) - overscan));
  const end = $derived(Math.min(rows.length, Math.ceil((scrollTop + height) / rowHeight) + overscan));
  const visible = $derived(rows.slice(start, end));

  $effect(() => {
    if (selected >= rows.length && rows.length > 0) selected = rows.length - 1;
  });

  export function focus() {
    viewport?.focus();
  }

  function select(i: number) {
    if (rows.length === 0) return;
    selected = Math.max(0, Math.min(rows.length - 1, i));
    const top = selected * rowHeight;
    const body = height - rowHeight; // minus the sticky header
    if (top < viewport.scrollTop) viewport.scrollTop = top;
    else if (top + rowHeight > viewport.scrollTop + body) viewport.scrollTop = top + rowHeight - body;
  }

  function keydown(e: KeyboardEvent) {
    if (onkey?.(e, rows[selected], selected)) {
      e.preventDefault();
      return;
    }
    const page = Math.max(1, Math.floor(height / rowHeight) - 2);
    switch (e.key) {
      case 'ArrowDown':
        select(selected + 1);
        break;
      case 'ArrowUp':
        select(selected - 1);
        break;
      case 'PageDown':
        select(selected + page);
        break;
      case 'PageUp':
        select(selected - page);
        break;
      case 'Home':
        select(0);
        break;
      case 'End':
        select(rows.length - 1);
        break;
      case 'Enter':
        if (rows[selected] !== undefined) onactivate?.(rows[selected], selected);
        break;
      default:
        return;
    }
    e.preventDefault();
  }

  /** Moves the selection down one row (used after entering a result). */
  export function advance() {
    select(selected + 1);
  }
</script>

<div
  class="grid"
  role="grid"
  aria-label={label}
  aria-rowcount={rows.length}
  dir={ltr ? 'ltr' : undefined}
  tabindex="0"
  bind:this={viewport}
  bind:clientHeight={height}
  onscroll={() => (scrollTop = viewport.scrollTop)}
  onkeydown={keydown}
>
  <div class="head" role="row" style:grid-template-columns={template}>
    {#each columns as c}
      <div role="columnheader" class={c.align ?? 'left'}>{c.label}</div>
    {/each}
  </div>
  {#if rows.length === 0}
    <div class="empty muted">{empty}</div>
  {:else}
    <div class="body" style:height="{rows.length * rowHeight}px">
      {#each visible as row, i (start + i)}
        {@const index = start + i}
        <!-- Keyboard input is handled by the grid itself (arrow keys, Enter). -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          role="row"
          tabindex="-1"
          aria-rowindex={index + 1}
          aria-selected={index === selected}
          class="row {rowClass?.(row, index) ?? ''}"
          class:selected={index === selected}
          class:alt={index % 2 === 1}
          style:top="{index * rowHeight}px"
          style:height="{rowHeight}px"
          style:grid-template-columns={template}
          onclick={() => select(index)}
          ondblclick={() => onactivate?.(row, index)}
        >
          {#each columns as c}
            <!-- Numbers and results stay LTR so "1½" and "+/-" never reorder in RTL. -->
            <div role="gridcell" class={c.align ?? 'left'} dir={c.align && c.align !== 'left' ? 'ltr' : undefined}>
              {c.value(row, index) ?? ''}
            </div>
          {/each}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .grid {
    position: relative;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    min-height: 120px;
    flex: 1;
    font-variant-numeric: tabular-nums;
  }
  .grid:focus-visible {
    outline: 2px solid var(--focus);
    outline-offset: -1px;
  }
  .head {
    position: sticky;
    top: 0;
    z-index: 1;
    display: grid;
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    color: var(--muted);
    min-width: max-content;
  }
  .head > div,
  .row > div {
    padding: 0 8px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .head > div {
    padding-block: 5px;
  }
  .body {
    position: relative;
    min-width: max-content;
  }
  .row {
    position: absolute;
    inset-inline: 0;
    display: grid;
    align-items: center;
    cursor: default;
  }
  .row.alt {
    background: var(--row-alt);
  }
  .row.selected {
    background: var(--selected);
  }
  .row:global(.dim) {
    color: var(--muted);
  }
  .right {
    text-align: end;
  }
  .center {
    text-align: center;
  }
  .empty {
    padding: 24px;
    text-align: center;
  }
</style>
