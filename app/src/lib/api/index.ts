// Typed wrappers around the Tauri commands in app/src-tauri/src/commands.
// Tauri maps camelCase argument names here to snake_case Rust parameters.

import { invoke } from '@tauri-apps/api/core';
import type {
  ByeKind,
  ExportFormat,
  GameResult,
  ImportResult,
  ListKind,
  ManualOutcome,
  Player,
  PlayerId,
  PlayerRecord,
  PrizeCategory,
  Settings,
  Source,
  Team,
  TournamentInfo,
  TournamentKind,
  TournamentView
} from './types';

export const api = {
  defaults: () => invoke<{ info: TournamentInfo; settings: Settings }>('defaults'),
  current: () => invoke<TournamentView | null>('tournament_current'),
  newTournament: (args: {
    path: string;
    info: TournamentInfo;
    settings: Settings;
    kind: TournamentKind;
  }) => invoke<TournamentView>('tournament_new', { args }),
  open: (path: string) => invoke<TournamentView>('tournament_open', { path }),
  openWindow: (path: string | null) => invoke<void>('open_window', { path }),
  importTrf: (trfPath: string, cot: string) =>
    invoke<ImportResult>('tournament_import_trf', { trfPath, cot }),
  importOther: (path: string) => invoke<void>('tournament_import_swiss_manager', { path }),
  close: () => invoke<void>('tournament_close'),
  saveAs: (path: string) => invoke<void>('tournament_save_as', { path }),
  exportTrf: (path: string) => invoke<void>('tournament_export_trf', { path }),
  undo: () => invoke<TournamentView>('tournament_undo'),
  redo: () => invoke<TournamentView>('tournament_redo'),
  updateInfo: (info: TournamentInfo) => invoke<TournamentView>('update_info', { info }),
  updateSettings: (settings: Settings) => invoke<TournamentView>('update_settings', { settings }),
  changeKind: (kind: TournamentKind) => invoke<TournamentView>('change_kind', { kind }),
  recentFiles: () => invoke<string[]>('recent_files'),
  takePendingFiles: () => invoke<string[]>('take_pending_files'),

  addPlayer: (player: Player) => invoke<TournamentView>('player_add', { player }),
  addPlayerFromRecord: (record: PlayerRecord, lateEntryBye: ByeKind | null) =>
    invoke<TournamentView>('player_add_from_record', { record, lateEntryBye }),
  updatePlayer: (player: Player) => invoke<TournamentView>('player_update', { player }),
  removePlayer: (player: PlayerId) => invoke<TournamentView>('player_remove', { player }),
  withdrawPlayer: (player: PlayerId, fromRound: number) =>
    invoke<TournamentView>('player_withdraw', { player, fromRound }),
  reenterPlayer: (player: PlayerId) => invoke<TournamentView>('player_reenter', { player }),
  setBye: (player: PlayerId, round: number, kind: ByeKind | null) =>
    invoke<TournamentView>('player_set_bye', { player, round, kind }),
  sortPlayers: () => invoke<TournamentView>('players_sort'),

  playerDbSearch: (query: string, limit = 50) =>
    invoke<PlayerRecord[]>('player_db_search', { query, limit }),
  playerDbSources: () => invoke<Source[]>('player_db_sources'),
  playerDbImportFide: (path: string) => invoke<number>('player_db_import_fide', { path }),
  playerDbImportNational: (path: string) => invoke<number>('player_db_import_national', { path }),

  pairNext: () => invoke<TournamentView>('round_pair_next'),
  pairManual: (games: [PlayerId, PlayerId][], pab: PlayerId | null, force: boolean) =>
    invoke<ManualOutcome>('round_pair_manual', { games, pab, force }),
  checkRound: (round: number) => invoke<string[]>('round_check', { round }),
  deleteLastRound: () => invoke<TournamentView>('round_delete_last'),
  swapColors: (round: number, board: number) =>
    invoke<TournamentView>('round_swap_colors', { round, board }),
  setResult: (round: number, board: number, result: GameResult | null) =>
    invoke<TournamentView>('result_set', { round, board, result }),

  listHtml: (kind: ListKind, lang: string) => invoke<string>('list_html', { kind, lang }),
  listPrint: (kind: ListKind, lang: string) => invoke<void>('list_print', { kind, lang }),
  listExport: (kind: ListKind, format: ExportFormat, lang: string, path: string) =>
    invoke<void>('list_export', { kind, format, lang, path }),

  addTeam: (team: Team) => invoke<TournamentView>('team_add', { team }),
  updateTeam: (team: Team) => invoke<TournamentView>('team_update', { team }),
  removeTeam: (team: number) => invoke<TournamentView>('team_remove', { team }),
  savePrizeCategory: (category: PrizeCategory) =>
    invoke<TournamentView>('prize_category_save', { category }),
  removePrizeCategory: (category: number) =>
    invoke<TournamentView>('prize_category_remove', { category }),

  teamStandings: () => invoke<void>('team_standings'),
  ratingReport: () => invoke<[number, number][]>('rating_report'),
  normCheck: (player: PlayerId) => invoke<unknown[]>('norm_check', { player }),
  publishUpload: (server: string, token: string) =>
    invoke<void>('publish_upload', { server, token })
};
