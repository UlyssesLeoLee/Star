//! Generated gRPC types for the optional Star native transport.
/*
@cypher schema=1 source_sha256=c4ae26255bd4d8fda29ba46cc91e6b4b0ba7b767e208a18d29d263f586fe1be7
MERGE (self:File {path:"crates/platform-messaging-proto/src/lib.rs"})
MERGE (wire:Symbol {id:"crates/platform-messaging-proto/src/lib.rs::wire",kind:"module"})
MERGE (self)-[:DEFINES]->(wire)
MERGE (proto:File {path:"crates/platform-messaging-proto/proto/native_transport.proto"})
MERGE (wire)-[:IMPORTS]->(proto)
@endcypher
*/

/// Generated types for the versioned native transport wire contract.
#[allow(missing_docs)]
pub mod wire {
    tonic::include_proto!("star.platform.transport.v1");
}
