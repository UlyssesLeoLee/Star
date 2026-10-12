/*
@cypher schema=1 source_sha256=f5794c87361d0617e9d4ee95d3deca171ce072e5bf9f9598f6de19992c795c54
MERGE (self:File {path:"crates/platform-messaging-proto/build.rs"})
MERGE (main:Symbol {id:"crates/platform-messaging-proto/build.rs::main",kind:"function"})
MERGE (protoc:ExternalService {id:"protoc-bin-vendored::protoc_bin_path",kind:"build-tool"})
MERGE (builder:ExternalService {id:"tonic_prost_build::configure",kind:"build-tool"})
MERGE (compile:ExternalService {id:"tonic_prost_build::compile_protos",kind:"build-tool"})
MERGE (proto:File {path:"crates/platform-messaging-proto/proto/native_transport.proto"})
MERGE (self)-[:DEFINES]->(main)
MERGE (main)-[:CALLS]->(protoc)
MERGE (main)-[:CALLS]->(builder)
MERGE (main)-[:CALLS]->(compile)
MERGE (main)-[:READS]->(proto)
@endcypher
*/

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/native_transport.proto");
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    tonic_prost_build::configure().compile_protos(&["proto/native_transport.proto"], &["proto"])?;
    Ok(())
}
