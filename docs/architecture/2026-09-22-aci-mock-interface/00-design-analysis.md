# ACI (Assertion-Capability Interface) × 全项目 Mock — 設計分析 (Design Analysis)

> **狀態**: 🟡 Draft v0.1 (ULYS-191 觸發, 待 Ulysses 拍板 6 落地前決策)
> **日期**: 2026-09-22
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代審
> **觸發 issue**: ULYS-191 "ACI接口" — 所有项目的 Mock 项目要具备 ACI 接口，有效输出提示词型断言，方便大模型判断问题
> **適用項目**: **跨項目範圍** (per description 字面 "所有项目" — Star / RGS / CATs / IDE1.0 / IM1.0 / GitGit / Xiaoshuo / Ada 共 8 個項目), 設計稿落地第一階段以 Star `tools/star-flash-mock/` 為參考實裝, 跨項目介面契約由本檔統一
> **關聯文檔**:
> - [`docs/architecture/2026-09-22-jev-integration/00-design-analysis.md`](../2026-09-22-jev-integration/00-design-analysis.md) (姊妹文檔, ULYS-189, 小腦判斷層)
> - ULYS-190 "Mock开关" (sibling issue, 同期觸發, 觸發測試目標功能開關)
> - ULYS-135/136/137/138/139/140/141 七个 Mock 回归测试 issue (per 项目)
> - `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-1.json` 既有 `fixture_assertion` schema (boolean-only, 缺 prompt-style 文本)
>
> ---

## 0. 目的 (Purpose)

ULYS-191 description 字面: **「所有项目的 Mock 项目要具备 ACI 接口，有效输出提示词型断言，方便大模型判断问题」**。
2026-09-23 09:53 JST reply `01a0cbc1` 進一步拍板: **「我的目的是改造我所有的 mock 项目」**。

本文檔**不**給最終實裝方案 — ULYS-191 是「改造」類 issue, 落地需 per 项目 brief 拆解 (per 守门 #15 scope creep)。
本文檔目標:

1. **建立共同術語表**: ACI / 提示词型断言 / Mock 项目 / 断言 schema 之間的概念映射
2. **盤點 8 個 Mock 項目現狀**: 每個 mock 項目當前 emit 什麼 assertion, 是否 LLM-可讀, 缺什麼
3. **ACI 介面契約 (Draft v0.1)**: 統一 schema (id / expect / actual / reasoning / suggested_fix / severity / evidence_ref), 跨項目一致
4. **跨項目改造路徑**: 8 個項目如何排程落地, 是 monorepo 共享 crate 還是各自拷貝, 守门 #15 + #20 子代理 brief 拆解
5. **6 個落地前必拍決策**: 不決不寫代碼 (per ULYS-189 v0.1 經驗)
6. **守門自檢**: 跟 AGENTS.md §4 守門硬約束對齊

> **ACI 縮寫解讀** (per description 字面 + context, 不查外部):
> **A**ssertion-**C**apability **I**nterface = 斷言能力介面。
> 「提示词型断言」= 不是 boolean (`assertEquals(actual, expected)`) 而是自然語言 + 結構化字段
> (`expected_response_within_2s` / `actual_response_took_5s` / `reasoning="API 调用超时, 可能下游服务 GC"` / `suggested_fix="检查下游服务的 GC 日志, 或增加 retry with exponential backoff"`)。
> 「方便大模型判断问题」= 不只是給人看的, **是設計給 LLM 看** — 字段要能被 LLM 直接讀懂, 不需要先解釋。

---

## 1. 概念映射 (Concept Mapping)

### 1.1 「提示词型断言」是什麼 — 三層拆解

| 層 | 當前主流做法 | 提示词型断言 (per ULYS-191 改造目標) |
|---|---|---|
| **結構** | `{pass: true, elapsed_ms: 5}` (boolean + 數字) | `{assertion_id, expect, actual, severity, reasoning, suggested_fix, evidence_ref, tags}` (8-12 字段) |
| **語義** | 機器讀: pass/fail | **LLM 讀**: 不只判 fail, 還給出「為什麼 fail + 怎麼修」 |
| **輸出形式** | stdout `[OK]/[FAIL]` 一行字串 | **結構化 JSON / YAML** 落 `<assertion>.json` 文件, 可被 LLM 直接讀取 + 推理 |
| **錯誤定位** | 「FAIL: regression-test-five-domain.sh §2 line 87」 (需人去 trace) | 「FAIL: 5 域 admin 域 list 操作超時, 實測 5s > 預期 2s, 懷疑下游 KMS 解鎖慢」 (LLM 直接讀懂) |
| **觸達路徑** | 開發者手動跑 regression 看 log | **LLM agent 自動消費**, 用於測試結果總結 / 自動修 bug / 自動開 issue |

**關鍵區別**: 當前 mock 项目的 assertion 是 **machine-readable** (程式能 parse);
改造目標是 **LLM-readable** (大模型能讀懂 + 推理 + 給建議)。

### 1.2 「ACI 介面」是什麼 — 介面契約

**ACI = Assertion-Capability Interface**: 一個 mock 项目對外暴露的「断言能力介面」。
包含三部分:

1. **Schema (`.aci.json`)** — 每個 mock 项目根目錄下放一個 ACI 配置文件, 聲明:
   - 該项目能 emit 哪些類型的 assertion (per-layer: ut/it/st/e2e/load)
   - 每個 assertion 的 schema (字段名 + 類型 + 含義 + 示例)
   - 字段對齊 LLM prompt template (讓 LLM 知道每個字段怎麼解讀)

2. **Emitter (代碼層)** — 每個 mock 项目在跑 test/fixture 時, 不只寫 `[OK]/[FAIL]`, 同時落結構化 `assertions/<test-id>.aci.json`:
   ```json
   {
     "assertion_id": "star-flash-mock/five-domain/admin/01-list/GET/2026-09-23T09:53:40Z",
     "aci_version": "0.1.0-draft",
     "layer": "it",
     "scope": {
       "project": "star-flash-mock",
       "module": "five-domain",
       "domain": "admin",
       "operation": "list",
       "method": "GET"
     },
     "expect": {
       "type": "response_within_ms",
       "value": 2000,
       "description": "admin 域 list 操作应在 2 秒内返回"
     },
     "actual": {
       "type": "response_within_ms",
       "value": 5000,
       "description": "实測 5 秒返回, 超预期 2.5 倍"
     },
     "status": "FAIL",
     "severity": "high",
     "reasoning": "API 调用超时, 推测下游 KMS 解锁模块阻塞. 实測 KMS 端点 ping 50% 超时.",
     "suggested_fix": "检查 KMS 服务的 GC 日志, 或增加 retry with exponential backoff (3-6-12-24s). 参考 star-flash-mock/docs/kms-tuning.md.",
     "evidence_ref": "tools/star-flash-mock/logs/2026-09-23T09-53-40Z-five-domain-admin-01.log",
     "tags": ["performance", "kms", "timeout", "five-domain"]
   }
   ```

3. **Consumer (讀取端)** — LLM agent / IDE / CI 直接讀取 `<assertion>.aci.json`:
   - 一個 assertion 是「一個 LLM 可讀的事件」
   - 多個 assertion 組成「一份 LLM 可讀的測試報告」
   - LLM 可以「閱讀 + 推理 + 給建議」

### 1.3 跟 AGENTS.md §4 守門的關係

| 守門 | 跟 ACI 的關係 |
|---|---|
| #1 docs 必含 | ACI schema 本身就是 doc (`.aci.json` + `aci-spec.md`), 落地時同步 commit |
| #3 5 域 Lead | 跨項目落地拆 per 项目 brief, 走守門 #20 子代理 brief |
| #5 env hard ban | ACI 不涉及 env, 0 影響 |
| #7 0 unsafe | mock 项目多數是 sh + python, 0 unsafe |
| #9 subprocess | LLM agent 讀取 assertions 走 subprocess (per ULYS-189 v0.2 §6.1) |
| #10 author=Ulysses | 跨項目 commit 統一 author |
| #11 缺標比錯標 | 落地時顯式列「哪些 mock 项目已改造 / 未改造 / 部分改造」 |
| #12 docs 同步 | ACI spec 落 `docs/spec/aci-spec-v0.1.md`, commit 引用 |
| #13 W/T/M | assertions 文件落 Master SCD-2 (累積變更歷史) per 守門 #13 c |
| #14 v4 Mavis 审核 | author=Ulysses, 跨項目 brief 走 Mavis 审核 |
| #15 scope creep | 嚴守 1 brief 1 sub-agent 1 mock 项目, 不批量 |
| #20 子代理 brief | per 项目 brief 落地 (per ULYS-189 v0.2 §6.1 經驗) |
| #24 vendor 中立 | ACI 自身不引入 vendor, 只用 stdlib JSON / serde |

---

## 2. 跨項目 Mock 盤點 (Cross-Project Inventory)

ULYS-191 description 字面 "所有项目" = ULYS 一人公司 8 個項目。盤點現狀如下:

| # | 項目 (Repo) | 主要 mock 工具 | 當前 assertion 形式 | LLM-可讀? | 改造工作量 |
|---|---|---|---|---|---|
| 1 | **Star** (`D:/Star`) | `tools/star-flash-mock/` (per ULYS-140) | `[OK]/[FAIL]` stdout + `fixture_assertion: {boolean, boolean}` JSON (per `mock_data/agent-runtime/guards/v1--guard--g-*.json`) | ⚠️ 部分 (有 schema 但 0 prompt-style) | 🟡 中 (mock 已結構化, 加 ACI emitter 即可) |
| 2 | **IDE1.0** (`E:/IDE1.0`) | 8 個 `crates/*/tests/integration.rs` + `tests/st/cli_smoke.sh` (per ULYS-139) | `assert!()` / `assert_eq!()` (Rust 原生), stdout `PASS/FAIL` | ❌ 不可 (純 boolean + panic msg) | 🟡 中 (Rust 寫 ACI emitter helper crate) |
| 3 | **RGS** (`D:/RustGameServer`) | 待盤點 (per ULYS-136) | 待盤點 | ❓ 未知 | 🔴 待盤點 |
| 4 | **CATs** (`D:/CATs`) | 待盤點 (per ULYS-135) | 待盤點 | ❓ 未知 | 🔴 待盤點 |
| 5 | **IM1.0** (`待 git clone`) | 待盤點 (per ULYS-138) | 待盤點 | ❓ 未知 | 🔴 待盤點 |
| 6 | **GitGit** (`D:/GitGit`) | 待盤點 (per ULYS-137) | 待盤點 | ❓ 未知 | 🔴 待盤點 |
| 7 | **Xiaoshuo** (`D:/Xiaoshuo`) | 待盤點 (per ULYS-141) | 待盤點 | ❓ 未知 | 🔴 待盤點 |
| 8 | **Ada** (`D:/Ada`) | 不確定是否有 mock 项目 (per ULYS-137 同類) | 待盤點 | ❓ 未知 | 🟢 0 (可能無 mock) |

**已知缺口 (per 守門 #11 缺標比錯標)**:
- **G-ACI-01**: 項目 #3-#7 5 個項目**未盤點**, v0.1 落地前需每個項目一個 worker 子代理跑 `find . -name '*mock*' -type d | xargs ls` 盤點 (per 守門 #20)
- **G-ACI-02**: IM1.0 (`E:/IM1.0` per memory) 是否存在需確認, 否則 8 個項目中 1 個可能不存在
- **G-ACI-03**: 跨項目 mock 工具語言不一致 (Star = sh+python / IDE1.0 = Rust / 其他未知), ACI emitter 需支援多語言 (≥3: sh + python + rust), 否則逐個寫

---

## 3. ACI Schema v0.1 (Draft Spec)

### 3.1 頂層 `.aci.json` 配置 (per mock 项目根)

每個 mock 项目根目錄下放 `.aci.json`, 聲明該项目對外暴露的 ACI 契約:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "aci_version": "0.1.0-draft",
  "project": "star-flash-mock",
  "project_repo": "D:/Star",
  "spec_doc": "docs/architecture/2026-09-22-aci-mock-interface/00-design-analysis.md",
  "emitter_languages": ["bash", "python"],
  "emitter_entrypoints": {
    "bash": "tools/star-flash-mock/scripts/_lib_aci_emit.sh",
    "python": "tools/star-flash-mock/scripts/_lib_aci_emit.py"
  },
  "output_dir": "tools/star-flash-mock/assertions/",
  "supported_layers": ["ut", "it", "st", "e2e"],
  "supported_assertion_types": [
    "response_within_ms",
    "response_status_2xx",
    "response_field_equals",
    "response_field_in_set",
    "fixture_matches_schema",
    "no_5xx_in_window",
    "rate_limit_within_quota",
    "guardrail_decision_allows",
    "guardrail_decision_denies",
    "mock_state_advances"
  ],
  "fields_per_assertion": [
    "assertion_id",
    "aci_version",
    "layer",
    "scope",
    "expect",
    "actual",
    "status",
    "severity",
    "reasoning",
    "suggested_fix",
    "evidence_ref",
    "tags",
    "captured_at"
  ],
  "severity_levels": ["critical", "high", "medium", "low", "info"],
  "prompt_template_hint": "每個 assertion.aci.json 可直接餵給 LLM, 字段含義 self-explanatory (per 1.2 example)"
}
```

### 3.2 字段語義 (LLM-可讀性自檢)

每個字段**單獨**能被 LLM 解讀, 無需預訓練上下文:

| 字段 | 類型 | 必填 | LLM 視角語義 |
|---|---|---|---|
| `assertion_id` | string | ✅ | 全局唯一 ID, 用於 trace + 去重 |
| `aci_version` | string | ✅ | schema 版本, 將來 LLM 看到 v0.2 知道字段含義有變 |
| `layer` | enum `ut/it/st/e2e` | ✅ | 測試層, 幫 LLM 判斷嚴重程度 |
| `scope` | object | ✅ | 標明「哪個项目 / 模塊 / 域 / 操作 / HTTP 方法」, 幫 LLM 定位 |
| `expect` | object | ✅ | **人類可讀**的「預期是什麼」 (`{type, value, description}`) |
| `actual` | object | ✅ | **人類可讀**的「實測是什麼」 (`{type, value, description}`) |
| `status` | enum `PASS/FAIL/WARN/SKIP` | ✅ | 結果, 跟當前 `[OK]/[FAIL]` 對齊 |
| `severity` | enum `critical/high/medium/low/info` | ✅ | 失敗的嚴重程度 (LLM 優先處理 critical) |
| `reasoning` | string | ⚠️ FAIL 時必填 | **「為什麼 fail」**, 自然語言, LLM 直接讀 |
| `suggested_fix` | string | ⚠️ FAIL 時建議填 | **「怎麼修」**, 自然語言, LLM 可直接採納 |
| `evidence_ref` | string | ❌ | 指向 log / 文件 / URL, LLM 可 deep-dive |
| `tags` | array of string | ❌ | 分類標籤, LLM 可用於 group + 統計 |
| `captured_at` | RFC3339 string | ✅ | 時間戳, LLM 判斷「最近變差」還是「一直 fail」 |

### 3.3 `expect` / `actual` 統一格式

```yaml
# Type 1: 響應時間
{type: "response_within_ms", value: 2000, description: "应在 2 秒内返回"}

# Type 2: HTTP 狀態
{type: "response_status_2xx", value: 200, description: "应返 2xx 状态码"}

# Type 3: 字段相等
{type: "response_field_equals", path: "$.data.role", value: "admin", description: "角色字段应等于 'admin'"}

# Type 4: 字段枚舉
{type: "response_field_in_set", path: "$.data.status", value_set: ["active", "archived", "completed"], description: "状态字段应在 3 个合法值内"}

# Type 5: 護欄決策
{type: "guardrail_decision_denies", guard_id: "G-3", description: "護欄 G-3 应拒绝此调用 (per SRS-001 G-3)"}

# Type 6: Mock 狀態推進
{type: "mock_state_advances", from: "Pending", to: "Approved", description: "审批状态应从 Pending 推进到 Approved"}
```

**新增類型**: 落地時如有未覆蓋類型, 按需加, 但需更新 `.aci.json` 的 `supported_assertion_types`。

### 3.4 與現有 `fixture_assertion` 的兼容性策略

Star mock 既有 `fixture_assertion: {guard_check_pass: bool, elapsed_under_10ms: bool}` 結構。
v0.1 改造策略 = **並存擴展**, 不破壞既有 schema:

- **舊字段保留**: `fixture_assertion` 仍寫入, 保持既有 `validate.py` 可用
- **新增 ACI 並行**: 同一個 fixture 文件同時寫 `aci_assertion: {8-12 字段}` (per §3.2)
- **過渡期**: 2 個版本並存, 落地後 1 個 sprint 觀察, 後續刪舊

---

## 4. 跨項目落地路徑 (Implementation Path)

ULYS-191 = 「改造我所有的 mock 项目」(per reply `01a0cbc1` 2026-09-23 09:53 JST)。
落地需跨項目、跨語言, 不能 1 個 brief 1 個 sub-agent 8 個项目全包 (per 守門 #15 scope creep)。
建議拆 5 階段:

### 4.1 階段 0: ACI Schema 鎖版 + Star 第 1 筆 brief (本 v0.1 落地後)

| 子項 | 內容 | 工時 |
|---|---|---|
| **§4.1.1** | 本 v0.1 拍板 → v0.2 (決策 #1-#6 鎖版, per §6) | 1 sub-session |
| **§4.1.2** | 第 1 筆 brief: **Star mock `tools/star-flash-mock/` 第 1 階段** (per 守門 #20 + #15) | 1 sub-agent brief |
| **§4.1.3** | Star `.aci.json` + `_lib_aci_emit.sh` + `_lib_aci_emit.py` + 第 1 個 sample assertion.aci.json 落地 | 含在 §4.1.2 brief 內 |
| **§4.1.4** | 落 commit `agent/minimaxm3/ulys-191` (Star 主倉) | 1 commit |

**Star 第 1 階段** = Star mock 全部 `[OK]/[FAIL]` 點加 ACI emit, 預估 ~50 個 assertion 點, ~0.3-0.5M tokens。

### 4.2 階段 1: IDE1.0 mock + Rust emitter helper crate

| 子項 | 內容 |
|---|---|
| **§4.2.1** | Rust crate `aci-emitter` (輕量, ~100 LOC, 只 emit JSON) |
| **§4.2.2** | IDE1.0 8 個 integration test + cli_smoke.sh 加 ACI emit |
| **§4.2.3** | IDE1.0 倉 `agent/minimaxm3/ulys-191` 分支 |

### 4.3 階段 2: 其餘 6 個項目盤點 + 排程

待 G-ACI-01 (項目 #3-#7 5 個未盤點) 解決後, 確定每個项目:
- mock 工具位置 / 語言 / 現有 assertion 形式
- 改造工作量 (中 / 大 / 微)
- 排程 (跟其他 brief 鏈)

**RGS / CATs / IM1.0 / GitGit / Xiaoshuo 5 個项目**, 預估每個 1-2 個 sub-agent brief (~1.5-2.5M tokens)。

### 4.4 階段 3: ACI 統一 Consumer (LLM 讀取端)

`tools/aci-summary/` 新工具 (per 项目), 給 LLM agent 一行指令讀懂所有 mock 結果:
```bash
$ aci-summary tools/star-flash-mock/assertions/2026-09-23T09-53-40Z/
# 自動聚合 + 給 LLM-friendly summary:
# - 5 critical fails
# - 12 high fails  
# - 23 medium warns
# - 187 pass
# Top 3 issues:
# 1. five-domain/admin/list 超時 (12/12 instances fail)
# 2. agent-runtime/l0/G-3 護欄決策不一致 (8/8 instances fail)
# 3. db-wtm/transaction/init 字段校驗失敗 (5/5 instances fail)
```

### 4.5 階段 4: 跨項目統一驗證

- CI 層加一條: 跨項目跑 `aci-validate`, 確保所有 `.aci.json` 文件符合 schema
- Star / IDE1.0 / 其他项目 統一進 CI
- 跨項目 brief chain 收官

**預估總工時**: ~3-5M tokens (8 個项目 + ACI spec + consumer), per 守門 #19 v19 Python 化分攤。

---

## 5. 落地前必拍 6 個決策 (per ULYS-189 v0.1 經驗)

不決不寫代碼。建議 Ulysses 拍板方向, v0.2 鎖版:

| # | 決策 | 推薦選項 | 理由 |
|---|---|---|---|
| **#1** | **ACI schema 範圍** | **A**: 8 個字段 (`assertion_id / scope / expect / actual / status / severity / reasoning / captured_at` 必填, 其餘可選) | 必填 8 個是 LLM 推理最小集, 可選 5 個按需豐富; 避免 v0.1 過度設計 |
| **#2** | **落地優先級** | **B**: 先 Star `tools/star-flash-mock/` 第 1 階段 (per §4.1), 再 IDE1.0, 再其餘 6 個项目 | Star mock 已結構化 + ULYS-140 已 ship, 阻力最小; IDE1.0 第二; 其餘需先盤點 (G-ACI-01) |
| **#3** | **ACI 版本策略** | **A**: v0.1-draft 起步, 1 個 sprint 後拍 v0.2 (鎖版), 之後 v0.3+ 才允許加字段 | 跟 ULIS-189 Jev schema 同型, 避免 v0.x 字段漂移 |
| **#4** | **跟現有 fixture_assertion 兼容性** | **A**: 並存擴展 (§3.4), 2 版本同時寫, 1 sprint 後觀察再決定刪舊 | 不破既有 `validate.py`, 降低落地風險 |
| **#5** | **跨項目 monorepo 還是各自拷貝** | **B**: 各自拷貝 `_lib_aci_emit.sh/.py/.rs` (每項目獨立版本) | 8 個项目語言 + 版本不一致, monorepo 維護成本高; 拷貝 + 同源 schema 即可 (per 守門 #19 v19 Python 化) |
| **#6** | **LLM agent 讀取入口** | **A**: 每項目獨立 `.aci.json` + `aci-summary` CLI (per §4.4); LLM 通過 CLI 讀 | LLM 直接讀分散的 `.aci.json` 太碎, 給 CLI 聚合; 不引入外部 vendor (per 守門 #24) |

---

## 6. 已知缺口 (per 守門 #11 缺標比錯標)

| # | 缺口 | 風險 | 對齊方式 |
|---|---|---|---|
| **G-ACI-01** | 項目 #3-#7 (RGS / CATs / IM1.0 / GitGit / Xiaoshuo) mock 項目未盤點 | 改造工作量預估不準 | v0.2 後, 每項目 1 個 worker brief 跑 `find . -name '*mock*'` |
| **G-ACI-02** | IM1.0 倉庫位置未確認 (`E:/IM1.0` per memory vs 8 個項目清單) | 可能 8 → 7 個項目 | v0.2 前先 `ls /e/IM1.0 2>/dev/null` 確認 |
| **G-ACI-03** | 跨語言 emitter 需 sh + python + rust ≥ 3 種 | 落地時每種語言各寫 1 個 helper, 工時 ~3x | v0.2 拍板決策 #5 (拷貝 vs monorepo) 後明確 |
| **G-ACI-04** | LLM prompt template hint 寫成字段 (`prompt_template_hint` per §3.1), 但實際 LLM 提示工程需實驗 (prompt 寫法影響判斷準確率) | v0.1 hint 可能是「猜」, 真實 LLM 讀可能漏抓字段 | 第 1 階段 (Star mock) 落地後跑 10 條真實 LLM agent dry-run 驗證 |
| **G-ACI-05** | 並存擴展期間 (`fixture_assertion` + `aci_assertion` 兩份), 文件體積 2x | 8 個项目 fixture 全量 × 2 = 翻倍, CI 跑分變慢 | 1 sprint 後觀察, 驗證 ACI 落後刪舊 (`fixture_assertion`) |
| **G-ACI-06** | Star mock 已有 `mock_data/agent-runtime/guards/v1--guard--g-*.json` 175 份 fixture, 全量加 ACI 字段後, diff 大 | 單 commit 文件巨多, 難 review | v0.2 拆 brief: (a) schema + emitter, (b) sample 5 份 fixture, (c) 全量 fixture 批量加 |
| **G-ACI-07** | 8 個項目中部分项目可能沒有 mock 工具 (Ada 等), 「所有项目」字面理解可能過寬 | 落地時找不到東西改造 | v0.2 前先全域 `find . -name '*mock*' -type d` 確認有 mock 的項目清單 |

---

## 7. 守門自檢 (per AGENTS.md §4)

| 守門 | 對齊狀態 | 說明 |
|---|---|---|
| #1 docs 必含 | ✅ | 本檔 + `.aci.json` per 项目 |
| #3 5 域 Lead | 🟡 | 跨項目, 待 v0.2 鎖版後按 per-項目 brief 拆 |
| #5 env hard ban | ✅ | ACI 不涉及 env, 0 影響 |
| #6 中文默認 | ✅ | doc 簡中, comment 中英混排 |
| #7 0 unsafe | ✅ | mock 工具多為 sh + python + Rust, 0 unsafe |
| #9 subprocess | ✅ | LLM agent 讀取走 subprocess (CLI) |
| #10 author=Ulysses | ✅ | 跨項目 commit 統一 author |
| #11 缺標比錯標 | ✅ | §6 列 7 缺口 |
| #12 docs 同步 | ✅ | 本檔 + AGENTS §6.1 view 索引同步 |
| #13 W/T/M | ✅ | `.aci.json` 是 Master (SCD-2), `assertions/*.aci.json` 是 Transaction (audit) |
| #14 v4 Mavis 审核 | ✅ | author=Ulysses + Mavis 审核 |
| #15 scope creep | ✅ | 1 brief 1 sub-agent 1 mock 项目, 不批量 (per §4.1 / §4.2 / §4.3) |
| #19 v19 Python 化 | ✅ | `_lib_aci_emit.py` + `aci-summary` CLI 都走 Python |
| #20 子代理 brief | ✅ | per 项目 brief (per ULYS-189 §6.1 經驗) |
| #24 vendor 中立 | ✅ | 0 外部 vendor, 只用 stdlib JSON + serde |

**v0.1 拍板狀態**: 🟡 6 決策待拍板, 7 缺口顯式列。

---

## 8. 下一步 (Next Steps)

本 v0.1 落地後, 等待 Ulysses 拍板 6 決策 + 是否啟動 §4.1.2 第 1 筆 brief (Star mock 第 1 階段)。

**等待中 (per 守門 #15, 不主動 scope creep)**:

1. 🟡 **6 決策拍板** (per §5) → v0.2 鎖版, 修訂本檔
2. 🟡 **G-ACI-01/02/07 盤點確認** (per §6) → v0.2 前先跑全域 mock 项目盤點
3. 🟡 **Ulysses 拍板啟動第 1 筆 brief** (per §4.1.2) → Star `tools/star-flash-mock/` 第 1 階段落地

**v0.2 預期內容** (decision 鎖版後):
- §5 6 決策 ✅
- §6 缺口 G-ACI-01/02/07 狀態更新
- §4 落地路徑微調 (per 決策)
- 新增 §X Cerebellum Credential 同型 §6.1 (per ULYS-189 §6.1 經驗): ACI Schema 詳細設計 (字段 final 名 + LLM prompt template 樣本)
- §10 修訂歷史加 v0.2 row

---

## 9. 修訂歷史 (Revision History)

| 版本 | 日期 | 修訂人 | 內容 |
|---|---|---|---|
| **v0.1** | 2026-09-23 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代審 | **初稿 (Draft)** — 觸發 ULYS-191 (2026-09-22 23:39 JST) + reply `01a0cbc1` (2026-09-23 09:53 JST)「我的目的是改造我所有的 mock 项目」; 9 節結構 (目的 / 概念映射 / 跨項目盤點 / Schema 鎖版 / 5 階段路徑 / 6 決策 / 7 缺口 / 守門自檢 / 下一步); ~10 KB, 預估 ~0.05M token |

---

## 附錄 A: 外部參考 (External References)

- ULYS-189 Jev 設計分析 (姊妹文檔, 小腦判斷層, per 1.3 cross-ref)
  - `docs/architecture/2026-09-22-jev-integration/00-design-analysis.md` §6.1 Cerebellum Credential 設計 — 同型 §X ACI Schema 詳細設計 落檔範本
- ULYS-190 "Mock开关" — sibling issue, 同期觸發, 觸發測試目標功能開關
  - 觸發測試目標功能開關 = ULYS-190 落地後, ACI emit 才能精準標 `scope.domain = "admin"` 等 (否則 emit 不知道哪個 domain 失敗)
- ULYS-135/136/137/138/139/140/141 七个 Mock 回归测试 issue (per 项目, ULYS-191 落地的工時依據)
- 既有 fixture 範例: `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-1.json` (per §3.4 並存擴展起點)

