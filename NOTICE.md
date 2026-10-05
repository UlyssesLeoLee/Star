# 📁 NOTICE — Star 仓第三方许可归属

> **生成方式**: `cargo about generate about-list.hbs -o <out> --all-features` (cargo-about 0.9.2)
> **交叉验证**: `cargo deny list --format json` (cargo-deny 0.20.2) — 两者独立产出后比对
> **生成日期**: 2026-10-05
> **依据**: [AGENTS.md §0 商业开源依赖硬约束](../../../AGENTS.md) —— 对修改、链接、捆绑、安装和再分发逐项审查并履行相应源码、许可证、NOTICE、安装信息等义务。
> **门禁**: `deny.toml` (CI 阻断) + `about.toml` (本文件清单) 语义保持一致

---

## 依赖闭包许可分布（实测 1265 个 crate）

数据来源：`cargo deny list --format json`，2026-10-05。

| 许可 | crate 数 | 义务性质 |
|---|---:|---|
| MIT | 611 | 宽松 |
| Apache-2.0 | 560 | 宽松（含专利授权与 NOTICE 义务） |
| Zlib | 22 | 宽松 |
| Unicode-3.0 | 19 | 宽松 |
| BSD-3-Clause | 12 | 宽松 |
| ISC | 8 | 宽松 |
| Unlicense | 6 | 宽松（公有领域） |
| Apache-2.0 WITH LLVM-exception | 6 | 宽松 + 官方例外 |
| **MPL-2.0** | **5** | ⚠️ **文件级 copyleft** |
| BSL-1.0 | 4 | 源许可型 |
| BSD-2-Clause | 3 | 宽松 |
| CC0-1.0 | 2 | 公有领域 |
| CDLA-Permissive-2.0 | 2 | 宽松 |
| **LGPL-2.1-or-later** | **2** | ⚠️ **copyleft（库链接型）** |
| BSD-1-Clause | 1 | 宽松 |
| MIT-0 | 1 | 宽松 |
| 0BSD | 1 | 宽松 |

**无任何** 非商业（NC）、field-of-use 或 source-available（BUSL / SSPL / Elastic / Facebook / PolyForm）许可。

## ⚠️ Copyleft 依赖明细（须履行文件级 copyleft 义务）

### MPL-2.0 × 5 — 弱 copyleft（文件级）

| crate | 版本 | 引入路径 |
|---|---|---|
| `cssparser` | 0.37.0 | ← `dom_query` 0.28.0 |
| `selectors` | 0.38.0 | ← `dom_query` 0.28.0 |
| `cssparser-macros` | 0.7.1 | ← `cssparser` |
| `dtoa-short` | 0.3.5 | ← `cssparser` |
| `option-ext` | 0.2.0 | ← `dirs-sys` 0.5.0 |

**义务评估**：MPL-2.0 为**文件级** copyleft —— 修改 MPL 文件须以 MPL 开源，但**将其作为库链接进自有程序不触发本仓源码披露**。本仓按 AGENTS.md §0「不得把 copyleft 等同于禁止商用」将其列入 `deny.toml` 白名单，义务范围限于上述 crate 自身文件。

### LGPL-2.1-or-later × 2 — copyleft（库链接型）

| crate | 版本 | 引入路径 |
|---|---|---|
| `r-efi` | 5.3.0 | ← `getrandom`（**UEFI target**） |
| `r-efi` | 6.0.0 | ← `getrandom`（**UEFI target**） |

**义务评估**：`r-efi` 为 `getrandom` 的 UEFI 平台后端，**仅在 `target_os = "uefi"` 时编译**。Windows / Linux 构建产物**不含**该 crate。若将来发布 UEFI 目标产物，须重新评估 LGPL 动态链接义务。

> ⚠️ **方法论教训（per 报告 §2.3 订正）**：`cargo deny check` 输出的 `license-not-encountered` 警告**不能单独用于断言某许可在依赖树中不存在**。v0.1 报告曾据 11 条该类警告断言「依赖树无 copyleft」，而 `cargo deny list` 的结构化输出证明 MPL-2.0 × 5 与 LGPL-2.1-or-later × 2 **确实存在**。断言"不存在"必须用 `cargo deny list` 这类枚举式输出交叉验证，不能用"未命中"式警告反推。

## 完整清单生成

```bash
# 精简清单（本文件的数据来源）
cargo deny list --format json          # 结构化, 适合 CI artifact
cargo deny list --format human         # 人类可读

# 完整 LICENSE 文本聚合 (~208 KB, 建议作 CI artifact 而非入库)
cargo about generate about-list.hbs -o target/NOTICE-full.md --all-features
```

## 已知待处理项

| 项 | 状态 | 说明 |
|---|---|---|
| `rsa v0.9.10`（经 jsonwebtoken） | 🟡 CVE-2023-49092 | Marvin Attack 时序侧信道，CVSS 5.9 Medium，**上游无补丁**。`star-api-rest/src/auth/mod.rs:157` 存在 RS256 私钥签名路径。详见报告 §3 缺口 #7 |
| `fxhash` / `instant` / `proc-macro-error` | 🟡 unmaintained | 均由 `sled v0.34.7` 传递引入，无 CVE 关联。详见报告 §3 缺口 #8 |
