# W16 KMS 与 Envelope Encryption Provider 研究

- 研究日期：2026-10-28
- 范围：自托管开源 KMS/Transit、云端 KMS、Rust 客户端与部署边界
- 状态：架构研究；**未批准任何生产 provider**，完整分发依赖与实际部署证据仍需后续门禁

## 1. 结论摘要

建议把 `KmsClient` 保持为 Star 所有的 provider-neutral Rust 端口，以“Star 本地生成 DEK、使用本地 AEAD 加密内容、provider 只 wrap/unwrap DEK”为共同模型。首个自托管 provider 候选为 OpenBao Transit；AWS KMS 适合作为可选云端 provider，尤其当 Vercel 与 AWS 的 OIDC 信任配置可以接受时。它们都不能成为 Project/Run 权限、保留期、数据事务、审计事实或 Outbox 的 owner。

目前协调审计给出的代码基线是 `KmsClient`/`LocalMockKms`：mock key 只在进程内存中，进程重启后并不构成持久密钥服务。故而 W16 在有持久密钥身份、重启恢复、密文可解包及 fail-closed 验证前，仍应将 transcript/body protection 标为未生产可用；不能以 mock 通过替代 provider 验收。

| 候选 | 角色与许可证 | Envelope 与安全能力 | Vercel / Multipass K3s 适配 | 当前建议 |
|---|---|---|---|---|
| OpenBao Transit | 自托管 KMS/Transit 软件；主仓 `LICENSE` 为 MPL-2.0。许可证授予使用、修改、分发等权利，并对分发的 covered source/executable 与修改文件施加相应通知/源码条件；没有非商业、行业、席位或请求量限制。该结论只覆盖主仓许可证，不替代 image/chart/依赖闭包审查。[LICENSE](https://github.com/openbao/openbao/blob/main/LICENSE) | `datakey` 可产生 plaintext DEK 和 wrapped DEK；Transit 也支持加解密、key version、rotate、rewrap。可用 ACL 将应用限于命名 key 的特定操作；服务端 TLS、认证、审计由部署方配置。[Transit](https://openbao.org/docs/secrets/transit/) | 需要 Rust 服务能以校验证书的 TLS 访问 OpenBao endpoint。Vercel 可用 OIDC/JWT 信任接入的方向，但 issuer、`aud`/`sub`、token TTL 与 policy 映射尚需实测。两台 Multipass guest 不足以做容错 Raft；本地测试只能按单节点、非 HA 使用并验证快照恢复。 | 推荐作为先行的自托管候选；不能把双节点配置称为 HA，也不能在 endpoint、backup/unseal、ACL、审计 sink 未验收前放行生产加密。 |
| AWS KMS | 托管云服务，不是可分发 OSS；Rust AWS SDK 仓库为 Apache-2.0。服务本身受 AWS Customer Agreement / Service Terms 管辖，并按 key 与 API use 计费；因此是可选托管适配器，不满足“完全开源自托管”的定位。[SDK LICENSE](https://github.com/awslabs/aws-sdk-rust/blob/main/LICENSE)、[AWS Service Terms](https://aws.amazon.com/service-terms/)、[KMS pricing](https://aws.amazon.com/kms/pricing/) | `GenerateDataKey` 返回一次性 plaintext DEK 与由 KMS key wrap 的副本；`Decrypt` 还原 DEK。Star 负责对正文做 AEAD。Key policy / IAM 可最小授权，encryption context 可约束用途；CloudTrail 记录 KMS 操作。对符合条件的对称 AWS-origin customer key 支持自动轮换，旧材料供旧 ciphertext 解密。[Data keys](https://docs.aws.amazon.com/kms/latest/developerguide/data-keys.html)、[least privilege](https://docs.aws.amazon.com/kms/latest/developerguide/least-privilege.html)、[CloudTrail](https://docs.aws.amazon.com/kms/latest/developerguide/logging-using-cloudtrail.html)、[rotation](https://docs.aws.amazon.com/kms/latest/developerguide/rotate-keys.html) | Vercel 已记录 OIDC federation 到 AWS STS `AssumeRoleWithWebIdentity` 的模式，可免长期 access key；Rust function 收到的 token 位于 request header，Rust 端 credential exchange 仍要实现/实测。Multipass K3s 需为自己的 ServiceAccount issuer 建立可验证的 IAM OIDC trust，不能假设它像托管 K8s 已配置好。AWS 端承接备份/服务运行职责，但 key deletion 超过等待期后不可恢复。[Vercel OIDC](https://vercel.com/docs/oidc)、[OIDC reference](https://vercel.com/docs/oidc/reference)、[key deletion](https://docs.aws.amazon.com/kms/latest/developerguide/deleting-keys.html) | 作为可选 cloud adapter 评估；生产采用前确认 Vercel 与自建 K3s 两侧的短期身份、成本、区域可用性和密钥删除防护。 |
| 第三方 `openbao` Rust crate | 非 OpenBao 官方发布的 Rust crate；其仓库声明 Apache-2.0 或 MIT。其 GitHub release 页面显示 2.2.5 于 2026-10-28 发布，但仓库体量仅 3 stars / 1 watcher（页面读取时），不是活跃社区规模的充分证据。[repo](https://github.com/valkyoth/openbao-rust-crate)、[releases](https://github.com/valkyoth/openbao-rust-crate/releases) | 项目宣称覆盖较多 OpenBao API 与版本 profile；这些是维护者声明，不能当作 Star 的兼容性、安全审查或持续性保证。 | 能为 OpenBao 调用提供 typed async API，但增加额外 crate/锁定依赖、feature 与 API profile 信任边界；依赖 SBOM、MSRV、功能默认值、TLS 与发布真实性均未由本研究审计。 | 不作为首选基础依赖。先用既有 Rust HTTP/TLS/Serde 组件构造最小 typed adapter；若后续试用此 crate，先独立审源码、锁文件、许可证闭包和 live compatibility fixtures。 |

## 2. 已确认的事实、推断与未知

### 已确认的事实

- OpenBao 官方 Transit 文档描述它为不存储提交 payload 的密码学服务，支持 datakey、key version、加密、解密及 rewrap。轮换后新的加密使用新 key version，旧 ciphertext 仍可解密；`rewrap` 可以更换保护该 ciphertext 的版本而不向调用方返回 plaintext。[Transit](https://openbao.org/docs/secrets/transit/)、[Transit API](https://openbao.org/docs/api/secret/transit/)
- OpenBao 支持按 path/action 配 ACL，应用与运维身份可以分离；所有 API 操作要求认证 token（初始化/健康等少量例外）。API 要求客户端验证 TLS，不应关闭证书校验。[Policies](https://openbao.org/docs/concepts/policies/)、[API](https://github.com/openbao/openbao/blob/main/website/content/docs/api/index.mdx)
- OpenBao 审计覆盖 API 请求/响应，敏感字符串默认 HMAC；审计配置必须落盘/远端可写，且没有任何启用 audit device 可写入时服务会拒绝请求。多审计 sink 可提高冗余，但一个阻塞 sink 也可能卡住请求，因此 audit 路径必须做资源与可用性设计。[Audit devices](https://openbao.org/docs/audit/)
- OpenBao Raft 提供节点数据复制、snapshot save/restore；Raft quorum 必须是多数。官方表格显示 2 节点的 quorum 是 2，故一台失效时没有容错；官方文档列出的生产推荐为 5 节点。当前两台 guest 上限因此只支持非 HA 验证拓扑。[Integrated Storage](https://openbao.org/docs/internals/integrated-storage/)、[Raft snapshots](https://openbao.org/docs/commands/operator/raft/)
- OpenBao 主仓有 MPL-2.0 `LICENSE`。MPL 的 covered source 与其修改需要保留许可证/通知，并按许可证对分发的 source/executable 提供相应源码；网络使用本身不等同于“把整个 Star 改成 MPL”。发布镜像、chart、安装器与传递依赖仍需逐项确认。[OpenBao LICENSE](https://github.com/openbao/openbao/blob/main/LICENSE)
- 截至本报告日期，OpenBao `v2.7.2` 于 2026-10-26 发布，包含安全修复；官方 README 公开 GitHub Issues/Discussions、mailing list 与 Zulip 维护渠道。AWS Rust SDK 官方 release 页面在 2026-09-11 有正式 release，均处在 2025-10-28 至 2026-10-28 的观察窗口内。[OpenBao v2.7.2](https://github.com/openbao/openbao/releases/tag/v2.7.2)、[OpenBao community channels](https://github.com/openbao/openbao/blob/main/README.md)、[OpenBao API compatibility note](https://github.com/openbao/openbao/blob/main/website/content/docs/api/index.mdx)、[AWS SDK 2026-09-11 release](https://github.com/awslabs/aws-sdk-rust/releases/tag/release-2026-09-11)
- AWS KMS 的 data-key API 明确不负责本地 payload encryption，也不存储/跟踪应用的 DEK；应用应尽快清理 plaintext DEK。CloudTrail 能记录 `GenerateDataKey`、`Decrypt` 等 KMS API，但需要配置保留策略以覆盖默认事件历史期限。[Data keys](https://docs.aws.amazon.com/kms/latest/developerguide/data-keys.html)、[CloudTrail](https://docs.aws.amazon.com/kms/latest/developerguide/logging-using-cloudtrail.html)
- Vercel 文档说明 OIDC federation 可为后端换取短期云凭证，减少持久 secret；Function 事件中的 token 可通过 request header 读取。Vercel Rust runtime 文档将其标为 Beta；不能由此推断 W16 Rust 服务部署流程已经验收。[Vercel OIDC](https://vercel.com/docs/oidc)、[OIDC reference](https://vercel.com/docs/oidc/reference)、[Rust runtime](https://vercel.com/docs/functions/runtimes/rust)
- OpenBao 上游 README 明确发布的可导入 API/SDK 是 Go module；官方 HTTP API 为语言无关协议，本次没有发现上游维护的 Rust SDK。官方 API 文档还明确提示当前 `/v1/` 前缀并不保证 API 向后兼容，故须固定 server release/profile 并以 live contract tests 验证升级。AWS Rust SDK repo 声明 Apache-2.0、MSRV 1.94.1，并提供 `aws-sdk-kms` 与 Tokio/`aws-config` 用法。当前 Star Rust baseline 为 1.99.0，所以声明的 SDK MSRV 不构成当前阻断；但 service crate 仍依赖 Smithy/runtime 等 crates，必须对实际 feature-resolved Cargo.lock 执行完整许可证与 SBOM 审查。[AWS SDK repo](https://github.com/awslabs/aws-sdk-rust)、[KMS README](https://github.com/awslabs/aws-sdk-rust/blob/main/sdk/kms/README.md)、[KMS Cargo.toml](https://github.com/awslabs/aws-sdk-rust/blob/main/sdk/kms/Cargo.toml)

### 推断与待核验

- **推断：**OpenBao 适合成为 self-hosted 的第一评估对象，因为完整 Transit API、key rotation、ACL、审计和备份路径均已公开，且近期有安全版本发布；但其 K3s Helm chart、镜像 digest、基础镜像、运行用户、持久卷、seal/unseal key 托管、备份异地性和告警策略没有在本研究中实际部署/验证。
- **推断：**用 Vercel OIDC JWT auth 可以避免 OpenBao 静态 token，但必须以具体 Vercel issuer、精确 `aud`、project/environment `sub`、短 token TTL、最小 ACL 做负向验收；OpenBao 与 Vercel 的产品文档没有提供一个已验证的“一键连接”承诺。
- **待核验：**AWS KMS Rust 集成若沿用官方 SDK，可减少 SigV4 自制风险，但其真实新增编译时间、二进制/内存开销、完整 license closure 与当前 workspace 的 HTTP/TLS 依赖复用情况尚未实测。
- **待核验：**云服务条款、Vercel plan/runtime 能力、请求额度、区域驻留、服务 SLA 与收费会变化；正式选型前需按部署地区与实际 account tier 重新核对，不在此报告做成本或法律审批。

## 3. 部署方案

### 测试阶段：不超过两个 Multipass 实例

- 默认先使用单实例 OpenBao（Raft 或本地持久存储按可重建 test profile 配置）和持久卷快照，第二实例留给被测 Rust 服务/其他 K3s 角色；**这不代表 HA，也不允许用于保存不可恢复的生产密钥**。
- 若 W16 只需验证业务接口、上下文绑定和 fail-closed，可以使用独立的 AWS KMS 测试 key，经短期身份授权；确认账户预算/区域后执行，不创建静态长期 IAM access key。
- 任何验收必须包含重新启动 provider 后成功解包旧 DEK、备份恢复演练、provider 不可用/权限撤销时拒绝读取；不把 in-memory LocalMockKms 算作持久化验证。

### 生产候选：self-host 与 cloud 两个 profile

- `openbao-transit` profile 固定 provider id/version、经 TLS 校验的 endpoint、认证模式、Transit mount、opaque key reference、key usage policy digest 与 rotation metadata。Vercel 到 OpenBao 的网络必须私有或受严格访问控制；即使 endpoint 可从公网抵达，也不得暴露未认证 listener 或任何 admin/system routes。
- `aws-kms` profile 固定 region、key ARN（或不可变 key identity）、IAM role/trust policy digest、encryption context schema version。Vercel 使用 OIDC 交换短期 STS role；K3s 则为 ServiceAccount OIDC issuer 建 IAM trust，或由受控 Rust identity broker 提供短期凭证。不得把长期 access key 放进任务卡、插件、CLI、镜像或数据库。
- provider profile 升级/切换必须显式迁移。每个已持久 envelope 固定 provider/version/profile digest、endpoint/key ref 与 ciphertext 格式；禁止使用新 provider 解旧 provider 的 opaque wrapped bytes。切换完成前 drain 所有待处理/重试中的加密操作。

## 4. 推荐的 provider-neutral Rust seam

先在现有 `KmsClient`/domain seam 上冻结跨 provider 的最小能力，不把供应商路径、HTTP header、token shape、AWS ARN 或 Transit ciphertext prefix 暴露给业务层：

```text
wrap_dek(profile, key_ref, plaintext_dek, encryption_context)
  -> WrappedDek { profile_id, profile_digest, key_ref, provider_key_version?, opaque_ciphertext }

unwrap_dek(envelope, encryption_context)
  -> secret-handled plaintext DEK

capabilities(profile) -> { generate_and_wrap, unwrap, optional_rewrap, optional_rotation_metadata }
```

持久化 Envelope 的 Star 结构至少含：envelope format version、固定 provider profile identity/version/digest、opaque key identity、wrapped DEK、AEAD suite、nonce、ciphertext/tag、context schema version 与内容版本。不要存 plaintext DEK、API token、AWS credentials 或明文内容。

Star 侧流程：

1. 先由 Star owner 校验 actor、Project/Run/resource 授权与当前 retention/security policy，再调用 KMS adapter；Agent/Plugin/Task CLI 不持有 KMS 凭证，也不直接访问 provider。
2. 生成短期 DEK；本地 AEAD 加密 transcript/content。将 canonical Project、Run、记录 id、revision 等稳定 id 绑定到 AEAD AAD，并映射到 provider 可用的认证 context。context 中不得放 PII 或 secret。
3. 清理 plaintext DEK 与 plaintext buffer（尽最大努力；文档必须说明 HTTP/TLS/allocator/core dump 等仍可保留副本）。应用层缓存默认为关闭；任何未来缓存都必须有明确 TTL、容量上限、清理及撤权语义。
4. 只有 provider 调用成功后才保存 ciphertext + wrapped key；数据库事务、RLS、append-only audit、retention、幂等与 outbox 仍由 Star owner 本地事务控制。KMS 成功但数据库失败产生的孤儿 wrapped key 不得变成业务完成事实；重试必须生成/绑定新 envelope 或以 owner idempotency key 去重。
5. KMS timeout、TLS 错误、认证失败、ACL deny、key disabled/deleted、unknown profile/version、解包 context 不一致或密文认证失败都 fail closed；禁止回退 LocalMockKms、明文存储、临时自签 TLS 或“接受网络写成功”。

## 5. Go / No-Go 与后续验收

| 门禁 | Go 条件 |
|---|---|
| 持久密钥 | provider 重启/进程重启后能解开历史 wrapped DEK；密文引用 immutable provider/key/profile identity；key disable/delete 前检查引用并阻止误删。 |
| Envelope 语义 | 两个 provider 的 round-trip；AAD/context 跨 Project/Run/object/revision 拷贝时解包或 AEAD 校验失败；篡改 nonce/ciphertext/tag 失败；明文不落日志、trace、error、metrics。 |
| 身份与最小权限 | positive 与 denied-path ACL/IAM 测试；撤权、凭证过期、aud/sub 不匹配、未知 provider、权限目录不可用都 fail closed；分别验证 Vercel OIDC 和 K3s ServiceAccount trust。 |
| 轮换与恢复 | OpenBao `rotate` + old-version decrypt + rewrap；AWS automatic rotation/old data key decrypt；OpenBao snapshot + unseal material restore 演练；AWS key deletion 等待期告警与不可恢复风险确认。 |
| 审计 | Star owner audit 事实与 provider audit log 可通过 request correlation id 关联；审计 sink 失效行为已决定且有负向测试；日志内容无 plaintext key/data。 |
| 可用性与资源 | KMS timeout、退避、并发上限、队列背压、取消与请求超时；受测调用延迟/内存有界；两个 Multipass 实例配额通过现有受控脚本核实。 |
| 依赖/发行 | 固定精确 crate/server/image/chart digest；生成实际 release SBOM；逐依赖核对 license、NOTICE、再分发条件、MSRV 与漏洞；未经通过不得宣布 provider 或 Rust client 已商业放行。 |

**当前决定：**继续 W16 的抽象与测试设计，但维持真实数据加密功能关闭。优先把 OpenBao Transit 作为自托管 first-provider prototype 候选；AWS KMS 作为 cloud profile 备选。最终上线决策须等待 provider contract/identity/restore/rotation/operational tests 与完整二进制依赖闭包审计。

## 6. 来源与证据时间

以上官方文档、主仓许可证和 release 页面于 **2026-10-28** 查阅；维护证据取 2025-10-28 至 2026-10-28 的发布记录。AWS Customer Agreement 页面标记 last updated 2026-08-14；AWS Service Terms 标记 last updated 2026-10-22。动态的服务文档和计费应在实际部署前重查。
