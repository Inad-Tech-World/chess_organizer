// Actions shared by the menu bar, toolbar buttons and the start screen.

import { goto } from '$app/navigation';
import { ask } from '@tauri-apps/plugin-dialog';
import { api } from '$lib/api';
import { COT, EXPORTED, pickOpen, pickSave, suggestName, TRF } from '$lib/files';
import type { ImportResult } from '$lib/api/types';
import { t } from '$lib/i18n/index.svelte';
import { app, notify, run } from '$lib/stores/app.svelte';

export async function openTournament(path?: string | null) {
  path ??= await pickOpen(COT);
  if (!path) return;
  if (app.view) {
    // This window already shows a tournament: open the new one beside it.
    await run(() => api.openWindow(path));
    return;
  }
  if (await run(() => api.open(path))) goto('/players');
}

/** Loads a fresh copy of the sample tournament (and sample player list). */
export async function loadSample() {
  if (app.view && !(await ask(t('sample-replace-confirm'), { kind: 'info' }))) return;
  if (await run(() => api.sampleLoad())) goto('/rounds');
}

export async function openInNewWindow() {
  const path = await pickOpen(COT);
  if (path) await run(() => api.openWindow(path));
}

export async function importTrf() {
  return importFile(TRF, (src, cot) => api.importTrf(src, cot));
}

/** Files exported by other pairing programs: TRF or a CSV/TXT player list. */
export async function importOther() {
  return importFile(EXPORTED, (src, cot) => api.importOther(src, cot));
}

async function importFile(
  filters: typeof TRF,
  call: (src: string, cot: string) => Promise<ImportResult>
) {
  if (app.view) {
    notify({ kind: 'info', message: t('import-new-window-hint') });
    return;
  }
  const src = await pickOpen(filters);
  if (!src) return;
  const base = src.replace(/\.[^./\\]+$/, '');
  const cot = await pickSave(COT, `${base}.cot`);
  if (!cot) return;
  const result = await run(() => call(src, cot));
  if (result) {
    app.view = result.view;
    if (result.warnings.length) {
      notify({ kind: 'info', message: t('import-warnings'), details: result.warnings });
    }
    goto('/players');
  }
}

export async function exportTrf() {
  if (!requireTournament()) return;
  const path = await pickSave(TRF, suggestName(app.view!.tournament.info.name, 'trf'));
  if (path && (await run(() => api.exportTrf(path))) !== undefined) {
    notify({ kind: 'info', message: t('export-done', { path }) });
  }
}

export async function saveCopy() {
  if (!requireTournament()) return;
  const path = await pickSave(COT, suggestName(`${app.view!.tournament.info.name} copy`, 'cot'));
  if (path && (await run(() => api.saveAs(path))) !== undefined) {
    notify({ kind: 'info', message: t('export-done', { path }) });
  }
}

export async function closeTournament() {
  if (!app.view) return;
  await run(() => api.close());
  app.view = null;
  goto('/');
}

export async function undo() {
  if (app.view?.can_undo) await run(() => api.undo());
}

export async function redo() {
  if (app.view?.can_redo) await run(() => api.redo());
}

export async function pairNextRound() {
  if (!requireTournament()) return;
  if (await run(() => api.pairNext())) goto('/rounds');
}

export async function deleteLastRound() {
  if (!requireTournament()) return;
  const n = app.view!.tournament.rounds.length;
  if (n === 0) return;
  if (await ask(t('rounds-delete-confirm', { round: n }), { kind: 'warning' })) {
    await run(() => api.deleteLastRound());
  }
}

export function requireTournament(): boolean {
  if (app.view) return true;
  notify({ kind: 'info', message: t('no-tournament-hint') });
  return false;
}

const NAV: Record<string, string> = {
  'nav.players': '/players',
  'nav.player-db': '/player-db',
  'nav.rounds': '/rounds',
  'nav.lists': '/lists',
  'nav.teams': '/teams',
  'nav.prizes': '/prizes',
  'nav.ratings': '/ratings',
  'nav.publish': '/publish',
  'file.setup': '/setup',
  'app.settings': '/settings'
};

const LIST_ITEMS: Record<string, string> = {
  'lists.starting-rank': 'StartingRank',
  'lists.alphabetical': 'Alphabetical',
  'lists.pairings': 'Pairings',
  'lists.standings': 'Standings',
  'lists.crosstable': 'Crosstable',
  'lists.scorecards': 'Scorecards',
  'lists.board-cards': 'BoardCards',
  'lists.prizes': 'Prizes'
};

/** Handles a menu item id sent by the native menu bar. */
export async function handleMenu(id: string) {
  const needsTournament = !['file.new', 'file.open', 'file.open-window', 'file.sample', 'file.import-trf',
    'file.import-other', 'app.settings', 'nav.player-db', 'help.keyboard'].includes(id);
  if (needsTournament && !requireTournament()) return;

  if (NAV[id]) return goto(NAV[id]);
  if (LIST_ITEMS[id]) return goto(`/lists?list=${LIST_ITEMS[id]}`);
  switch (id) {
    case 'file.new':
      app.newDialog = true;
      return;
    case 'file.open':
      return openTournament();
    case 'file.open-window':
      return openInNewWindow();
    case 'file.sample':
      return loadSample();
    case 'file.import-trf':
      return importTrf();
    case 'file.import-other':
      return importOther();
    case 'file.export-trf':
      return exportTrf();
    case 'file.save-as':
      return saveCopy();
    case 'file.close':
      return closeTournament();
    case 'edit.undo':
      return undo();
    case 'edit.redo':
      return redo();
    case 'players.add':
    case 'players.sort':
      app.action = id;
      return goto('/players');
    case 'rounds.pair':
      return pairNextRound();
    case 'rounds.manual':
    case 'rounds.check':
      app.action = id;
      return goto('/rounds');
    case 'rounds.delete':
      return deleteLastRound();
    case 'help.keyboard':
      app.shortcuts = true;
      return;
  }
}
