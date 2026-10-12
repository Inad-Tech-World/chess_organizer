// Native open/save dialogs with the file types the app uses.

import { open, save } from '@tauri-apps/plugin-dialog';

type Filter = { name: string; extensions: string[] };

export const COT: Filter[] = [{ name: 'Chess Organiser tournament', extensions: ['cot'] }];
export const TRF: Filter[] = [{ name: 'FIDE Tournament Report File', extensions: ['trf', 'txt'] }];
export const FIDE_LIST: Filter[] = [{ name: 'FIDE rating list', extensions: ['xml', 'zip', 'txt'] }];
export const CSV: Filter[] = [{ name: 'CSV', extensions: ['csv', 'txt'] }];
export const ANY: Filter[] = [{ name: 'All files', extensions: ['*'] }];
export const EXPORTED: Filter[] = [
  { name: 'TRF or player list', extensions: ['trf', 'txt', 'csv', 'tsv'] },
  ...ANY
];

export async function pickOpen(filters: Filter[]): Promise<string | null> {
  const r = await open({ multiple: false, directory: false, filters });
  return typeof r === 'string' ? r : null;
}

export async function pickSave(filters: Filter[], defaultPath?: string): Promise<string | null> {
  return (await save({ filters, defaultPath })) ?? null;
}

/** File name for a tournament, e.g. "Kampala Open 2026.cot". */
export function suggestName(name: string, ext: string): string {
  const safe = name.replace(/[\\/:*?"<>|]+/g, ' ').trim() || 'Tournament';
  return `${safe}.${ext}`;
}
