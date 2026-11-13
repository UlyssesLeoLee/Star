# Brief: W16-C-ENVELOPE-CORE-20261028

**Agent**: worker
**Phase**: W16-C-ENVELOPE-CORE-IMPLEMENTATION-20261028
**Created**: 2026-10-28 06:42:01

---

# W16-C：provider-neutral envelope core

## 目标
在当前 dev 基线 0dfa292f2407bbc4b26255c4cd948acba6217cfa 上实现 W16-C 的第一阶段：仅在 domain-kms crate 中提供可序列化、版本化、有界的 envelope 数据结构和依赖现有 provider-neutral KMS 契约的 Rust AES-256-GCM 加解密核心，并把本阶段的实际状态同步到 SRS、基本设计、详细设计和 WBS。

## 唯一文件所有权
只修改 MAGOS manifest 中的 10 个文件。不得改 Cargo.toml、Cargo.lock、REST/HTTP handler、SQL/migration、身份代码、provider 实现或其他文件。不得复制、部署任何其他仓库代码。

## 实现边界
- 先读取当前 profile.rs 与 provider_contract.rs 的接口。使用固定的 KmsProfileIdentity 和 KmsProviderClient 执行每次操作；不得在每个 envelope 操作里做隐式 capability discovery。
- 每个 envelope 使用单次生成的数据密钥；使用 AES-256-GCM 认证加密，nonce 使用 OS CSPRNG，固定 96-bit nonce 与默认 128-bit tag；AAD 必须是 DD 中字段顺序/版本定义的 canonical binary encoding，禁止用 JSON 序列化或字符串拼接冒充 canonical AAD。
- envelope 编码应固定格式版本、AAD schema version、suite、profile identity、wrapped DEK、nonce 与 ciphertext；拒绝未知格式/算法、非法长度、超出文档明确标注的 plaintext/ciphertext/metadata 上限的输入。对外错误不得带明文、DEK、provider 原始响应或敏感 AAD。
- 解密须先验证 envelope/profile identity 和 AAD 版本，再经绑定的 provider 解包并认证解密；任何 mismatch、篡改或 KMS 错误均 fail closed，不回退 LocalMockKms、明文或进程内替代密钥。
- SecretDataKey 不可序列化/克隆；本 lane 不得宣称内存已可靠清零。因零化 feature/依赖需要独立 Cargo.lock lane，文档中把 zeroize 保证留为 OPEN，并说明生产启用仍被阻断。
- 新增精确单元测试：正常往返、改动 AAD、改动 ciphertext/tag、格式/大小边界、profile 不匹配与 provider 错误；断言不会把明文/DEK 写入 envelope 或安全错误。仅使用现有依赖。
- 这是核心库的部分实现，不接生产请求路径、不表示 TranscriptBodyProtector / W16-C 已完成，不关闭 W16/WLD-7/PT-09/10。实际 KMS provider、可信 profile resolver、REST/RLS、持久化及跨重启恢复仍须保持 OPEN/NOT_RUN。

## AAD 与资源界限补充
- 规范顺序按 DD §3.1：domain separator、schema version、tenant、project、run、optional worktree tag/UUID、resource kind、resource、actor、base revision、payload、checkpoint revision、retention revision、envelope format。Worktree 是相对旧 DD 的显式版本化增量，放在 Run 后；tag `0` 表示 absent，tag `1` 后跟 16-byte UUID。
- 所有多字节整数使用 big-endian；resource kind 是 1–64 字节受限 ASCII，长度前缀为 u16。由固定宽度推导，AAD 上限为 absent 201 bytes、present 217 bytes；focused tests 必须验证这两个最大边界和完整 test vector。
- 单对象上限为 plaintext 256 KiB、wrapped DEK 64 KiB、JSON envelope 1 MiB。`rand::rng()` 是 rand 0.10 `ThreadRng`，通过 `SysRng/getrandom` seed 并支持自动 reseed；不得宣称每个 nonce 字节均由 OS RNG 直接读取。

## 文档更新
在已列出的 SRS/BD/DD、WBS 与报告中精确区分 envelope core 有实现/测试证据与生产保护/解密闭环尚未实现；修订历史增加新条目，不改写既有历史叙事。报告按仓库七段结构填写，明确通过与未执行的验证、已知缺口、Agent 执行/接手情况、守门规则和签批。WBS 中只把本阶段核心任务标为 partial，不得关闭 W16-C 或 W16。

## 验收与集成准备
按 workspace Rust 1.99.0 与仓库守门运行格式检查及 cargo test -p domain-kms --lib -j 4 --offline --locked；若失败，报告真实原因，不扩大权限/依赖/文件范围。运行 git diff --check，核对差异严格落在 manifest，确认本 lane worktree 干净并提交。提交 subject 引用本 brief。不要 rebase/merge/push；把 commit SHA、测试命令和结果回报协调者，由协调者独立审查后再集成。

## CypherGraph
Rust 源文件必须维护准确的 @cypher graph header；先查询 CodeGraph。若该 worktree 未索引或 graph 返回不相关内容，说明该缺口并对准确 source 做定向读取；不要自行初始化/索引。

## Read COMMENTS
如有 @W16-C-ENVELOPE-CORE-20261028 留言，先读取并逐条答复；没有时记录无留言。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
