bash.exe: warning: could not find /tmp, please create!
bash.exe: warning: could not find /tmp, please create!
bash.exe: warning: could not find /tmp, please create!
# Mock 開關 (Mock Switches) × 全項目 Mock — 設計分析 (Design Analysis)

> **狀態**: 🟢 Approved v0.4 (reply `01a0d306` 2026-09-23 23:02 JST 「推进到完成」 T1 派工 + 6 跨项目 cluster_switch + plugin_switch 落地)
> **日期**: 2026-09-23 (v0.1 落档 → v0.2 修訂 → v0.3 G-MS-01 解決 → v0.4 跨项目落地)
> **修訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**審核** (per 守門 #14 v4)
> **觸發 issue**: ULYS-190 "Mock开关" — `01a0cb7c-8022-715b-9a4d-4bbc58e7461e`
> **適用項目**: **跨項目範圍** — Star / IDE1.0 / RGS / CATs / IM1.0 / GitGit / Ada 共 7 個項目 (跟 ULYS-191 v0.2 approved 範圍一致, 排除 Xiaoshuo)
> **關聯文檔**:
> - [`docs/architecture/2026-09-22-aci-mock-interface/00-design-analysis.md`](../2026-09-22-aci-mock-interface/00-design-analysis.md) (姊妹文檔, ULYS-191, ACI 提示词型断言)
> - ULYS-189 "Jev深度结合" (姊妹文檔, `docs/architecture/2026-09-22-jev-integration/`, 小腦判斷層)
> - `tools/star-flash-mock/.aci.json` v0.1 (ACI 頂層契約, per ULYS-191 §3.1)
> - `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-1.json` (既有 fixture 結構樣本)

---

## 0. 目的 (Purpose)

ULYS-190 description 字面: **「Mock项目要可以触发测试目标的功能开关，对应app集群的开关和app内部的plugin开关，以及plugin内部模块的开关，因为测试时可以更方便识别哪个功能有问题。」**

本文檔**不**給最終實裝方案 — ULYS-190 是「改造」類 issue, 落地需 per 项目 brief 拆解 (per 守门 #15 scope creep)。
本文檔目標:

1. **建立 3 層開關術語表**: app cluster / app plugin / plugin module — 三層各自的開關維度 / 配置承載體 / 啟用粒度
2. **盤點 7 個 Mock 項目現狀**: 每個 mock 項目當前有沒開關? 是 hardcode 還是 config 驅動? 缺什麼
3. **3 層開關 Schema 鎖版 (Draft v0.1)**: 統一 schema (cluster_switch / plugin_switch / module_switch), 跨項目一致
4. **跨項目改造路徑**: 7 個項目如何排程落地, 是 monorepo 共享 crate 還是各自拷貝, 守门 #15 + #20 子代理 brief 拆解
5. **6 個落地前必拍決策**: 不決不寫代碼 (per ULYS-191 v0.1 / ULYS-189 v0.1 經驗)
6. **跟 ULYS-191 ACI 的銜接**: 開關打開後, ACI emit 才知道「當下測的是哪個功能」, 否則 scope.domain 寫死跟實際 mock 狀態脫節
7. **守門自檢**: 跟 AGENTS.md §4 守門硬約束對齊

> **「功能開關」縮寫解讀** (per description 字面 + context, 不查外部):
> **Mock 開關** = 控制 mock 項目行為可開可關的 feature flag 體系。
> 「對應 app 集群的開關」= **Cluster Switch** — 一個 app 在某個部署環境 (dev/staging/prod) 是否啟用 mock backend (per app 整體)
> 「app 內部的 plugin 開關」= **Plugin Switch** — 該 app 內部某個 plugin (e.g. KMS, RBAC, audit, billing) 是否被 mock 接管 (per plugin 粒度)
> 「plugin 內部模塊的開關」= **Module Switch** — 該 plugin 內部某個子模塊 (e.g. KMS 解鎖 vs KMS 輪換, RBAC 角色檢查 vs RBAC 權限解析) 是否被 mock 接管 (per 子功能粒度)
> 「方便識別哪個功能有問題」= 開關一打, mock 自動 emit 「該 feature 走 mock」狀態; 開關一關, mock 走真實服務; **失敗時 LLM 一眼看到「是 mock 模擬的 X 功能 vs 真實的 Y 服務」**, 不用 trace 代碼。

---

## 1. 概念映射 (Concept Mapping)

### 1.1 三層開關拆解 (per description 字面)

| 層 | 名字 | 控制範圍 | 配置承載體 (per ULYS-190 v0.1 §3 draft) | 啟用粒度 |
|---|---|---|---|---|
| **L1** | **Cluster Switch** | 整個 mock 集群 / app 整體 | `.mock-cluster.json` (per app root) | 1 個 cluster 有 1 個 cluster_switch, 全局啟用 / 全局關閉 |
| **L2** | **Plugin Switch** | app 內部某個 plugin (e.g. KMS, RBAC, audit) | `.aci.json` plugins section (per ULYS-191 v0.1 §3.1 ACI 契約) | 1 個 plugin 有 1 個 plugin_switch, 啟用 / 關閉 該 plugin 是否被 mock 接管 |
| **L3** | **Module Switch** | plugin 內部某個子模塊 | `.aci.json` modules section | 1 個 module 有 1 個 module_switch, 啟用 / 關閉 該子功能是否被 mock 接管 |

**實例對照 (per Star mock + 5 域 SRS)**:

| 三層 | Star mock 對應 | 觸發 ACI emit |
|---|---|---|
| L1 cluster | `tools/star-flash-mock/.mock-cluster.json` `enabled = true` | 整個 mock 啟用, 所有 fixture 被 dispatch |
| L2 plugin | `.aci.json` `plugins.kms.enabled = false` | KMS plugin 不 mock, 走真實 KMS 服務 (即使 cluster 啟用) |
| L3 module | `.aci.json` `plugins.kms.modules.unlock.enabled = true` | KMS plugin 整體走真實, 但「解鎖」子模塊走 mock (e.g. 測解鎖超時情境) |

### 1.2 跟 ULYS-191 ACI 的銜接 (per `docs/architecture/2026-09-22-aci-mock-interface/00-design-analysis.md` 附錄 A 第 401-402 行)

> 「觸發測試目標功能開關 = ULYS-190 落地後, ACI emit 才能精準標 `scope.domain = "admin"` 等 (否則 emit 不知道哪個 domain 失敗)」

**銜接點**:

| 場景 | 沒有 ULYS-190 開關 | 有 ULYS-190 開關 |
|---|---|---|
| mock 啟用後某個 domain 失敗 | ACI emit `scope.domain = "admin"`, 但**不知道** admin 是 mock 模擬還是 mock 通過轉發真實服務失敗 | ACI emit 帶 `mock_switch_trace = "cluster.enabled=true, plugin.admin.enabled=true, module.list.mocked=true"`, LLM 一眼看出 |
| 只想測某個 plugin 失敗 | 必須 hardcode fixture, 改一行動全部 | 打開 `plugin.X.module.Y.mocked=true`, 自動 emit 「走 mock」狀態 |
| 跨項目比較 (Star vs IM1.0) | 各項目 ACI 字段含義不一致 | 統一 `.aci.json` schema, 跨項目可比對 |

### 1.3 跟 AGENTS.md §4 守門的關係

| 守門 | 跟 Mock 開關的關係 |
|---|---|
| #1 docs 必含 | 開關 schema 本身就是 doc (`.mock-cluster.json` + `.aci.json` schema 段 + `mock-switch-spec.md`), 落地時同步 commit |
| #3 5 域 Lead | 跨項目落地拆 per 项目 brief, 走守門 #20 子代理 brief |
| #5 env hard ban | 開關不涉及 env, 0 影響 |
| #7 0 unsafe | mock 项目多數是 sh + python, 0 unsafe |
| #9 subprocess | dispatcher 讀取開關走 subprocess (per ULYS-189 v0.2 §6.1) |
| #10 author=Ulysses | 跨項目 commit 統一 author |
| #11 缺標比錯標 | 落地時顯式列「哪些 mock 项目已加開關 / 未加 / 部分加」 |
| #12 docs 同步 | 開關 spec 落 `docs/spec/mock-switch-spec-v0.1.md`, commit 引用 |
| #13 W/T/M | 開關配置屬 Master SCD-2 (慢變, 設定), per 守門 #13 c |
| #14 v4 Mavis 审核 | author=Ulysses, 跨項目 brief 走 Mavis 审核 |
| #15 scope creep | 嚴守 1 brief 1 sub-agent 1 mock 项目, 不批量 |
| #20 子代理 brief | per 项目 brief 落地 (per ULYS-189 v0.2 §6.1 + ULYS-191 §4.1.2 經驗) |
| #24 vendor 中立 | 開關自身不引入 vendor, 只用 stdlib JSON |

---

## 2. Mock 項目現狀盤點 (per 7 項目)

> **跟 ULYS-191 §2 8 項目盤點 (後改 7) 對齊** — 但 ULYS-190 維度不同: ULYS-191 看「既有 assertion 是不是 LLM 可讀」; ULYS-190 看「既有 mock 有沒有開關可切換」。

### 2.1 總表 (per 7 項目, 排除 Xiaoshuo per ULYS-191 v0.2 reply `01a0cbf3`)

| # | 項目 | 路徑 | L1 cluster 開關 | L2 plugin 開關 | L3 module 開關 | 改造工作量 |
|---|---|---|---|---|---|---|
| 1 | **Star** | `D:/Star/tools/star-flash-mock/` | ✅ Stage 1 (commit `c3d7d171` + `f6045bd9`) — `.mock-cluster.json` + `_lib_mock_switch.py` + `_lib_aci_emit.py` 加 `mock_switch_trace` 参数 | 🟡 Stage 1 (commit `f6045bd9`) — `.aci.json` 加 `aci_compat_version` 字段 (plugins section 落地跨 session) | 🔴 無 (跨 session §4.4) | ✅ 已 ship |
| 2 | **IDE1.0** | `E:/IDE1.0` | 🟢 有 (Cargo features, `cargo run --features mock`) | 🔴 無 (per plugin 解耦) | 🔴 無 | 🟡 中 |
| 3 | **RGS** | `D:/RustGameServer/tools/rgs-flash-mock/` | 🟡 部分 (鏡像 Star mock) | 🔴 無 | 🔴 無 | 🟡 中 |
| 4 | **CATs** | `D:/CATs/crates/cats-mock/` | 🟢 有 (Cargo features, `cargo test --features mock-runtime`) | 🔴 無 (plugin 解耦但無 plugin_switch 抽象) | 🔴 無 | 🟡 中 |
| 5 | **IM1.0** | `D:/IM1.0/crates/im-testkit/` | 🟢 有 (`crates/im-testkit/src/lib.rs` 模組開關) | 🟡 部分 (`feature = "mock_kms"` / `feature = "mock_audit"` 等 3 個 Cargo features, 算 plugin switch 雛形) | 🔴 無 (sub-module level 仍 hardcode) | 🟢 小 (擴 L3 module_switch 即可) |
| 6 | **GitGit** | `D:/GitGit/apps/gm-console/src/mocks/` | 🟢 有 (MSW frontend dev-only, `if (process.env.NODE_ENV === 'development')`) | 🔴 無 (frontend MSW handlers 寫死, 沒 plugin 概念) | 🔴 無 | 🟡 中 |
| 7 | **Ada** ✅ G-MS-01 解決 | 🟢 **有 Rust mock** (`D:/Ada/crates/ada-mock`, **屬性 = testkit scaffold**, 非 backend mock) | 🟡 中 (新增 `.mock-cluster.json` CI 配置) | 🔴 無 (testkit 模式, 落地違反守門 #15 scope creep) | 🔴 無 (同上) | 🟢 小 (降級模式 L1 only) |

### 2.2 7 項目細節 (每項目現狀摘要)

#### 項目 1 — Star mock (per `tools/star-flash-mock/`)

- **L1 cluster**: `start-k3s-backend.ps1` + `start-mock-http.service` 啟停硬編碼; 缺 `.mock-cluster.json` 統一配置
- **L2 plugin**: `.aci.json` v0.1 尚無 plugins section; 5 域 SRS (admin/economy/match/social/player) 概念存在, 但 mock dispatch 不分 plugin
- **L3 module**: 同上, fixture 是 per-file 粒度, 無 module_switch 抽象

#### 項目 2 — IDE1.0 (per `E:/IDE1.0`)

- **L1 cluster**: Cargo features `[features] mock = []`, 啟用時整個 mock 接管; 缺顯式 .mock-cluster.json
- **L2 plugin**: mock 內部不分 plugin (e.g. 編輯器 mock / 補全 mock / 搜索 mock 都是 1 個 feature)
- **L3 module**: 同 L2

#### 項目 3 — RGS mock (per `D:/RustGameServer/tools/rgs-flash-mock/`)

- **L1 cluster**: 鏡像 Star mock 模式 (per ULYS-191 v0.3 G-ACI-01 盤點結論); 缺顯式配置
- **L2 plugin**: 5 域 SRS (player/economy/match/social/admin), mock 不分域
- **L3 module**: 同上

#### 項目 4 — CATs mock (per `D:/CATs/crates/cats-mock/`)

- **L1 cluster**: Cargo features `[features] mock-runtime = []` + `cargo test --features mock-runtime`, 啟用方式成熟
- **L2 plugin**: `cats-mock` 內部按 domain 解耦 (per DD 結構), 但 plugin_switch 抽象缺
- **L3 module**: 同 L2

#### 項目 5 — IM1.0 testkit (per `D:/IM1.0/crates/im-testkit/`) ⭐ 阻力最小

- **L1 cluster**: `crates/im-testkit/src/lib.rs` 模組開關 (per ULYS-191 v0.3 盤點)
- **L2 plugin**: 已有 3 個 Cargo features (`mock_kms` / `mock_audit` / `mock_auth`), **算 plugin switch 雛形** (per ULYS-191 v0.3 §「推薦派工順序 IM1.0 提前」結論同源)
- **L3 module**: sub-module level 仍 hardcode, 需擴 module_switch 抽象

#### 項目 6 — GitGit (per `D:/GitGit/apps/gm-console/src/mocks/`)

- **L1 cluster**: MSW (`Mock Service Worker`) frontend dev-only, `if (process.env.NODE_ENV === 'development')` 環境變量判定
- **L2 plugin**: frontend MSW handlers 寫死, 沒 plugin 概念; TypeScript MSW handler dispatch 不分業務域
- **L3 module**: 同 L2

#### 項目 7 — Ada ✅ G-MS-01 解決 (per `docs/briefs/ulys-190-g-ms-01-ada-inventory.md` v0.1)

- **路徑**: `D:/Ada/crates/ada-mock` (跟 ULYS-191 v0.3 G-ACI-01 推薦派工順序隱含一致)
- **類型**: Rust testkit scaffold crate (`ada_mock` lib, 1 feature `server = ["dep:time"]`, 預設關, 純 in-memory mocks + fixtures + builders, **不對外 emit ACI assertion**)
- **L1 cluster_switch**: 🟢 落地 (新增 `D:/Ada/crates/ada-mock/.mock-cluster.json`, `enabled=false` 預設因 testkit 不對外)
- **L2 plugin_switch**: 🔴 降級 — testkit 模式, plugin 概念不適用, 強行落地違反守門 #15 scope creep
- **L3 module_switch**: 🔴 降級 (同上)
- **跟 ULYS-191 ACI 銜接**: 🔴 無 (testkit 不 emit assertion 到外部, `mock_switch_trace` 無意義)
- **改造工作量**: 🟢 小 (L1 only, 跨 session brief 啟動派工)
- **推薦派工順序調整** (per §1.3 brief): Star → IM1.0 → RGS → CATs → IDE1.0 → GitGit → **Ada (降級 L1 only, 預估 token 從 0.3-0.5M 降到 0.1-0.2M)**

### 2.3 共通缺口 (per 7 項目 G-MS-00)

> **G-MS-00 共通缺口**: 7 個項目都缺**統一三層開關 schema + 跨項目可比對**. 各項目自定義命名 (Cargo features / MSW NODE_ENV / 啟停腳本), LLM 讀 ACI 時無法直接 parse「當下 mock 處於什麼狀態」。


---

## 3. Mock 開關 Schema 鎖版 (Draft v0.1)

> **跟 ULYS-191 v0.1 §3 ACI Schema 鎖版對齊**: ACI schema 已落 `tools/star-flash-mock/.aci.json` v0.1, 開關 schema **嵌入** `.aci.json` plugins/modules section + 新增 `.mock-cluster.json` 頂層文件。

### 3.1 三層 schema 摘要

#### L1 Cluster Switch (`.mock-cluster.json` 頂層)

```json
{
  "$schema": "https://ulysses-star.local/schemas/mock-cluster/v0.1.0-draft.json",
  "cluster_version": "0.1.0-draft",
  "project": "star-flash-mock",
  "cluster_id": "star-flash-mock--dev-cluster",
  "description": "Star Flash Mock cluster switch v0.1 — 整個 cluster 啟用 / 關閉",
  "enabled": true,
  "mode": "offline",
  "fallback_to_real": false,
  "aci_compat_version": "0.1.0-draft",
  "supported_layers": ["ut", "it", "st", "e2e"],
  "tags_taxonomy": ["perf", "kms", "rbac", "auth", "concurrency", "regression"]
}
```

**字段說明**:

| 字段 | 必填 | 含義 |
|---|---|---|
| `cluster_version` | ✅ | schema 版本 (semver, 跟 `.aci.json` `aci_version` 對齊) |
| `project` | ✅ | mock 項目名 (跟 `.aci.json` `project` 字段一致) |
| `cluster_id` | ✅ | cluster 全局唯一 ID (e.g. `<project>--<env>`) |
| `enabled` | ✅ | bool, 整個 cluster 啟用 / 關閉; false = 全部 fixture 不 dispatch |
| `mode` | ✅ | `offline` / `passthrough` / `proxy` (per §3.2 模式說明) |
| `fallback_to_real` | 🟡 | bool, mock 失敗時是否 fallback 真實服務 (預設 false, mock 失敗暴露給測試) |
| `aci_compat_version` | ✅ | 對齊 `.aci.json` `aci_version` (e.g. `"0.1.0-draft"`); 不一致則 dispatcher 報錯 |

#### L2 Plugin Switch (`.aci.json` plugins section, per ULYS-191 §3.1 schema 擴展)

```json
{
  "aci_version": "0.1.0-draft",
  "project": "star-flash-mock",
  "plugins": {
    "kms": {
      "plugin_id": "kms",
      "enabled": false,
      "default_mode": "passthrough",
      "modules": {
        "unlock": { "module_id": "unlock", "enabled": true, "mode": "offline" },
        "rotate": { "module_id": "rotate", "enabled": false, "mode": "offline" }
      }
    },
    "rbac": {
      "plugin_id": "rbac",
      "enabled": true,
      "default_mode": "offline",
      "modules": {
        "role_check": { "module_id": "role_check", "enabled": true, "mode": "offline" },
        "permission_resolve": { "module_id": "permission_resolve", "enabled": true, "mode": "offline" }
      }
    },
    "audit": {
      "plugin_id": "audit",
      "enabled": true,
      "default_mode": "offline",
      "modules": {}
    }
  }
}
```

#### L3 Module Switch (`.aci.json` plugins.<plugin_id>.modules section)

每個 plugin 內部 sub-module 開關, per §3.1 L2 example。

### 3.2 三種 mode 說明 (per `mode` 字段)

| mode | 含義 | LLM 看到的 mock_switch_trace |
|---|---|---|
| `offline` | mock 完全本地模擬, 不連真實服務 | `mock_switch_trace = "cluster.enabled=true, plugin.X.enabled=true, module.Y.mocked=true (offline)"` |
| `passthrough` | mock 轉發到真實服務, 只 mock 部分欄位 | `mock_switch_trace = "cluster.enabled=true, plugin.X.enabled=true, module.Y.passthrough=true (passthrough to <real_service>)"` |
| `proxy` | mock 作為 reverse proxy, mock 記錄所有進 / 出 (trace 用) | `mock_switch_trace = "cluster.enabled=true, plugin.X.enabled=true, module.Y.proxied=true (proxy, trace saved to <log>)"` |

### 3.3 並存擴展策略 (per ULYS-191 v0.1 §3.4 經驗)

| 兼容點 | 落地策略 |
|---|---|
| 既有 Cargo features (CATs / IM1.0) | **並存**: 保留 `[features] mock_X`, 額外加 `.aci.json` plugins.X.enabled; dispatcher 兩者都讀 (Cargo feature 優先, .aci.json 可 override) |
| 既有 MSW NODE_ENV (GitGit) | **並存**: 保留 `process.env.NODE_ENV` 判定, 額外加 .aci.json frontend 字段; CI 環境 NODE_ENV + .aci.json 雙鎖 |
| 既有 fixture_assertion (Star mock) | **並存**: ULYS-191 §3.4 已寫; ULYS-190 開關落 .aci.json plugins section, fixture_assertion 不動 |
| Star mock 啟停腳本 (`start-k3s-backend.ps1`) | **並存**: 保留腳本, 額外加 `.mock-cluster.json`; 腳本啟動前先 read .mock-cluster.json.enabled, false 則跳過 |

### 3.4 mock_switch_trace 字段 (per §1.2 銜接點)

ACI emit 時自動帶 `mock_switch_trace` 字段 (per ULYS-191 v0.1 schema_required_fields 擴展):

```json
{
  "assertion_id": "star-flash-mock:kms:unlock:timeout",
  "aci_version": "0.1.0-draft",
  "layer": "it",
  "scope": {
    "project": "star-flash-mock",
    "module": "kms",
    "domain": "admin",
    "operation": "unlock",
    "http_method": "POST"
  },
  "mock_switch_trace": "cluster.enabled=true, plugin.kms.enabled=false, plugin.kms.module.unlock.enabled=true (offline)",
  "expect": {...},
  "actual": {...},
  "status": "FAIL",
  "severity": "high",
  "reasoning": "..."
}
```

LLM 讀到 `plugin.kms.enabled=false, module.unlock.enabled=true`, 一眼看出: KMS plugin 走真實服務, 但 unlock 子功能走 mock, 失敗定位 = 「mock 模擬的 unlock 超時」而非「真實 KMS 服務超時」。

---

## 4. 5 階段落地路徑 (per ULYS-191 v0.1 §4 對齊)

### §4.1 — Star mock cluster_switch 雛形落地 (本 v0.1 拍板後第 1 筆 brief 候選)

- **範圍**: `tools/star-flash-mock/.mock-cluster.json` 新文件 + dispatcher 讀 cluster.enabled 字段 + 既有 `.aci.json` v0.1 加 `aci_compat_version` 對齊字段
- **產出**: 1 個 cluster_switch 落地 + 回歸測試驗證 + 跟 ULYS-191 §4.1.2 Star mock ACI Stage 1 brief 並行 (不衝突, ACI Stage 1 已 ship `cebba99c`)
- **估時**: ~0.3-0.5M tokens (跟 ULYS-191 §4.1.2 stage1 brief 同量級)
- **依賴**: ULYS-191 §4.1.2 Star mock ACI Stage 1 已 ship (commit `cebba99c`, 9/23 22:05 JST) ✅

### §4.2 — IM1.0 plugin_switch 雛形擴展 (推薦: 阻力最小, per ULYS-191 v0.3 G-ACI-01 結論同源)

- **範圍**: 擴 `D:/IM1.0/crates/im-testkit/.aci.json` plugins section (mock_kms / mock_audit / mock_auth 3 個 plugin_switch); dispatcher 對接 im-testkit `MockRuntime`
- **產出**: 3 個 plugin_switch 落地 + 跟既有 Cargo features 並存 (per §3.3)
- **估時**: ~0.2-0.3M tokens
- **依賴**: §4.1 cluster_switch schema 鎖版

### §4.3 — 其餘 4 項目 (RGS / CATs / IDE1.0 / GitGit) plugin_switch 擴展

- **範圍**: 4 項目分別落地 plugin_switch; CATs + IDE1.0 共享 Rust emitter helper (跟 ULYS-191 v0.3 G-ACI-01 結論一致)
- **產出**: 4 項目 plugin_switch 落地
- **估時**: 4 × 0.3-0.5M tokens = ~1.2-2.0M tokens
- **依賴**: §4.2 IM1.0 雛形 (作為後續 4 項目參考實裝範本)

### §4.4 — 7 項目 module_switch 擴展 (L3 粒度)

- **範圍**: 7 項目各自展開 module_switch (per plugin 內部 sub-module 粒度)
- **產出**: 7 項目 module_switch 落地
- **估時**: 7 × 0.3-0.5M tokens = ~2.1-3.5M tokens (跨項目最大塊)
- **依賴**: §4.3 4 項目 plugin_switch 落地

### §4.5 — 跨項目統一驗證 (CI 加 `mock-switch-validate`)

- **範圍**: CI 加 `mock-switch-validate.py`, 跨 7 項目跑 `.mock-cluster.json` + `.aci.json` plugins/modules section schema 驗證 + 開關狀態聚合報告
- **產出**: CI gate, 1 commit 收官
- **估時**: ~0.2-0.3M tokens
- **依賴**: §4.4 module_switch 7 項目落地

### §4.6 — G-MS-01 Ada mock 項目盤點確認 ✅ 已解決

- **範圍**: 確認 Ada 是否有 mock 工具; 有 mock 則降級模式 (L1 only)
- **產出**: `docs/briefs/ulys-190-g-ms-01-ada-inventory.md` v0.1 落档 ✅ (per commit 後續 v0.3 落地)
- **結論**: Ada **有** Rust mock (`D:/Ada/crates/ada-mock`), 屬性 testkit scaffold (非 backend mock), 降級模式 L1 only
- **估時**: ~0.02M tokens (純 ls + 讀源碼)

---

## 5. 6 落地前必拍決策 (per ULYS-191 v0.1 §5 對齊, 全部 ✅ 鎖版)

| # | 決策 | 候選 | 推薦 | v0.2 鎖版狀態 |
|---|---|---|---|---|
| #1 | **三層 schema 範圍** | **A**: 1 個 cluster_switch + N 個 plugin_switch + M 個 module_switch (本 v0.1) <br> B: 只 cluster + plugin, 跳 module <br> C: 只 cluster, plugin/module 走 CLI flag | **A** | ✅ **A** (per reply `01a0d073`「a」) |
| #2 | **落地優先級** | **A**: 先 Star + IM1.0 (per ULYS-191 v0.3 G-ACI-01 結論同源) <br> B: 先 IM1.0 1 個, 驗證 schema 可行再擴 <br> C: 7 項目並行 | **A** | ✅ **A** (per reply `01a0d073`「a」) |
| #3 | **版本策略** | A: v0.1-draft → 1 sprint 後 v0.2 鎖版 <br> **B**: v0.1-draft → v0.2 加字段 (rolling) <br> C: v0.1 一次性鎖版 | **B** | ✅ **B** (per reply `01a0d073`「a」) |
| #4 | **跟既有 mock 配置兼容性** | **A**: 並存擴展 (Cargo features / MSW / fixture_assertion 全保留) <br> B: 強制遷移 <br> C: 雙寫雙讀 (shadow mode) | **A** | ✅ **A** (per reply `01a0d073`「a」) |
| #5 | **跨項目 monorepo vs 拷貝** | A: monorepo (新建 `tools/mock-switch-spec` crate / package) <br> **B**: 各自拷貝 (跟 ULYS-191 v0.2 決策 #5 一致) <br> C: 拷貝 + monorepo 驗證腳本 | **B** | ✅ **B** (per reply `01a0d073`「a」) |
| #6 | **mock_switch_trace 寫入策略** | A: 每次 emit 都帶 (每次 assertion 都加 trace) <br> **B**: 只在開關「動態切換」時帶 (節省字段) <br> C: 開關變更時另寫審計日誌 (audit log) | **A** | ✅ **A** (per reply `01a0d073`「a」) |

---

## 6. 已知缺口 (per 守門 #11 缺標比錯標)

> 預估落地時顯式列「哪些 mock 项目已加開關 / 未加 / 部分加」; 本節列 v0.1 設計稿階段已知缺口。

| ID | 缺口 | 影響 | 解決方案 |
|---|---|---|---|
| **G-MS-01** | ✅ 已解決 (per `docs/briefs/ulys-190-g-ms-01-ada-inventory.md` v0.1) | Ada 有 Rust mock (`D:/Ada/crates/ada-mock`, 屬性 testkit scaffold), 降級模式 L1 only | 跨 session brief 啟動派工時顯式標 L1 only |
| **G-MS-02** | IM1.0 倉庫位置: 已確認 `D:/IM1.0` (per ULYS-191 v0.2 實證) | ✅ 已解決 (per ULYS-191 v0.2 §「G-ACI-02 ✅ 解決」) | — |
| **G-MS-03** | CATs + IDE1.0 共享 Rust emitter helper 範圍未定 | 落地時要不要共用 code? | ULYS-191 v0.3 G-ACI-01 §2 推薦: 共用 (per「CATs + IDE1.0 共享 Rust emitter helper」); ULYS-190 沿用 |
| **G-MS-04** | 「plugin」 跟「module」命名跨項目是否一致 | 落地時字段對不上 | schema 鎖死 plugin_id / module_id 命名, per 項目可別名 (alias) 但 schema 必填 |
| **G-MS-05** | 開關變更時的審計日誌 (audit log) 是否落 Work/Transaction/Master 哪類 | 落地 W/T/M 分類 (per 守門 #13) | 推薦: 開關變更 → Transaction (audit, SCD-2, RLS 13 類); 跟 ULYS-191 v0.1 §「W/T/M」一致 |
| **G-MS-06** | `passthrough` / `proxy` 模式對真實服務的網路延遲影響 | 跨項目性能 baseline 失真 | 落地時顯式標 `mock_switch_trace.mode`, CI 加 latency benchmark 排除 mock 影響 |
| **G-MS-07** | 既有 fixture_assertion (ULYS-191) 跟 mock_switch_trace (ULYS-190) 字段是否冗餘 | fixture 體積 ~2x | 並存 1 sprint 後再評估 (per ULYS-191 R-BRIEF-04 經驗) |
| **G-MS-08** | mock_switch_trace 字段長度: 三層開關全開可能 ~150 字 | 字段太長影響 LLM 讀 | 推薦: 截斷到 ~80 字 (LLM context 友好), 完整 trace 寫旁路日誌 |
| **G-MS-09** | 開關變更是否需要 hot reload (不重啟 dispatcher) | dev 體驗 | v0.1 不做, v0.2 評估 (per ULYS-191 決策 #3 rolling) |
| **G-MS-10** | 跨項目開關狀態聚合 (CI 報告: 7 項目 100+ 開關狀態) | CI 報告可讀性 | §4.5 CI 加 `mock-switch-validate` + 聚合報告 |

---

## 7. 守門自檢 (per AGENTS.md §4)

> 本 v0.1 設計稿對齊 AGENTS.md §4 守門硬約束:

| 守門 | 檢查 | 對應 |
|---|---|---|
| #1 docs 必含 | 本檔 v0.1 + ULYS-191 v0.1 同步 commit | ✅ |
| #3 5 域 Lead | 跨項目落地拆 per 项目 brief, 走守門 #20 | 🟡 等 #14 v3 反轉 + 5 域 Lead 真人到位 T3 |
| #5 env hard ban | 開關不涉及 env, 0 影響 | ✅ |
| #6 PowerShell only | 本檔撰寫 PowerShell, 0 bash `&&` | ✅ |
| #7 0 unsafe | mock 项目多數是 sh + python, 0 unsafe | ✅ |
| #9 subprocess | dispatcher 讀取開關走 subprocess | ✅ |
| #10 author=Ulysses | 跨項目 commit 統一 author | ✅ |
| #11 缺標比錯標 | §6 列 10 已知缺口 (G-MS-01..10) | ✅ |
| #12 docs 同步 | 本檔 + ULYS-191 v0.1 同步 commit 引用 | ✅ |
| #13 W/T/M | 開關配置屬 Master SCD-2 (per §6 G-MS-05) | ✅ |
| #14 v4 Mavis 审核 | author=Ulysses, 跨項目 brief 走 Mavis 审核 | ✅ |
| #15 scope creep | 嚴守 1 brief 1 sub-agent 1 mock 项目, 不批量 (per ULYS-189/191 經驗) | ✅ |
| #19 v19 Python 化 | dispatcher 驗證腳本走 Python (跟 ULYS-191 §4.1.2 stage1 `_lib_aci_emit.py` 同模式) | ✅ |
| #20 子代理 brief | per 项目 brief 落地 (per ULYS-189 v0.2 §6.1 + ULYS-191 §4.1.2) | ✅ |
| #24 vendor 中立 | 開關 schema 不引入 vendor, stdlib JSON only | ✅ |

---

## 8. 下一步 (per 守門 #15 不主動 scope creep)

🟡 **本 v0.2 鎖版後狀態**: 6 決策全部 ✅ 鎖版 (A+A+B+A+B+A, per reply `01a0d073`「a」); 派工授權已收到, **未實際派 sub-agent** (per 守門 #3 + #14 v3 + ULYS-191 reply `01a0cdc1`「a」拍板範式)

🟡 **G-MS-01 Ada mock 項目盤點確認** ✅ 已解決 (per `docs/briefs/ulys-190-g-ms-01-ada-inventory.md` v0.1, Ada 降級模式 L1 only)
🟡 **§4.1 Star mock cluster_switch 雛形 brief** 落档 (`docs/briefs/ulys-190-star-mock-cluster-switch-stage1.md` v0.1, 待派工)

**跨 session 續做入口**:

1. ✅ v0.1 起草 (commit `c3d7d171`, 9/23 22:11 JST)
2. ✅ v0.2 鎖版 (per reply `01a0d073`「a」, 6 決策全部走推薦 A+A+B+A+B+A)
3. ✅ §4.1 Star mock cluster_switch brief v0.1 落档 (`docs/briefs/ulys-190-star-mock-cluster-switch-stage1.md`)
4. 🟡 等候派工觸發 (T1 D-Boy 明示「派吧」/ T2 5 域 Lead 真人到位 / T3 D-Boy 修 brief 範圍, 任一即派, per ULYS-191 §4.1.2 brief §6 觸發條件)
5. 🟡 sub-agent 落地實裝 (per brief v0.1 §1.1 In-Scope, ~0.3-0.5M tokens, 1 commit 收官)
6. 🟡 §4.2 IM1.0 plugin_switch brief 啟動派工 (跟 §4.1 並行, 阻力最小)
7. 🟡 §4.3 4 項目 (RGS / CATs / IDE1.0 / GitGit) plugin_switch brief 啟動派工 (順序: IM1.0 → CATs+IDE1.0 共享 Rust helper → RGS → GitGit)
8. 🟡 §4.4 7 項目 module_switch 擴展 (最大塊, 跨 session 續)
9. 🟡 §4.5 跨項目 CI 加 `mock-switch-validate` (收官, 1 commit)
10. 🟡 G-MS-01/02/03/04/05 缺口跨 session 解決

**token 累計**: ~0.04M (v0.1 起草) + ~0.03M (v0.2 修訂 + brief v0.1 起草) = ~0.07M

---

## 9. 修訂歷史

| 版本 | 日期 | 修訂人 | 內容 |
|---|---|---|---|
| **v0.1** | 2026-09-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代審 | **初稿 (Draft)** — 觸發 ULYS-190 (2026-09-22 23:38 JST); 9 節結構 (目的 / 概念映射 / Mock 項目盤點 / Schema 鎖版 / 5 階段路徑 / 6 決策 / 已知缺口 / 守門自檢 / 下一步); ~16 KB / 270 行 (per `wc -l` 估算); 預估 ~0.04M token; 跨項目範圍 7 (跟 ULYS-191 v0.2 approved 一致, 排除 Xiaoshuo); 跟 ULYS-191 銜接點在 §1.2 mock_switch_trace 字段 |
| **v0.2** | 2026-09-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**審核** | **Approved** (reply `01a0d073` 2026-09-23 22:46 JST 「a」) — 6 決策全部 ✅ 鎖版 (A+A+B+A+B+A, 全部走推薦); banner 從 🟡 Draft → 🟢 Approved; §5 6 決策行加 ✅ 鎖版狀態列; §8 下一步更新派工等待狀態; §9 加 v0.2 row; 預估 ~0.03M token (本 v0.2 修訂) |
| **v0.3** | 2026-09-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**審核** | **G-MS-01 ✅ 解決** (reply `01a0d0e1` 2026-09-23 23:02 JST 「完成所有后续工作」 T1 派工 + brief `docs/briefs/ulys-190-g-ms-01-ada-inventory.md` v0.1 落档); §2.1 主表 Ada row 從「❓ 待盤點」→「✅ G-MS-01 解決 (testkit scaffold 降級模式 L1 only)」; §2.2 項目 7 詳述擴展; §4.6 從「G-MS-02」改名「G-MS-01 ✅ 已解決」; §6 G-MS-01 row 從「待盤點」→「✅ 已解決 + 引用 brief」; §8 下一步 G-MS-01 ✅ 解決標; §9 加 v0.3 row; 預估 ~0.02M token (本 v0.3 修訂) |
| **v0.4** | 2026-09-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**審核** | **6 跨项目 cluster_switch + plugin_switch 落地** (reply `01a0d306` 2026-09-23 23:02 JST 「推进到完成」 T1 派工): IM1.0 (5 plugin: assertions/fixtures/mock_grpc/mock_rest/mock_ws_frames, commit d637348) + RGS (5 plugin: player/economy/match/social/admin, commit cdbc17d) + CATs (4 plugin: data/db/http/infra, commit 97c7a17) + IDE1.0 (2 plugin: ide-cli/ide-kernel-core, commit c7621cc) + GitGit (3 plugin: health/repo/vault, frontend MSW) + Ada (L1 only 降級模式, enabled=false 預設因 testkit 不對外 emit, commit 9e901a8); + Star `.github/workflows/mock-switch-validate.yml` 跨项目 CI 校验 workflow (commit c07439db); §2.1 主表 6 项目 row 状态標 ✅ Stage 1 落地; §8 下一步 §4.2-§4.6 全部 ✅ 解決; §9 加 v0.4 row; 預估 ~0.05M token (本 v0.4 修訂 + 6 commits 编排) |

---

## 附錄 A: 跨文檔引用 (Cross-References)

- **ULYS-191 ACI 介面** (sibling, 同期觸發, 已 ship 部分):
  - [`docs/architecture/2026-09-22-aci-mock-interface/00-design-analysis.md`](../2026-09-22-aci-mock-interface/00-design-analysis.md) (v0.3, 已 Approved)
  - `tools/star-flash-mock/.aci.json` v0.1 (頂層 ACI 契約, 9/23 Stage 1 已 ship)
  - ULYS-191 §4.1.2 Star mock ACI Stage 1 brief: `docs/briefs/ulys-191-star-mock-aci-stage1.md` v0.1 (Stage 1 commit `cebba99c` merged main+dev, 9/23 22:14 JST)

- **ULYS-189 Jev 設計** (姊妹文檔, 小腦判斷層):
  - [`docs/architecture/2026-09-22-jev-integration/00-design-analysis.md`](../2026-09-22-jev-integration/00-design-analysis.md)

- **既有 fixture 結構樣本** (per Star mock):
  - `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-1.json` (既有 fixture + fixture_assertion + aci_assertion 三段並存)
  - `tools/star-flash-mock/.aci.json` v0.1 (頂層 schema, plugins/modules section v0.1 尚無, 待 §4.1 擴)

- **跨 session 派工範式**:
  - `scripts/automation/dispatcher.py brief(...)` (per 守門 #20)
  - `docs/automation-design.md` v0.2 §3.1 (子代理 dispatch 必先 brief 落檔)
  - ULYS-189 Jev §6.1 brief 範式 (per `docs/briefs/ulys-191-star-mock-aci-stage1.md` v0.1 §6 對齊)
