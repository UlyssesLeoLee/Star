# ULYS-190 §4.6 — Ada mock 项目盘點 (G-MS-01 ✅ 解決) brief

> **狀態**: 🟡 Draft v0.1 (G-MS-01 盤點落地, 待 Ulysses 拍板啟動 §4.6 + §4.7 brief)
> **日期**: 2026-09-23
> **制定者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代審
> **觸發**: ULYS-190 v0.2 approved + §4.1 Stage 1 ship + reply `01a0d0e1` 2026-09-23 23:02 JST 「完成所有后续工作」 T1 派工 + G-MS-01 盤點落地需求
> **設計稿引用**: [`c3d7d171` v0.1 + `0f199757` v0.2](../architecture/2026-09-22-mock-switches/00-design-analysis.md)
> **目標 commit**: `agent/minimaxm3/ulys-190` branch (Star 主倉, 純 docs 落档)

## 0. 目的 (Purpose)

落實 ULYS-190 §4.6 + design-analysis §6 G-MS-01 ✅ 解決: 確認 Ada 是否有 mock 工具, 避免「7 項目」字面過寬。本 brief 純 docs (per 守門 #15 + #19v19 + 0 改 logic), 落地 G-MS-01 盤點結論 + 同步到 design-analysis v0.3 + cross-reference ULYS-191 v0.3 G-ACI-07 同源問題。

## 1. G-MS-01 Ada 盤點結論 (per 本 session 2026-09-23 實證)

### 1.1 Ada 项目根定位

- **路徑**: `D:/Ada` (已 ls 驗證, 跟 ULYS-191 v0.3 G-ACI-01 推薦派工順序隱含一致)
- **類型**: Rust workspace + Flutter/Dart apps (`apps/gm-console-app/` Flutter UI)
- **Cargo workspace**: 16+ Rust crate (`ada-core` / `ada-m01-acquisition` / `ada-m02-normalizer` / `ada-m03-data-flow-engine` / ... / `ada-m16-cluster-coordinator`)

### 1.2 Ada mock 工具確認 ✅ G-MS-01 解決

**結論**: Ada **有** Rust mock crate (`D:/Ada/crates/ada-mock`), 但**屬性不同於** Star / RGS / CATs / IDE1.0 的 mock backend:

| 維度 | Star mock (`tools/star-flash-mock/`) | Ada mock (`crates/ada-mock/`) |
|---|---|---|
| 角色 | Mock backend (HTTP fixture server) | Test scaffolding crate (in-memory mocks + fixtures) |
| 設計目的 | 對外提供 mock HTTP API, 替換真實後端服務 | 單元測試用 in-memory mocks, 不對外暴露 |
| Cargo features | 0 個顯式 features | 1 個 (`server = ["dep:time"]`, 預設關) |
| Mock 對象 | 整個 cluster / app | crate 內部 builder + fixture |
| ACI 整合 | ULYS-191 §4.1.2 Stage 1 已 ship | 無 (純 unit test scaffold, 不發 ACI emit) |
| 跨項目可比對 | 是 (per ULYS-190 §3 統一 schema) | 否 (testkit 模式, 不對外 emit assertion) |

### 1.3 改造工作量評估

| 維度 | 評估 |
|---|---|
| L1 cluster_switch | 🟡 中 (新增 `.mock-cluster.json`, 但因為是 testkit, `enabled=false` 是常態, 落地 CI 配置即可) |
| L2 plugin_switch | 🔴 大 (Ada mock 沒有 plugin 概念, 需新增 plugin abstraction; 但對 testkit 來說可能 over-engineering) |
| L3 module_switch | 🔴 大 (同上) |
| 跟 ULYS-191 ACI 銜接 | 🔴 無 (testkit 不 emit assertion 到外部, `mock_switch_trace` 無意義) |

**推薦**: Ada 採用**降級模式** — 只落地 L1 cluster_switch (CI 配置, `enabled=false` 預設), 不落地 L2/L3 plugin/module_switch。理由:
1. Ada mock 是 testkit, 不對外 emit, plugin/module 粒度無實際應用場景
2. ULYS-190 §2.3 IM1.0 vs Ada 對比: IM1.0 mock 是 backend integration (e.g. `mock_kms` for IM gateway), Ada mock 是 unit test scaffold, 兩者定位根本不同
3. 強行落地 L2/L3 會違反守門 #15 scope creep (over-engineering testkit)

### 1.4 跨項目派工順序調整 (per §1.3)

| 原 §4.3 順序 (per design-analysis v0.2 §2 推薦) | 調整後順序 (per §1.3 Ada 降級) |
|---|---|
| Star → IM1.0 → RGS → CATs → IDE1.0 → GitGit → Ada | Star → IM1.0 → RGS → CATs → IDE1.0 → GitGit → **Ada (降級 L1 only)** |


## 2. 落地清單 (per brief v0.1 §1.1, 純 docs)

| 序 | 路徑 | 類型 | 內容 |
|---|---|---|---|
| 1 | `docs/architecture/2026-09-22-mock-switches/00-design-analysis.md` | 修改 (v0.2 → v0.3) | G-MS-01 ✅ 解決 + §2.3 Ada 維度改寫 (testkit 降級模式) + §9 加 v0.3 row |
| 2 | `docs/briefs/ulys-190-g-ms-01-ada-inventory.md` | 新文件 (本檔) | G-MS-01 盤點落地 (per §1.1-§1.4) |

**commit 數**: 1 commit (per 守門 #1 v19 5 守門全套跑 + 守門 #10 author=Ulysses)

### Out-of-Scope (本 brief 不做, 跨 session 續做)

- ❌ Ada L1 cluster_switch 實裝 (`D:/Ada/crates/ada-mock/.mock-cluster.json` 新文件) — 跨 session brief 啟動派工
- ❌ Ada L2/L3 plugin/module_switch — 永久降級 (testkit 模式, per §1.3)
- ❌ Star 5 sample fixture 全量加 `mock_switch_trace` (per §4.4 + G-MS-BRIEF-03)
- ❌ 其他 6 項目 (IM1.0 / RGS / CATs / IDE1.0 / GitGit / Ada) cluster_switch / plugin_switch / module_switch — 跨 session brief
- ❌ 跨項目 CI 加 `mock-switch-validate` (per design-analysis §4.5) — **本 session 已 ship (commit `f6045bd9` 後續)**
- ❌ 守門 #15 限制: 1 sub-agent 1 切點, 不批量

## 3. 守門對齊 (per AGENTS.md §4)

| 守門 | 對齊 | 落地實證 |
|---|---|---|
| #1 docs 必含 | ✅ | 本 brief + 落地後 design-analysis v0.3 + §9 修訂歷史 |
| #5 env hard ban | ✅ | 0 env 涉及, 純 docs |
| #6 中文默認 | ✅ | 本檔簡中 |
| #7 0 unsafe | ✅ | 0 code, 純 docs |
| #10 author=Ulysses | ✅ | commit 走 Ulysses |
| #11 缺標比錯標 | ✅ | §6 G-MS-01 ✅ 解決 + §6 G-MS-BRIEF-03 [TBD] 顯式標 |
| #12 docs 同步 | ✅ | commit 引用本 brief + design-analysis v0.3 |
| #13 W/T/M | ✅ | 0 涉及 (本 brief 純 docs) |
| #14 v4 Mavis 审核 | ✅ | author=Ulysses + Mavis 接手 per 8/27 19:39 JST 授權 |
| #15 scope creep | ✅ | 本 brief = 1 純 docs brief, 不跨項目不批量 |
| #19 v19 Python 化 | ✅ | 0 涉及 (純 docs) |
| #20 子代理 brief | ✅ | 本 brief = 子代理 brief, 落 `docs/briefs/ulys-190-*.md` |
| #24 vendor 中立 | ✅ | 0 涉及 (純 docs) |

## 4. 風險 + 已知缺口 (per 守門 #11)

| # | 風險 / 缺口 | 緩解 |
|---|---|---|
| **G-MS-01** | ✅ 解決 (本 brief 落地) — Ada 有 Rust mock (`crates/ada-mock`), 屬性為 testkit scaffold, 跟 Star/RGS backend mock 不同 | 降級模式 (L1 cluster_switch only, L2/L3 跳過), per §1.3 |
| **R-BRIEF-01** | Ada mock 是 testkit 模式, 強行落地 L2/L3 plugin/module_switch 違反守門 #15 scope creep | 跨 session brief 派工時顯式標「L1 only」 |
| **R-BRIEF-02** | Ada 派工順序從原 v0.2 §2 的「Star → IM1.0 → RGS → CATs → IDE1.0 → GitGit → Ada」改為「Star → IM1.0 → RGS → CATs → IDE1.0 → GitGit → Ada (降級 L1 only)」, 可能影響派工預估 token | 跨 session brief 落地時顯式標 L1 only, 預估 token 從 ~0.3-0.5M 降到 ~0.1-0.2M |

## 5. 跨 session 續做入口

1. ✅ ULYS-190 v0.2 approved (per reply `01a0d073`「a」)
2. ✅ §4.1 brief v0.1 落档 (per commit `0f199757`)
3. ✅ §4.1 Stage 1 全部 7 交付物 ship (per commit `f6045bd9`)
4. ✅ §4.5 跨項目 CI 加 `mock-switch-validate` (per commit `f6045bd9` 後續, 工具落 `tools/mock-switch-validate.py`)
5. ✅ G-MS-01 Ada 盤點落地 (本 brief)
6. 🟡 design-analysis v0.3 升版 (G-MS-01 ✅ 解決 + §2.3 Ada 維度改寫)
7. 🟡 §4.2 IM1.0 plugin_switch brief 啟動派工 (推薦: 阻力最小, 跟 ULYS-191 v0.3 G-ACI-01 結論同源)
8. 🟡 §4.3 4 項目 (RGS / CATs / IDE1.0 / GitGit) plugin_switch brief 啟動派工
9. 🟡 §4.4 7 項目 module_switch 擴展 (最大塊, 跨 session 續)
10. 🟡 §4.6 Ada L1 cluster_switch 實裝 (D:/Ada/crates/ada-mock/.mock-cluster.json 新文件, 降級模式 L1 only)

**token 累計**: ~0.02M (本 brief v0.1 起草, 對齊 ULYS-191 G-ACI-07 brief ~0.02M)
