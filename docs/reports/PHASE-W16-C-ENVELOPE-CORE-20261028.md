# PHASE-W16-C-ENVELOPE-CORE-20261028

> 状态：🟡 W16-C provider-neutral envelope core 已快进合入本地 `dev`；完整生产验收未完成。

## §0 目的

记录 W16-C 当前代码切片、文档同步、实际验证和未完成的生产门。该报告不关闭 W16-C、W16、WLD-7、PT-09 或 PT-10，也不表示协作正文可以在产品路径中共享。

## §1 改动矩阵

| 文件/范围 | 改动 | 状态 |
|---|---|---|
| `crates/domain-kms/src/transcript_envelope.rs` | provider-neutral envelope、canonical AAD、AES-256-GCM protector 与 focused unit tests | 本 lane 部分实现 |
| `crates/domain-kms/src/lib.rs` | 导出 envelope module/types 并更新 CypherGraph 关系 | 本 lane 修改 |
| `SRS-W16-PAYLOAD-SECURITY-001.md` | 明确 optional Worktree AAD 与单对象限制，区分 core 与生产验收 | v0.2 |
| `BD-W16-PAYLOAD-SECURITY-001.md` | 加入当前 API、RNG/边界及 zeroization/生产限制 | v0.2 |
| `DD-W16-PAYLOAD-SECURITY-001.md` | 固定完整 AAD 字段顺序、编码、可选 Worktree tag 和上限 | v0.2 |
| `WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` | 记录 W16-C 实施切片和后续生产阶段门 | v0.71 |
| 本 brief / dispatcher audit | 记录 lane 边界与执行状态 | 由协调者核验 |

## §2 验证摘要

- `rtk proxy cargo fmt --package domain-kms`：通过，格式化检查无输出错误。
- `rtk proxy cargo +1.99.0 test --locked --offline -p domain-kms --lib -j 4`，使用 lane 专属 `CARGO_TARGET_DIR=E:\DevCache\cargo\target-w16-c-envelope-core`：**21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s**。此前首轮运行 19/21；2 项因测试中把 12-byte `canvas-draft` vector 的 AAD 长度误写成 217（该 vector 实际为 165）而失败，修正后逻辑未再改；一次共享 target 并行碰撞以 Windows `LNK1104` 失败，随后改用 lane 专属 target。
- 两次 21/21 测试编译都保留 3 个 deprecation warning：`crates/domain-kms/src/lib.rs` LocalMockKms 路径的 `Nonce::from_slice` 位于 297、342、392 行；本 lane 新增路径已改用 `Nonce::try_from`，未扩大修补到 mock 实现。
- CodeGraph 工具报告本 worktree 未索引（缺少 `.codegraph/`），按仓库约束停止继续查询并对 `profile.rs`、`provider_contract.rs` 做精确源码核对。
- CypherGraph Guardian `py -3 .../cgg_guard.py --root . --changed --stamp`：PASS，2 个改动 Rust 源文件已刷新 SHA-256 header。
- 包级 `cargo fmt --package domain-kms -- --check`、`git diff --cached --check`、10 项 MAGOS manifest 精确匹配与提交前完整差异审查均通过。
- 在目标 `dev` 工作树再次运行 lane 专属 `cargo +1.99.0 test --locked --offline -p domain-kms --lib -j 4`：**21 passed; 0 failed**。
- 未运行 REST、PostgreSQL/RLS、Neon、真实 KMS、Multipass/K3s、持久化、跨重启、性能或商业分发验证。

## §3 已知缺口

- `TranscriptEnvelopeProtector` 是 core library API；尚未接入生产 `TranscriptBodyProtector`、认证/授权 owner、REST 路径、目标数据库或协作写入链。
- 没有生产 KMS provider 或 trusted profile resolver；不得用 `LocalMockKms` 作为生产回退。没有持久化、重启恢复、key rotation/rewrap、backup restore 或引用保留验证。
- 单对象上限：plaintext 256 KiB、wrapped DEK 64 KiB、JSON envelope 1 MiB；这不证明进程 RSS、并发量、队列、背压、取消时限或 PT-09/PT-10 预算。
- nonce 使用 rand 0.10 `ThreadRng`，经 `SysRng/getrandom` seed 并自动 reseed；不声称逐字节直接调用 OS RNG。
- `SecretDataKey` 只有 best-effort 清理，没有 audited `zeroize` feature；AES 内部副本清理也未证明。可靠 zeroization 保持 OPEN。
- 当前 envelope 尚未实现 ciphertext digest，也未实现 owner transaction/RLS、持久业务 resource reference 或跨实例可恢复读回。
- optional `worktree_id` 是本次相对旧 DD 的显式设计增量，位置为 Run 之后。AAD 最大长度由字段宽度推导为 absent 201 bytes / present 217 bytes。

## §4 子代理失败接手清单

- 原 W16-C worker 在用量上限处停止，尚未完成最终报告校订、staged diff 审查和提交；协调者接手完成审查与收尾。Cargo 链接冲突来自并行 lane 共用的目标目录；本 worker 未终止、覆盖或清理其他 lane 的构建。
- 协调者已完成独立源代码审查与 lane 专属 Rust 测试复核；提交 `7db36f6f2fbeb09f3b6d31917b33c0490239510e` 已从 `codex/w16-c-envelope-core-20261028` fast-forward 合入本地 `dev`。未 push，也未清理其他 worktree。

## §5 守门规则

- 只修改已登记的 W16-C manifest 文件；不改 `Cargo.toml`/`Cargo.lock`、REST、SQL/migration 或 provider 实现。
- 通过固定 profile 的 `KmsProviderClient` 调用 generate/unwrap；不在每次 envelope 操作内隐式 discovery。
- 不记录正文、明文 DEK、provider 原始响应或敏感 AAD；外部错误只包含稳定静态分类。
- 明确区分 AES-GCM core 单测与真实 provider、身份、owner、RLS、恢复和性能证据；任何未验证门保持 OPEN/NOT_RUN。
- 保留其他 lane 与用户工作区数据；本 worker 不执行集成或远端推送。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 / 本报告审阅 | 架构师（Mavis 接手审核）；仅审阅本阶段范围与证据，不批准生产 provider 或关闭 W16 |
| SRE Lead | 目标 provider、恢复及资源预算待验收 |
| 平台 Lead | Rust adapter 与生产身份/RLS 待实现和 review |
| 评审主持 | lane 专属 `domain-kms --lib` 测试 21/21；其余生产验证 OPEN/NOT_RUN |
| PM | W16-C/W16/WLD-7/PT-09/PT-10 均保持 OPEN/PARTIAL/NOT_RUN |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 W16-C provider-neutral envelope core、AAD 增量、单对象边界、验证现状与生产缺口；不表示 W16 或共享写入闭环。 | W16-C envelope core lane。 |
| v0.2 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 订正 CodeGraph 未索引及原 worker 用量上限中断的记录；补入协调者独立审查、Rust 1.99.0 lane 专属测试与格式验证结果，并保留集成和生产验收为未完成。 | 协调者接手收尾和独立复核。 |
| v0.3 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补记精确 manifest、staged diff 门、dev 分支回归结果与本地集成提交；生产验收仍 OPEN/NOT_RUN。 | W16-C 已快进合入本地 `dev`。 |
