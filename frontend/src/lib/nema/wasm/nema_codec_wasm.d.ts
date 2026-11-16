/*
@cypher schema=1 source_sha256=dc0b7c60601d27c0b28c14ef8e4ae51159a41d9893934923012f371fed05f39b
MERGE (self:File {path:"frontend/src/lib/nema/wasm/nema_codec_wasm.d.ts"})
MERGE (format:Symbol {id:"frontend/src/lib/nema/wasm/nema_codec_wasm.d.ts::formatNemaWithProfile",kind:"function"})
MERGE (profiles:Symbol {id:"frontend/src/lib/nema/wasm/nema_codec_wasm.d.ts::listNemaInterpreterProfiles",kind:"function"})
MERGE (parse:Symbol {id:"frontend/src/lib/nema/wasm/nema_codec_wasm.d.ts::parseNema",kind:"function"})
MERGE (parse_profile:Symbol {id:"frontend/src/lib/nema/wasm/nema_codec_wasm.d.ts::parseNemaWithProfile",kind:"function"})
MERGE (render:Symbol {id:"frontend/src/lib/nema/wasm/nema_codec_wasm.d.ts::renderView",kind:"function"})
MERGE (self)-[:DECLARES]->(format)
MERGE (self)-[:DECLARES]->(profiles)
MERGE (self)-[:DECLARES]->(parse)
MERGE (self)-[:DECLARES]->(parse_profile)
MERGE (self)-[:DECLARES]->(render)
@endcypher
*/
/* tslint:disable */
/* eslint-disable */

/**
 * Formats a strict NEMA document and verifies it reparses under the same profile.
 */
export function formatNemaWithProfile(profile_json: string, document_json: string): string;

/**
 * Lists bundled Rust interpreter profiles. Callers must select one explicitly.
 */
export function listNemaInterpreterProfiles(): string;

/**
 * 解析 `.nema` 源码并返回 Rust 权威 AST JSON（保留 spans、字典和语义块）。
 */
export function parseNema(source: string): string;

/**
 * Parses `.nema` source using the caller's exact, pinned interpreter profile.
 */
export function parseNemaWithProfile(profile_json: string, source: string): string;

/**
 * 自适应渲染视图 (Table / 5W1H / Graph 行视图) → JSON 字符串。
 * 对应 IDE1.0 interpreter::adaptive_render_view。
 */
export function renderView(source: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly formatNemaWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly listNemaInterpreterProfiles: () => [number, number, number, number];
    readonly parseNema: (a: number, b: number) => [number, number, number, number];
    readonly parseNemaWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly renderView: (a: number, b: number) => [number, number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
