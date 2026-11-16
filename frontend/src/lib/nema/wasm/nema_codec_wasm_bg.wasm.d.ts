/*
@cypher schema=1 source_sha256=e35681f84881642c9e859eb1f7584dfee456e83c5545cc8862fee55843f9e5cc
MERGE (self:File {path:"frontend/src/lib/nema/wasm/nema_codec_wasm_bg.wasm.d.ts"})
MERGE (format:Symbol {id:"frontend/src/lib/nema/wasm/nema_codec_wasm_bg.wasm.d.ts::formatNemaWithProfile",kind:"function"})
MERGE (profiles:Symbol {id:"frontend/src/lib/nema/wasm/nema_codec_wasm_bg.wasm.d.ts::listNemaInterpreterProfiles",kind:"function"})
MERGE (parse:Symbol {id:"frontend/src/lib/nema/wasm/nema_codec_wasm_bg.wasm.d.ts::parseNema",kind:"function"})
MERGE (parse_profile:Symbol {id:"frontend/src/lib/nema/wasm/nema_codec_wasm_bg.wasm.d.ts::parseNemaWithProfile",kind:"function"})
MERGE (render:Symbol {id:"frontend/src/lib/nema/wasm/nema_codec_wasm_bg.wasm.d.ts::renderView",kind:"function"})
MERGE (self)-[:DECLARES]->(format)
MERGE (self)-[:DECLARES]->(profiles)
MERGE (self)-[:DECLARES]->(parse)
MERGE (self)-[:DECLARES]->(parse_profile)
MERGE (self)-[:DECLARES]->(render)
@endcypher
*/
/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const formatNemaWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number, number];
export const listNemaInterpreterProfiles: () => [number, number, number, number];
export const parseNema: (a: number, b: number) => [number, number, number, number];
export const parseNemaWithProfile: (a: number, b: number, c: number, d: number) => [number, number, number, number];
export const renderView: (a: number, b: number) => [number, number, number, number];
export const __wbindgen_externrefs: WebAssembly.Table;
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_start: () => void;
