/*
@cypher schema=1 source_sha256=b4c2b69747bf9d24dcbe87cf7d1d4d001f012de68e51ed4df3f18ea44694e85d
MERGE (self:File {path:"crates/star-desktop/src-tauri/build.rs"})
MERGE (main:Symbol {id:"crates/star-desktop/src-tauri/build.rs::main",kind:"function"})
MERGE (tauri_build:ExternalService {id:"tauri_build::build",kind:"function"})
MERGE (self)-[:DEFINES]->(main)
MERGE (main)-[:CALLS]->(tauri_build)
@endcypher
*/
//! Run Tauri's build-time config, capability, and application-context generation.

fn main() {
    tauri_build::build();
}
