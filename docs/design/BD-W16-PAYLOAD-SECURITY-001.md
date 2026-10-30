# BD-W16-PAYLOAD-SECURITY-001 — 协作载荷安全基本设计

> 状态：Draft v0.5；补充 BD-CANVAS-AGENT-001 v0.2 的 W16 安全边界，不取代其 Run/owner 结构。W16-A 的部分设备身份与 PostgreSQL owner/RLS 切片已验收，OAuth repository 本地 PG18.6 slice 3/3 通过；Canvas 活跃 SSE 流撤销关闭单用例已在当前 dev 基线通过（run 6d23b8ceb1f0499091885c42271c2fb0），生产协作链仍未闭合。W16-C 只有 provider-neutral envelope core。

## 1. 目标与边界

沿用 Project → Engineering Run → Worktree → 本地 Branch。身份 owner 负责用户、设备凭据和设备撤销；协作 owner 负责 draft/conflict/revision、RLS、幂等、审计和 Outbox；KMS provider 仅 wrap/unwrap DEK 与报告能力。各 owner 只在自己的短事务内写事实，不跨 owner 共写事务。Agent、插件和任务卡 CLI 不接触 KMS 凭证或宿主管理接口。

本文件是 `SRS-W16-PAYLOAD-SECURITY-001` 的基本设计；Provider 候选及其来源证据见 W16 provider research。尚未批准任何生产 provider。

## 2. 身份与撤销设计

- Enrollment 由已认证 actor 发起；服务端为设备分配稳定 opaque `device_id`，并验证设备凭据的持有证明后才激活绑定。client body/header 只提供 challenge/proof，不提供可信 actor、tenant、scope 或设备作者身份。
- 令牌由 issuer 绑定 `actor_id / tenant_id / device_id / device_generation / client_id / scope`。服务主体、Agent Runtime 与用户设备采用不同 principal kind，不以 nil UUID 或个人设备 claim 代替服务身份。
- 每个受保护读写与订阅均核验当前设备 generation/status。撤销产生 append-only audit 与版本化失效事件，清理该设备 refresh-session；现有 SSE 被关闭。事件暂时丢失时只允许有界授权租约，未知状态不可视为有效。
- OAuth authorization-code 和 refresh-token 单次消费采用持久化原子条件更新并返回唯一胜者；消费事实与令牌签发/轮换处于其所属 auth owner 的事务边界。

## 3. W/T/M 逻辑记录分类

| 逻辑记录 | 主分类 | 生命周期与约束 |
|---|---|---|
| Device binding / public credential / generation | Master | 目标为 SCD Type 2；撤销不物理删除；按 tenant/user RLS；凭据私钥不上传。当前 W16-A migration 只保留当前 binding 行并原位推进 generation/status，尚无历史版本表或有效期列，SCD Type 2 仍未实现，需在 DDD 决策前维持此差异。 |
| Device enrollment challenge 与短期兑换状态 | Work | 短 TTL、有限尝试次数、过期删除；不记录 proof secret。 |
| Device revoke / auth replay audit | Transaction | append-only、物理删除禁止、审计必携、RLS；只写 opaque identity 与结果。 |
| Protected draft checkpoint metadata | Work | 有明示 retention；正文密文以受控 blob/payload reference 保存；不可在主表放明文。 |
| Conflict / canonical revision / idempotency receipt / Outbox | Transaction | append-only 事实或受控状态版本；原子 owner 事务、幂等、RLS；Outbox 只有资源/版本/事实 ID。 |
| KMS profile 与可用 key identity 映射 | Master | 版本化、SCD Type 2、无 secret；密钥生命周期由 provider 管理。 |

这是逻辑分类，不是已批准的 DDL。新增物理表须逐表补齐 retention、RLS 与审计策略后才能实施。当前设备 binding 的原位状态模型与 Master 的 SCD Type 2 目标存在实现差异；不得据此宣称历史状态可查询。

## 4. 保护与持久化边界

1. owner 先从认证上下文取得 actor/device/tenant，再校验 Project/Run/Worktree 资源授权、expected revision 与当前 retention policy。
2. Rust `TranscriptBodyProtector` 对已 coalesce 的 checkpoint 生成/取得短期 DEK，用 versioned AEAD 加密正文，再由配置的 `KmsClient` wrap DEK。Project/Run/optional Worktree/resource/revision/schema 的 canonical identity 作为 AAD；只用经 owner 校验的值构造上下文。
3. owner 保存密文、wrapped DEK 与必要 envelope metadata；正文不进入 audit、metric、trace、Outbox、NOTIFY 或 SSE。保护和解密都要求相同的稳定 AAD version。
4. KMS 调用成功但 owner 事务失败时，不能产生业务成功回执；孤儿 envelope 可按幂等/引用对账审计并清理。客户端收到失败时只显示 `local_only / sync_pending`。
5. 解密前重新授权当前 actor/device/Run/resource 并检查 retention。撤权、密文篡改、provider 不可用、未识别 envelope 或旧 key 不可用均拒绝，不泄露明文错误详情。

### 4.1 W16-C 当前代码切片与界限

- `domain-kms::TranscriptEnvelopeProtector` 是 provider-neutral core；它只接受固定的 `KmsProfileIdentity` 与 `KmsProviderClient`，每次保护调用一次 `generate_data_key`，读回调用一次 `unwrap_data_key`，不在逐 envelope 路径触发 capability discovery。
- 当前切片使用 AES-256-GCM、96-bit nonce、默认 128-bit tag。nonce 由 rand 0.10 `rand::rng()` 提供的 `ThreadRng` 填充；该 RNG 通过 `SysRng/getrandom` seed 并支持自动 reseed，不能描述成每字节直接由 OS RNG 提供。
- 单对象上限为明文 256 KiB、wrapped DEK 64 KiB、JSON envelope 1 MiB。该固定上限限制单次编码/加密对象，但不能证明并发 RSS、队列、取消、进程内密钥清零或生产 workload 达标。
- `SecretDataKey` 当前仅有 best-effort 清除行为；没有启用经审计的 zeroize 依赖/feature。AES 实现内部临时副本也没有清零保证。生产启用保持阻断。
- 该类型尚未接入 `TranscriptBodyProtector`、REST、owner transaction、RLS、持久 KMS provider、profile resolver 或跨重启恢复；不得把 branch 级实现标为共享协作可用。

### 4.2 W16-A 已验证切片与边界

- 隔离本地 PostgreSQL 18 acceptance run `6af04e5a765446c8b3baaffff126bd52` 共 70 步、失败 0；覆盖 W16 migration 顺序、owner role/ACL、设备 generation `FOR SHARE` 与拒写、tenant+actor refresh-token RLS、NULL generation 约束及旧弱同名 CHECK 升级。
- Rust `device_auth_pg` 两项 PostgreSQL 测试通过，覆盖 proof/revoke repository 行为与并发 enrollment challenge 上限；独立 Run/Worktree binding HTTP 场景验证其自身的绑定、并发、回滚与 detach 路径。后者不证明 Canvas 设备身份端到端。
- OAuth repository 的隔离 PostgreSQL 18.6 runner `1b4673269a04435daa1fab5c93333133` 通过三个 named tests，覆盖 auth-code 并发单赢家和消费/令牌持久化，以及 refresh 并发单赢家、唯一 successor、旧 refresh 撤销和 stale-snapshot 拒绝。runner 使用 disposable superuser 与迁移所需 test-only placeholders，不证明生产应用角色 grants、RLS runtime、audit/SCD2 或 OAuth HTTP 流程；完整 E2E-COLLAB-14 仍 PARTIAL/OPEN。
- Canvas SSE 撤销用例 active_canvas_sse_closes_after_device_binding_revocation 在当前 dev 基线使用生产 group GET SSE router/hub、已提交 device binding 撤销与 PostgreSQL Outbox wake，经本地 PostgreSQL 18.6 disposable runner（run 6d23b8ceb1f0499091885c42271c2fb0）通过：41/41 runner steps exit 0，exact test 1 passed/0 failed/0 ignored/3 filtered，0.16s。测试使用 postgres superuser 与 no-op audit/SCD2 prerequisites、unsigned build attestation；只证明单个活动流在设备撤销后关闭。
- 此切片未证明目录锁与并发 revoke 的阻塞关系、全体共享读写/订阅/resolve、多实例 SSE wake-up 与恢复、生产 audit/SCD2、Neon/Multipass/K3s 部署、KMS 持久化恢复或 PT-09/10。OAuth repository 并发单赢家仅有独立 disposable PG18.6 slice；生产 runtime grants/RLS 与 HTTP auth flow 未验收。Canvas SSE 单用例 PASS 不关闭上述门；相关 W16 门继续 OPEN/PARTIAL。

## 5. Provider 与部署策略

- 共同 provider-neutral 模型为 Star 生成 DEK、Star Rust 侧完成 AEAD，provider 负责保护/恢复 DEK；业务 owner 不依赖 provider subject、API 路径、token 格式或 cursor。
- 每份持久 envelope 固定 provider/profile ID、版本与 digest、opaque key identity、key version、envelope format 与 AAD schema。切换 profile 前须 drain 待处理操作并执行显式迁移。
- W16 研究将 OpenBao Transit 作为首个自托管候选，将 AWS KMS 作为可选云端适配器；两者均未批准生产。OpenBao MPL-2.0 不限制商业用途，但发布镜像、chart 和依赖许可闭包仍须核验；AWS KMS 是托管服务，不是自托管 OSS 方案。详见 provider research 及其来源。
- 测试阶段最多使用 2 个 Multipass inventory 实例；两节点 Raft 不能容忍任一节点故障，也不代表 HA。Neon/Vercel 与本地 Multipass/K3s 是两个独立验证拓扑；不以 Docker Desktop 为前提。

## 6. 资源与错误状态

每 draft 至多一个在途 checkpoint 写入，一个合并后的 bounded pending delta；KMS 并发、超时、队列、暂存明文、DEK、密文帧与 cache 均来自固定版本的 Collaboration Profile。KMS 不逐键调用，UI 不因 KMS 慢操作阻塞输入。取消、超时、OOM/队列超限与 provider 失效统一保持 canonical revision 不变，并报告可恢复的 local-only 状态。

验收必须覆盖 `device_unbound`、`device_revoked`、`credential_replayed`、`kms_unavailable`、`key_version_unknown`、`aad_mismatch`、`payload_integrity_failed`、`profile_mismatch`；对外错误不含 key、正文、token 或内部 provider secret。

## 7. 上位与下位追踪

| 需求 | 下位实现/验证 |
|---|---|
| SRS-W16-PAYLOAD-01..03 | DD §2；E2E-COLLAB-12/13/14 |
| SRS-W16-PAYLOAD-04..07 | DD §3–§5；E2E-COLLAB-15..19 |
| SRS-W16-PAYLOAD-08 | DD §6；PT-10 + PT-10-GATE-01 |

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增身份 owner、协作 owner、provider adapter、W/T/M 逻辑分类、密文边界、失败策略和最多两个 Multipass 测试实例约束。未定义或声称已实施 DDL/provider。 | W16 认证与 KMS 前置审计及 provider 研究。 |
| v0.2 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 纳入 W16-C 单对象有界 envelope core 的实际状态、AES-GCM/RNG 语义与未通过的生产/清零/并发/恢复限制；optional Worktree 纳入 AAD 上下文。 | W16-C envelope core 实施与源文档对照。 |
| v0.3 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 W16-A 的部分 PostgreSQL 18 owner/RLS 验收，并明确 Device binding 尚未实现 SCD Type 2；其它端到端撤销、OAuth 并发、生产 KMS、恢复与性能门继续开放。 | W16-A PG18 acceptance 及 SCD2 实现差异复核。 |
| v0.4 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 OAuth repository PG18.6 slice 3/3 的范围和临时 superuser/test-only prerequisite 边界，并注明 Canvas SSE candidate compile 因资源门中断、DB 未运行；完整端到端与生产授权验收保持开放。 | W16-A OAuth PG runner 结果和 Canvas candidate 资源门状态同步。 |
| v0.5 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步当前 dev 基线上 Canvas SSE 撤销单用例 PG18.6 PASS（run 6d23b8ceb1f0499091885c42271c2fb0）与精确 Rust 测试结果；保留 superuser、test-only no-op audit/SCD2 和 unsigned attestation 限制，生产授权、审计与完整协作链继续 OPEN。 | 当前 dev focused acceptance 与设计状态复核。
