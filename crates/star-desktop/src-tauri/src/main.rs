/*
@cypher schema=1 source_sha256=efbe3cf10180d82b1c94e5a8a33125f414546383e60d880d2fa058e4637357b1
MERGE (self:File {path:"crates/star-desktop/src-tauri/src/main.rs"})
MERGE (main:Symbol {id:"crates/star-desktop/src-tauri/src/main.rs::main",kind:"function"})
MERGE (run:Symbol {id:"crates/star-desktop/src-tauri/src/lib.rs::run",kind:"function"})
MERGE (self)-[:DEFINES]->(main)
MERGE (main)-[:CALLS]->(run)
@endcypher
*/
//! Star Desktop binary entry point; application setup lives in `lib.rs`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

fn main() {
    star_desktop_lib::run();
}
