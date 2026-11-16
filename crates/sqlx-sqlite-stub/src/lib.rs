/*
@cypher schema=1 source_sha256=c23c95283a80a18e2a781065e58468d8d172f84a36a5cf5baf79721c41f91b80
MERGE (self:File {path:"crates/sqlx-sqlite-stub/src/lib.rs"})
@endcypher
*/
//! 无依赖 stub, 顶替 crates-io `sqlx-sqlite 0.8.6` (见本 crate Cargo.toml 头部说明)。
//!
//! 本 workspace 无任何代码启用 sqlx 的 sqlite 后端 (cargo tree -i sqlx-sqlite 为空),
//! 因此本 crate 永不参与编译; 仅用于把 `libsqlite3-sys =0.30.1` 从解析闭包中移除,
//! 解除与 rusqlite 0.41 (libsqlite3-sys 0.39) 的 cargo `links = "sqlite3"` 冲突
//! (RUSTSEC-2026-0459 修复的一部分, per 2026-11-16 License Gate 修复)。
//!
//! 若未来启用 sqlx sqlite feature, 本 stub 的空实现会在编译期显式失败, 提示重新评估。
