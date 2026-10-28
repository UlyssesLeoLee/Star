# W16 KMS 与载荷保护来源审计

**日期：** 2026-10-28

**审计基线：** `dev@3b873d3e452f3340bb2e92f1d155fc5e54730a3e`

**范围：** Star `crates/` 内 KMS/会话载荷保护实现，以及已发现的本机 Rust 服务候选。只读静态审查；没有运行构建、测试或服务。

## 0. 决策摘要

1. Star 当前没有可用于生产的持久化 KMS provider，也没有 `TranscriptBodyProtector` 的具体实现或完整读回/解密路径。唯一 `KmsClient` 实现是 `LocalMockKms`：随机主密钥和 DEK 都只在进程内，重启即失效。不能把它、内存凭证仓或 mock round-trip 测试描述为生产保护能力。
2. 会话写入已有较好的边界：保护器必须由调用方注入；载荷、消息元数据、Run、派发事件和审计事实在同一 PostgreSQL 事务提交；载荷表有租户/操作者 RLS 与 TTL 字段。但密码学上下文、密钥封装格式、策略解析和运行时保护器都尚未闭合。
3. 在本机候选中没有发现可直接复用的数据密钥生成、持久密钥封装、unwrap、轮换或恢复模块。AssetsLake 的相关 Rust 模块只提供 S3 对象 CRUD；RustGameServer 的相关模块只检查 TLS 证书过期，Vault 拉取仍是未实现的规划字段。两者只适合参考不同的外围接口，不作为 KMS/信封加密实现来源。
4. 本轮没有批准任何外部代码复用。AssetsLake 根许可证为 AGPL-3.0：这不是商用禁令，但若修改网络服务，AGPL 第 13 条要求向远程交互用户提供对应源码；具体组合/分发影响和权利授权仍需在复用前审查。[GNU AGPL-3.0 第 13 条](https://www.gnu.org/licenses/agpl-3.0.html)
5. 保护型草稿、冲突内容及会话载荷在持久 provider、解密/恢复、资源上下文 AAD、轮换与删除验收完成前，必须保持执行与读回路径 fail closed。商业产品门禁不能以当前 `LocalMockKms` 通过内存 round-trip 代替。

## 1. 审计范围与证据

Star 在本审计使用独立 worktree，基线 HEAD 如上；当前代码文件没有改动。重点源码 SHA-256：

Star 根 `Cargo.toml` 声明作者 `Star Team`、许可证 `Apache-2.0`；`domain-kms/Cargo.toml` 声明 `MIT OR Apache-2.0`。本轮只依赖该仓声明和当前 `Cargo.lock`，未进行第三方完整许可证/SBOM审查。

| Star 文件 | SHA-256 |
|---|---|
| `crates/domain-kms/src/lib.rs` | `917d0e06907c549253cfeca41fedb2f288d3da8d72460fd1d73e9fe79b20c1f2` |
| `crates/domain-kms/Cargo.toml` | `b3511b976342205a9f82e6ab78023c6ee390e2d1a3ff72282c358f56da08afb8` |
| `crates/star-api-rest/src/group_api/scoped_chat_store.rs` | `7208fee03866bdaefef113f758ae9df64264a60fb4f1c0bcdca59b98d63a7a63` |
| `crates/star-api-rest/Cargo.toml` | `e8ec52e4e0bea320ace7240f12f85b6097af442d9118e9ed0740bb306791dfaa` |
| `db/migrations/2026-09-29-worktree-group-phase-5-chat-store.sql` | `b8293662f85bfee95a4d3616bd295f8adb3e17b1e77a546d6bae694a810e2872` |
| `Cargo.lock` | `4dc96f590d6691371e49ee445aa82beb10ab03de5d15b8d669b976f67a934d42` |

本机调查限定于已定位的 `D:\RustGameServer`、`D:\Ada`、`E:\CATLLM`、`E:\OpenAIActixMCP`、`E:\AssetsLake` 仓库及其 Rust 源码树；排除了 `.git`、`target`、`node_modules` 和 `.codex`。这是有边界的候选筛查，不是整台机器所有仓库或所有依赖的穷尽性清点。`E:\CATLLM` 中发现未跟踪环境配置文件，未打开、未读取。

## 2. Star KMS 与载荷保护边界

| 模块 | 当前事实 | 证据与限制 |
|---|---|---|
| `KmsClient` / `EncryptedBlob` | `KmsClient` 暴露 `generate_dek(tenant)`、`encrypt(tenant, dek_id, plaintext)`、`decrypt(tenant, blob)`、`rotate_dek(...)`、`health()`。blob 有算法、DEK ID、`encrypted_dek`、单一 `nonce`、ciphertext、tenant 与 `key_version`。接口不接收 Project/Run/Worktree/载荷 AAD，也没有版本化的资源加密上下文。 | `crates/domain-kms/src/lib.rs:70-145` |
| `LocalMockKms` | 仅有 `impl KmsClient for LocalMockKms`。构造时随机生成进程主密钥；DEK 和版本写入 `RwLock<HashMap<...>>`。`generate_dek` 返回 wrapped DEK，但 `encrypt` 返回空 `encrypted_dek`；`decrypt` 只按内存 map 查 DEK，不会从 blob 解封。重启不能解密。 | `crates/domain-kms/src/lib.rs:185-213,228-348` |
| 轮换 | mock 的 `rotate_dek` 新建一个 DEK 并返回新 ID；不迁移/重封旧密文、不持久化审计事实，也没有可验证的旧密钥撤销与读取兼容流程。 | `crates/domain-kms/src/lib.rs:350-362` |
| Provider / 依赖 | `KmsMode` 枚举列出 LocalMock、Vault、AWS KMS，但不是 adapter 存在的证据。`domain-kms` 直接依赖包括 `aes-gcm`、`rand`、`base64`、`tokio` 等；当前 Cargo workspace/lock 搜索没有找到 Vault 或 AWS KMS provider crate。`zeroize` 出现在 workspace lockfile，但不是 `domain-kms` 的直接依赖，也未见本模块使用。 | `crates/domain-kms/Cargo.toml:13-25`; `Cargo.lock`; bounded provider-name search |
| 凭证管理器 | `with_local_mock_kms()` 明确把 LocalMockKms 和 `CredentialDb::in_memory()` 组合起来；相关调用搜索命中单元测试/测试状态构造器。不得视作持久凭证存储或生产 provider 接线。 | `crates/star-credential/src/lib.rs:169-198`; tests and test-only API state references |
| `TranscriptBodyProtector` | 是注入的 trait，只有 `protect(context, plaintext)`，没有 `unprotect`；全仓 Rust 源码搜索没有发现具体实现。`PgScopedChatWorkflow::new` 要求 `Arc<dyn TranscriptBodyProtector>`，但搜索没有发现该构造器的应用组合调用。 | `crates/star-api-rest/src/group_api/scoped_chat_store.rs:37-85`; repository-wide bounded `.rs` search |
| 保护上下文 | 包含 tenant、actor、session、message、目标 Worktree IDs；缺少 Project/Run、payload ID、规范化上下文版本和 retention policy revision。trait 注释要求 authenticated envelope encryption、AAD、目标集合中最严格的 Project retention、无明文日志、wrapped key 可独立轮换和销毁，但没有执行实现。 | `scoped_chat_store.rs:37-63` |
| 持久化 | 保护结果必须带非空 ciphertext、wrapped key、key ID 和正 retention 秒数，否则本次提交返回 Unavailable。载荷、消息元数据、Run intent、派发事件、幂等回执和审计事件同事务提交。payload schema 有 ciphertext、wrapped key、key ID、TTL/expiry；RLS 对 tenant+actor 强制过滤。 | `scoped_chat_store.rs:230-274,335-429`; migration `2026-09-29-worktree-group-phase-5-chat-store.sql:62-77,234-245` |

因此，当前事务和 RLS 只证明数据写入边界存在，不证明内容已经由可恢复的生产密钥保护。没有具体 protector 时，`PgScopedChatWorkflow` 无法作为可用保护路径组合；目前也未找到读取并解密既有载荷的 workflow。

## 3. 验收缺口矩阵

| 维度 | 当前状态 | 需要关闭的条件 |
|---|---|---|
| 资源上下文 AAD | KMS 接口没有 AAD 参数；会话保护上下文不包含 Project/Run/payload 身份和版本。AES-GCM 调用未传 AAD。 | Star owner 解析并验证 canonical tenant/project/run/worktree 集合、actor、session、message、payload、schema revision；保护与解密都使用同一稳定序列化 AAD，任何字段变更都认证失败。 |
| DEK 封装与持久读取 | 生成时的 wrapped DEK 与密文输出分离；主要 `encrypt` 输出 wrapped key 为空，mock `decrypt` 依赖内存 map。 | 明确定义 versioned envelope，持久保存 ciphertext、payload nonce、wrapped DEK/key envelope、KMS profile/key version 与必要的 wrap nonce；新进程只凭持久记录及授权 provider 可解密。 |
| 租户/资源隔离 | mock 检查 tenant 字段并按 tenant+DEK map 查找，但密文没有关联数据；tenant 外层字段不是 AAD。 | AAD 绑定经 owner 验证的租户和资源身份；测试跨租户、跨 Project/Run/Worktree 替换 envelope 和篡改元数据均失败。 |
| 密钥版本与轮换 | 有 `key_version` 字段和 rotate 方法签名；mock 只分配新 ID，不重封数据、保留旧 key，且没有持久审计。 | 明确 key lifecycle：版本不可变、重封/再加密策略、旧密文可读窗口、撤销/销毁、并发与失败恢复；每次迁移可审计且幂等。 |
| 重启、备份与恢复 | 主密钥和 DEK 均进程态；无备份/恢复证明。 | 持久 provider + 可恢复的密钥引用/版本；在隔离环境证明应用与数据库恢复后仍能解密，密钥不可用时读取 fail closed。 |
| 保留期与销毁 | 文档要求保护器解析最严格 Project policy；workflow 直接接受返回的 `retention_seconds` 并存到 expiry，没有校验策略来源或版本。 | Run owner 在授权事务内解析所有目标的最严格保留策略并固化 policy revision；到期删 ciphertext/envelope 并审计 key-destruction 语义，不允许 protector 替 owner 决策。 |
| 解密 API 与错误 | Trait 只有 protect；提交阶段保护失败或字段无效会 Unavailable，没有生产 decrypt/read path 证据。 | 增加同权限边界下的解密/读回流程；provider 超时、解包失败、AAD 错误、未授权/撤权、密钥删除都明确 fail closed，错误不包含明文或密钥内容。 |
| 内存/取消 | 模块在普通数组/Vec 中持有主密钥和 DEK，未直接依赖或调用 zeroize；未见 cancellation/timeout 契约。 | 受限明文/DEK 生命周期、零化、取消与超时清理、有界并发与临时缓冲预算；密钥操作不得进入日志或不受控事件。 |
| 依赖与分发闭包 | 锁文件固定了依赖版本，但没有本轮 SPDX/SBOM 或全传递依赖许可证核对。 | provider 接入前生成对应目标/feature 的 SBOM，审查每个直接/传递依赖、二进制与镜像的商业用途和再分发义务，记录许可证/NOTICE。 |

## 4. 本机 Rust 来源调查

### 4.1 候选模块证据

| 来源 | HEAD / branch / 状态 | 相关模块及工作文件 SHA-256 | 许可证、依赖闭包与实现状态 |
|---|---|---|---|
| `E:\AssetsLake` | HEAD `0156aca56c0d7364120d7389ebe1ccf0cb64b750`; `main`; `main...origin/main [ahead 2, behind 6]`。有 16 个已修改路径与未跟踪 `backend/rust-toolchain.toml`。目标 `storage_service.rs` 与根 `LICENSE` 干净；`backend/Cargo.toml` 有本地改动，已对实际工作文件 hash。 | `backend/src/services/storage_service.rs` `fbeb221bc9391551aa368495d6e25a24cb804fea9fafa342965e08d90384a268`; `backend/Cargo.toml` `2dbe3b0c655c49bad4041226ed4f8f438de5617aca7e7f382de41880b0352ae2`; `backend/Cargo.lock` `99b6f312b8bffc40617a11de994825e64d147309505c32eb20b2b0babdd1aa2f`; `LICENSE` `6f1e622c82a380075843bb084a7ec3b1f1d12a4a02526d75e78b0924a860aa75`. | `LICENSE` 标明 AGPL-3.0；Cargo 作者字段为 AssetsLake Team，但当前证据未独立证明模块贡献者/权利链或再授权权。后端直接依赖有 `aws-config`、`aws-sdk-s3`、`aws-credential-types`、`sqlx`；Cargo.lock 存在。代码是 S3 对象 put/get/delete 等存储适配，没有 AWS KMS、DEK wrap/unwrap 或载荷 AAD；presigned URL 仍 `unimplemented!`。依赖许可证闭包/SBOM 未审计；Docker/K8s 配置存在，但实际运行部署未核验。 |
| `D:\RustGameServer` | HEAD `9997c9434af1aced6c96886f7bd82ff2bf8ce4fb`; branch `docs/gemarc-5d-atlas-v10.0`。工作树在 mock 测试脚本/CI 有修改和未跟踪文件；候选 `cert_rotation.rs` 本身没有 diff。 | `crates/shared-platform/src/cert_rotation.rs` `2f0ef6e6373f808c219d80fe0b233d8749aaa0ac7e31b878b85bfb98ce295864`; `crates/shared-platform/Cargo.toml` `d1181677f9393867018fcd738b8c3ce1d1f2cf43274135095ea918e68d619153`; root `Cargo.toml` `a825a04ae96cf58ed64caca39eaf7173dd9a82dda8b9533c40b9e3017e4c1b1e`; `Cargo.lock` `b1f189ecc7b5cdc2dfd1d08c25463d7d19858423850ba1518f04e27f3bfec74c`; `LICENSE` `bd3328238fe80f9a0f0bea5469ed40022a31fcc1809bbbca8fc28fbaf407988b`. | Root manifest声明 Apache-2.0 和作者 Ulysses；源文件声明 Vault 自动取证书属于后续阶段，默认 `vault_enabled=false`。模块只检查证书有效期/轮换阈值；`shared-platform` 依赖中未发现 Vault/KMS SDK 或密钥封装实现。Cargo.lock 存在；未做传递依赖 SPDX/SBOM 审查。 |

`E:\AssetsLake\backend/Cargo.toml` 当前工作文件的更改仅包含本地 Rust 版本策略注释与 `rust-version`；该工作树有其他用户改动，以上所有读取均为只读。AGPL-3.0 不等于禁止商业使用；GNU 文本要求修改后的网络交互版本按第 13 条提供 Corresponding Source。将模块合入其他程序、服务端部署或再分发的具体义务仍须做法律和组合边界审查，不能仅凭根许可证文件放行。[GNU 对 AGPL 网络交互条款的说明](https://www.gnu.org/licenses/gpl-faq.html#AGPLv3InteractingRemotely)

### 4.2 筛查但未识别为 KMS 来源的仓库

| 仓库 | 当前状态 | 结果与限制 |
|---|---|---|
| `D:\Ada` | HEAD `2f8b07002d0da9e99cebe2c9b5712d1029be8e08`; branch `wt-gemarc-5d-atlas`; 仅发现未跟踪 `docs/prototype/`。 | 在 `crates/` Rust 源码的 KMS、envelope、wrapped-key、Vault/AWS KMS 词项筛查中没有定位到数据密钥/载荷保护模块；无候选源文件，故没有模块 hash 或依赖闭包放行结论。 |
| `E:\CATLLM` | HEAD `d4f06e1fedf56f42fa463d6be44b2d50f7cb2c5e`; branch `main`; 存在未跟踪 Cargo、Rust 源码、部署目录及环境配置文件。环境配置文件未打开。 | 在 Rust 源码树的限定词项筛查中未定位到 KMS/envelope 模块；未跟踪工作内容与未审授权利链排除直接复用。 |
| `E:\OpenAIActixMCP` | HEAD `0818cc6cdf0243288cc50804cea2e75b21f8621c`; branch `main`; 工作树干净。 | 在其 Rust `src/` 树筛查中未发现 KMS/envelope 载荷保护实现；无候选模块，不作依赖或许可放行。 |

## 5. 复用判断与边界

| 模块 | 判断 | 允许的参考范围 | 禁止推断 |
|---|---|---|---|
| AssetsLake `storage_service.rs` | 仅作独立对象存储适配参考；当前不复制。 | S3 client 构造、配置映射和对象 CRUD 的边界设计。若今后要复用，先确认完整权利链，审查 AGPL 网络/组合/发布要求，再审查完整 Cargo.lock 许可证闭包与 NOTICE/SBOM。 | S3 对象存储不是 KMS；CRUD、不透明上传或 TLS 不等于 envelope encryption、持久 DEK、AAD、密钥恢复或隐私载荷加密。 |
| RustGameServer `cert_rotation.rs` | 仅作证书到期与阈值配置参考；不复制到数据保护链。 | 未来若需 TLS 证书到期告警，可独立评估小模块边界。 | `vault_enabled` 规划字段不是 Vault adapter，也不能证明密钥轮换或 KMS。 |
| Ada、CATLLM、OpenAIActixMCP 已筛查代码 | 没有识别出适用 KMS/载荷保护模块；本轮不复用。 | 若之后发现具体来源文件，重新固定其 HEAD、脏状态、文件 hash、所有权/许可证与完整依赖闭包。 | 不能从仓库名、Rust/Actix 框架、服务能力或锁文件存在推导出安全实现或合法复用许可。 |

Star 自有实现应保留 owner 的授权、RLS、保留策略、事务、幂等、审计和资源预算为权威；KMS 是可替换的密码学 provider adapter，不拥有业务授权或 retention 真相。外部 provider 选型不属于本审计范围。

## 6. 推荐实施与验收顺序

1. **先收敛数据契约：** 定义版本化 `ProtectedPayloadEnvelope` 与 canonical AAD；包含 tenant、Project、Run、Worktree 集合、actor、session、message、payload 和上下文/策略版本。分别表达密文 nonce 与封装元数据，固定 provider profile、key ID/version；确认字段在 owner 数据库中可持久和可审计。
2. **再落 Star provider seam：** 分开定义 AEAD 数据加密与 KMS wrap/unwrap 职责；支持 create/wrap/unwrap、版本查询和健康状态。不得让业务领域依赖某云 SDK 或外部 provider 的 subject/ACK/私有格式。配置缺失、权限撤销、密钥服务不可用或 envelope 版本未知时 fail closed。
3. **保持 owner 原子边界：** owner 在已授权事务中解析所有目标与最严格保留策略 revision，构造 AAD；只有保护成功后，才让 ciphertext/envelope、消息事实、Run intent、Outbox 和回执同事务提交。解密时重新复验当前授权与上下文；到期删除载荷并按策略审计密钥销毁，不能由保护器覆盖业务 owner 的规则。
4. **补齐秘密生命周期与资源限制：** 对 DEK/plaintext 设置短生命周期和零化；对并发、最大 payload、provider timeout/cancellation、临时缓冲、日志字段和重试做限额，避免 Rust 服务出现无界 plaintext/key 缓存。不得把 key bytes 写入日志、Outbox、BI 或持久 message payload。
5. **完成独立集成验收后才开放：** 覆盖跨 tenant/project/run/worktree AAD 替换、ciphertext/wrapped-key 篡改、全新进程重启、数据库与密钥引用备份恢复、密钥轮换中旧密文读取、销毁/到期、授权撤销、provider outage/timeout/cancel、事务回滚与重复提交、无明文日志，以及并发/内存预算。执行专用 disposable PostgreSQL/RLS 和 provider integration 验收；本轮未运行测试或构建。
6. **最后做交付闭包审查：** 固定 provider 和全部启用 feature，生成 SPDX/SBOM，逐个核对传递依赖许可证、公告、二进制/镜像安装及分发义务；对任何外部源码复用，先完成实际权利人授权和许可证组合审查，并只将对应文件清单送审。

## 7. 本轮未完成项与验证边界

- 未实现 provider、具体 protector、解密读回、跨进程恢复和生产装配；不得标为完成。
- 外部候选的完整传递依赖许可证闭包、全局 SPDX/SBOM、版权权属证明和运行部署状态未审结；因此没有许可放行或代码复用结论。
- 只完成上述精确仓库/源代码树的静态筛查；未对机器上所有盘符、归档、副本、其他用户目录进行穷尽性扫描。
- 未运行 `cargo`、测试、迁移或服务；按本阶段授权仅做只读审计，并对指定文档差异做静态核对。
