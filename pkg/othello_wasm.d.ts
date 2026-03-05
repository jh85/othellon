/* tslint:disable */
/* eslint-disable */

export class WasmGame {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 0 = black, 1 = white
     */
    current_player(): number;
    /**
     * Returns board as flat array: 0=empty, 1=black, 2=white
     */
    get_board(): Uint8Array;
    /**
     * Returns legal moves as flat array: [row0, col0, row1, col1, ...]
     */
    get_legal_moves(): Uint32Array;
    /**
     * Returns [black_score, white_score]
     */
    get_score(): Uint32Array;
    is_game_over(): boolean;
    must_pass(): boolean;
    constructor(size: number);
    pass_turn(): void;
    /**
     * Pick and play the best move using AI search.
     * `level`: 0=random, 1=shallow, 2=default, 3=deep, 4=deeper.
     * Returns [row, col] or empty if pass.
     */
    play_ai_move(level: number): Uint32Array;
    play_move(row: number, col: number): void;
    /**
     * Pick and play a random legal move. Returns [row, col] or empty if pass.
     */
    play_random_move(): Uint32Array;
    size(): number;
    undo(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmgame_free: (a: number, b: number) => void;
    readonly wasmgame_current_player: (a: number) => number;
    readonly wasmgame_get_board: (a: number) => [number, number];
    readonly wasmgame_get_legal_moves: (a: number) => [number, number];
    readonly wasmgame_get_score: (a: number) => [number, number];
    readonly wasmgame_is_game_over: (a: number) => number;
    readonly wasmgame_must_pass: (a: number) => number;
    readonly wasmgame_new: (a: number) => [number, number, number];
    readonly wasmgame_pass_turn: (a: number) => [number, number];
    readonly wasmgame_play_ai_move: (a: number, b: number) => [number, number, number, number];
    readonly wasmgame_play_move: (a: number, b: number, c: number) => [number, number];
    readonly wasmgame_play_random_move: (a: number) => [number, number, number, number];
    readonly wasmgame_size: (a: number) => number;
    readonly wasmgame_undo: (a: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
