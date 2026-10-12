// Window-wide UI state. Each window shows one tournament (or none).

import type { CommandError, TournamentView } from '$lib/api/types';

export type Notice = {
  id: number;
  kind: 'error' | 'planned' | 'info';
  message: string;
  phase?: number | null;
  details?: string[];
};

export const app = $state({
  view: null as TournamentView | null,
  notice: null as Notice | null,
  busy: false,
  /** Menu action for the current page to pick up, e.g. `players.add`. */
  action: null as string | null,
  newDialog: false,
  shortcuts: false
});

let nextNotice = 1;

export function isCommandError(e: unknown): e is CommandError {
  return typeof e === 'object' && e !== null && 'code' in e && 'message' in e;
}

function isView(x: unknown): x is TournamentView {
  return typeof x === 'object' && x !== null && 'tournament' in x && 'can_undo' in x;
}

export function notify(n: Omit<Notice, 'id'>) {
  const id = nextNotice++;
  app.notice = { ...n, id };
  if (n.kind === 'info') {
    setTimeout(() => {
      if (app.notice?.id === id) app.notice = null;
    }, 4000);
  }
}

export function showError(e: unknown) {
  if (isCommandError(e)) {
    notify({
      kind: e.code === 'planned' ? 'planned' : 'error',
      message: e.message,
      phase: e.phase
    });
  } else {
    notify({ kind: 'error', message: String(e) });
  }
}

/** Runs a command; a returned `TournamentView` becomes the window's state. */
export async function run<T>(fn: () => Promise<T>): Promise<T | undefined> {
  app.busy = true;
  try {
    const result = await fn();
    if (isView(result)) app.view = result;
    return result;
  } catch (e) {
    showError(e);
    return undefined;
  } finally {
    app.busy = false;
  }
}

/** Consumes a pending menu action if it matches. */
export function takeAction(name: string): boolean {
  if (app.action === name) {
    app.action = null;
    return true;
  }
  return false;
}
