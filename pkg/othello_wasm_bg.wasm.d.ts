/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const __wbg_wasmgame_free: (a: number, b: number) => void;
export const wasmgame_current_player: (a: number) => number;
export const wasmgame_get_board: (a: number) => [number, number];
export const wasmgame_get_legal_moves: (a: number) => [number, number];
export const wasmgame_get_score: (a: number) => [number, number];
export const wasmgame_is_game_over: (a: number) => number;
export const wasmgame_must_pass: (a: number) => number;
export const wasmgame_new: (a: number) => [number, number, number];
export const wasmgame_pass_turn: (a: number) => [number, number];
export const wasmgame_play_ai_move: (a: number, b: number) => [number, number, number, number];
export const wasmgame_play_move: (a: number, b: number, c: number) => [number, number];
export const wasmgame_play_random_move: (a: number) => [number, number, number, number];
export const wasmgame_size: (a: number) => number;
export const wasmgame_undo: (a: number) => [number, number];
export const __wbindgen_externrefs: WebAssembly.Table;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_start: () => void;
