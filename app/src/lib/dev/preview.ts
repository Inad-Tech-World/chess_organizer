// Browser preview: lets `pnpm dev` run in an ordinary browser, without the
// Tauri shell, against a read-mostly fake backend built from
// testdata/trf/sample.trf (regenerate with `cargo xtask fixture`).
// Only loaded in development builds; see +layout.svelte.

import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import type { GameResult, TournamentView } from '$lib/api/types';
import fixture from './fixture.json';

const desktopOnly = (cmd: string) =>
  Promise.reject({
    code: 'invalid',
    message: `Browser preview: "${cmd}" needs the desktop app (cargo tauri dev).`,
    phase: null
  });

const planned = (feature: string, phase: number) =>
  Promise.reject({ code: 'planned', message: `${feature} is planned for Phase ${phase}`, phase });

export function installPreviewBackend() {
  mockWindows('main');
  // `?preview=start` shows the start screen; otherwise the sample is open.
  let open = new URLSearchParams(location.search).get('preview') !== 'start';
  const view = structuredClone(fixture.view) as unknown as TournamentView;
  const lists = fixture.lists as Record<string, string>;

  mockIPC(
    (cmd, args) => {
      const a = (args ?? {}) as Record<string, unknown>;
      switch (cmd) {
        case 'tournament_current':
          return open ? view : null;
        case 'tournament_open':
          open = true;
          return view;
        case 'tournament_close':
          open = false;
          return null;
        case 'recent_files':
          return [view.path];
        case 'take_pending_files':
          return [];
        case 'defaults':
          return fixture.defaults;
        case 'player_db_search':
        case 'player_db_sources':
          return [];
        case 'list_html':
          return lists[JSON.stringify(a.kind)] ?? planned('This list', 1);
        case 'round_pair_next':
          return planned('FIDE Dutch pairing', 1);
        case 'result_set': {
          const round = view.tournament.rounds[(a.round as number) - 1];
          const game = round?.pairings.find((p) => p.type === 'Game' && p.board === a.board);
          if (game?.type === 'Game') game.result = a.result as GameResult | null;
          return structuredClone(view);
        }
        default:
          return desktopOnly(cmd);
      }
    },
    { shouldMockEvents: true }
  );
}
