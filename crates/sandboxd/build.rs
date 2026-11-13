/*
@cypher schema=1 source_sha256=75b02c3a93f60390ba6a0ee55285a7a5cff86a069063004be91daff62aad74a7
MERGE (self:File {path:"crates/sandboxd/build.rs"})
@endcypher
*/
//! tonic-build 构建脚本 (per DD-SANDBOX-002 §1.1)
//!
//! - `proto/sandboxd.proto` → OUT_DIR (prost 0.13 + tonic 0.12 codegen)
//! - protoc 走 `protoc-bin-vendored` (跟 `crates/platform-messaging-proto/build.rs` 同款,
//!   不依赖宿主 protoc, Windows/Linux/macOS 三平台 CI 一致)

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/sandboxd.proto");
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/sandboxd.proto"], &["proto"])?;
    Ok(())
}
