# HANDOFF-SEC-LICENSE-001: 许可门禁现状 + Marvin Attack / ES256 迁移决策

> **状态**: 🟡 待办（已拍板暂不迁移，转 handoff）
> **日期**: 2026-10-05
> **拍板**: per `ask_8f97b10454f7bf708c009f40` — Ulysses 选定「迁 ES256（ECDSA P-256）彻底消除」，**随后于同轮改判「先不迁移，但要写进 handoff」**。本文件即该改判的落档。
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **关联文档**: [ADR-0054](../architecture/2026-08-26-upgrade/adr/0054-cd-delivery-stack-argocd-kargo-rollouts.md) ｜ [PHASE-CD-SELECTION-REPORT v0.2](PHASE-CD-SELECTION-REPORT.md) ｜ [NOTICE.md](../../../NOTICE.md) ｜ 根目录 `deny.toml`

---

## §1 一句话

**本仓 JWT 存在真实的 RSA 私钥签名路径，Marvin Attack（CVE-2023-49092）暴露面已确认，但上游永久无补丁；Ulysses 已拍板 ES256 迁移为推荐方案，本阶段不实施，转本 handoff 跨 session 续做。**

## §2 事实基础（全部经 2026-10-05 实测复核）

### 2.1 暴露面确认：私钥签名路径真实存在

| 位置 | 内容 |
|---|---|
| `crates/star-api-rest/src/auth/mod.rs:156-160` | `issue_token()` 用 `Header::new(Algorithm::RS256)` + `EncodingKey::from_rsa_pem(config.private_key_pem)` + `encode()` —— **RS256 私钥签名** |
| `crates/star-api-rest/src/auth/mod.rs:166` | `verify_token()` 用 `DecodingKey::from_rsa_pem(public_key_pem)` 做公钥验签 |
| `crates/star-api-rest/src/auth/oauth/keypair.rs:91-93` | `RsaPrivateKey::new(&mut rng, 2048)` + `RsaPublicKey::from(&private_key)` —— 运行时生成 RSA 2048 |
| `crates/star-api-rest/src/auth/oauth/keypair.rs:150-151` | `private_key() -> RsaPrivateKey`（`from_pkcs8_pem`） |
| `crates/star-api-rest/src/auth/oauth/keypair.rs:205` | JWK `alg: "RS256"`（`n`/`e` 分量来自 `RsaPublicKey`） |

**这是服务端网络路径**：token 由 `issue_token` 在认证流程中签发，攻击者可反复调用并测量响应耗时，满足 RUSTSEC 所述「attacker can observe timing over the network」前提。

> ⚠️ **v0.1 报告曾判定「仅走 RSA 公钥验签，不执行私钥运算，故 Marvin 不可达」—— 该结论已被上述代码证伪并订正。** 见 PHASE-CD-SELECTION-REPORT v0.2 §3 缺口 #7。

### 2.2 官方事实（rustsec.org，2026-10-05 复核）

- **RUSTSEC-2023-0071 / CVE-2023-49092**，影响 crate `rsa`，经 `jsonwebtoken 11.1.0` 引入（`cargo tree -i rsa` 实证）
- **CVSS 5.9 MEDIUM**（`AV:N / AC:H / PR:N / UI:N / C:H`）
- **无补丁**：`patched = [] is intentional`。rsa 0.9.10（本仓版本）与 0.10.0-rc.18 截至 2026-09-12 仍受影响，**升级无法解决**
- 官方 workaround：*"Avoid using the rsa crate in settings where attackers can observe timing, e.g. over the network. Local use on a non-compromised computer is fine."*

### 2.3 当前门禁处置

`deny.toml` `[advisories] ignore` 中显式登记该条（非静默放行）：

- reason 内写明「**已修正 v0.1 错误判定**：存在 RS256 私钥签名路径，符合官方'网络可观测时序'威胁模型」
- 复审期限 `2026-11-04` 写在 reason 文本内
- ⚠️ **cargo-deny 0.20.2 不支持 ignore 项的 `expiration` 字段**（实测 `error[unexpected-keys]: expected ["id", "reason"]`），故复审期限**无机器强制力**，依赖人工复审

## §3 本阶段决策（2026-10-05）

**不实施 ES256 迁移。** 保留上述显式 ignore 项 + 人工复审。

代价如实记录：RSA 私钥签名路径在理论**仍可**被高精度时序攻击（AC:H 需极高精度时序测量，实用性低但非零）。

## §4 推荐方案：迁 ES256 的改动清单（续做时直接执行）

### 4.1 前置评估（动手前必做）

1. **确认下游对接方**：`jwks` 端点对外暴露（`oauth_integration.rs` 有集成测试），须确认是否有外部消费者依赖 `alg: "RS256"` 与 RSA 的 `n`/`e` JWK 分量
2. **确认已签发 token 存量**：`exp` 通常为小时级，确认无长 TTL token 需要双算法并存过渡
3. **确认密钥分发方式**：`JWT_PRIVATE_KEY_PEM` / `JWT_PUBLIC_KEY_PEM` env var（守门 #5 v2，不入 log）须同步换为 EC PEM

### 4.2 代码改动点

| # | 文件 | 改动 |
|---|---|---|
| 1 | `auth/mod.rs:156` | `Algorithm::RS256` → `Algorithm::ES256` |
| 2 | `auth/mod.rs:157` | `EncodingKey::from_rsa_pem(...)` → `EncodingKey::from_ec_pem(...)`（`jsonwebtoken::EncodingKey`，P-256 走 PKCS#8 SEC1 PEM） |
| 3 | `auth/mod.rs:166` | `DecodingKey::from_rsa_pem(...)` → `DecodingKey::from_ec_pem(...)` |
| 4 | `auth/mod.rs:168` | `Validation::new(Algorithm::RS256)` → `ES256` |
| 5 | `oauth/keypair.rs:12-15` | 依赖 `rsa` + `pkcs8` → 改用 `p256` + `pkcs8`（或 `elliptic-curve`）生成 P-256 keypair |
| 6 | `oauth/keypair.rs:89-100` | `RsaPrivateKey::new(rng, 2048)` → `SigningKey::random(&mut OsRng)`；PEM 序列化改 SEC1/SPKI |
| 7 | `oauth/keypair.rs:165` | `extract_jwk_components(&RsaPublicKey)` → 提取 EC 的 `x` / `y`（base64url）分量 |
| 8 | `oauth/keypair.rs:205` | JWK `alg: "RS256"` → `"ES256"`，`kty: "RSA"` → `"EC"`，`crv: "P-256"` |
| 9 | `Cargo.toml` (star-api-rest) | 移除 `rsa` 依赖（`jsonwebtoken` 的 `rust_crypto` feature 需确认是否仍引入 `rsa`） |
| 10 | `tests/common/mod.rs:279-280` | 测试 fixture 换 EC keypair |
| 11 | `tests/oauth_integration.rs:225` | 断言 `alg == "RS256"` → `"ES256"` |
| 12 | `src/lib.rs:678-686, 738` | 同上（单测内嵌 keypair + JWK 断言） |
| 13 | `tests/fixtures/*.pem` | 测试用 RSA fixture 换 EC（`schedule_rules.rs:1063` 引用） |

### 4.3 验证要求（不得跳过）

```
cargo check --workspace --all-targets -j 4      # 守门 #1 派生规 v2, 须 EXIT 0
cargo test -p star-api-rest --lib -j 4          # 含 keypair / JWK 单测
cargo deny check                                # 须确认 rsa 已离开依赖树
```

**关键判据**：迁移完成后 `cargo tree -i rsa` 应输出「package ID specification rsa did not match any packages」——**必须以枚举式输出确认 rsa 消失，不能只看 `cargo deny check` 没有报错**（per PHASE-CD-SELECTION-REPORT v0.2 §2.3 方法论：否定式结果不能反推不存在，此处是正向确认，同样应取实证）。

### 4.4 破坏性影响（须提前公告）

- **已签发 RS256 token 全部失效** —— 切换后验签算法为 ES256，旧 token 报 `Invalid`
- **JWKS 端点不兼容** —— 消费方若硬编码 `alg: "RS256"` 或解析 `n`/`e` 分量会失败
- **密钥须重新生成与分发** —— RSA PEM 不适用于 ECDSA

## §5 续做触发条件

满足任一即应启动 §4 迁移：

1. Star API 暴露到**公网**或跨不可信网络边界（当前本机 k3s / 内网部署，风险可接受）
2. 出现外部第三方消费 JWKS（`alg` 硬编码）
3. 合规审查要求关闭 CVSS 5.9 及以上未修复漏洞
4. `jsonwebtoken` 上游移除 `rsa` 依赖（届时迁移变成被动必做）

## §6 相关文件索引

| 文件 | 作用 |
|---|---|
| `crates/star-api-rest/src/auth/mod.rs` | JWT 签发/验签主逻辑（§4.2 改动点 1-4） |
| `crates/star-api-rest/src/auth/oauth/keypair.rs` | 密钥生成 + JWK 构造（§4.2 改动点 5-8） |
| `crates/star-api-rest/tests/` | 集成与单测（§4.2 改动点 10-13） |
| `deny.toml` `[advisories] ignore` | 当前显式登记项，含准确的风险描述 |
| `NOTICE.md` | 依赖闭包许可分布（`rsa` 条目标注 CVE-2023-49092） |
| `scripts/automation/license_gate_mutation.py` | 许可门禁变异测试（迁移后应重跑，确认 `rsa` 移除未破坏门禁） |

## §7 守门约束（续做时适用）

- **守门 #5 v2**：`JWT_PRIVATE_KEY_PEM` / `JWT_PUBLIC_KEY_PEM` 走 env var，**禁入 log / println**
- **守门 #7**：0 unsafe
- **守门 #1 派生规 v2**：`cargo check --workspace --all-targets -j 4` 须 0 err
- **守门 #11**：缺标比错标 —— 迁移未完成前**不得**在文档中把 ES256 写成已完成
- **守门 #19 v19**：脚本须落 `scripts/automation/` 且 commit message 引用相对路径
- **AGENTS.md §0**：迁 ES256 后须重跑 `cargo deny list`，确认 `rsa` 与 CVE-2023-0071 一并离开闭包，并更新 `NOTICE.md`

## §8 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：Marvin Attack 暴露面确认 + ES256 迁移改动清单 + 续做触发条件 | `ask_8f97b10454f7bf708c009f40` 选定迁 ES256 后改判「先不迁移，但要写进 handoff」 |
