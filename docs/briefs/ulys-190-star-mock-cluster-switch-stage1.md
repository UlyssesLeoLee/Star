# ULYS-190 §4.1 第 1 笔 brief — Star mock Cluster Switch 第 1 阶段實装

> **狀態**: 🟡 Draft v0.1 (ULYS-190 v0.2 approved 後, 待 Ulysses 派工觸發)
> **日期**: 2026-09-23
> **制定者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代審
> **觸發**: ULYS-190 v0.2 approved (reply `01a0d073` 2026-09-23 22:46 JST 「a」) + design-analysis §4.1
> **設計稿引用**: [`c3d7d171` v0.1 + 本 commit v0.2](../architecture/2026-09-22-mock-switches/00-design-analysis.md)
> **目標 commit**: `agent/minimaxm3/ulys-190` branch (Star 主倉, `D:/Star`)

## 0. 目的 (Purpose)

落實 ULYS-190 §4.1 第 1 階段: 給 Star `tools/star-flash-mock/` 加 L1 cluster_switch 雛形,
產出 (a) `.mock-cluster.json` 頂層配置 + (b) `_lib_mock_switch.py` 讀取 helper + (c) dispatcher 接入 `.aci.json` `aci_compat_version` 對齊 + (d) 1 份 sample fixture 帶 `mock_switch_trace` 驗證 (per §3.4)。
本 brief 是「7 個項目跨項目改造」的**第 1 個 sub-agent brief**,
對齊 ULYS-191 §4.1.2 + ULYS-189 §6.1 brief 拆解範式 (per AGENTS.md 守門 #20)。

---

## 1. 範圍 (Scope)

### 1.1 In-Scope (本 brief 必做)

1. **`.mock-cluster.json`** — 新文件, Star mock 项目根 (`tools/star-flash-mock/.mock-cluster.json`), 完整聲明 §3.1 L1 cluster_switch schema
2. **`_lib_mock_switch.py`** — Python 讀取 helper (對應 `_lib_aci_emit.py` 風格), 提供 `MockSwitchReader` class + `is_cluster_enabled()` / `get_mode()` helper
3. **`.aci.json` 加 `aci_compat_version` 對齊字段** — 修改 `tools/star-flash-mock/.aci.json` v0.1 (per ULYS-191 §4.1.2 Stage 1), 加 `aci_compat_version: "0.1.0-draft"` 字段, 跟 `.mock-cluster.json.aci_compat_version` 對齊
4. **1 份 sample fixture 帶 `mock_switch_trace` 字段** — 修改 `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-1.json` (per ULYS-191 §4.1.2 Stage 1), 在 `aci_assertion` 內加 `mock_switch_trace` 字段, 驗證 emitter 讀取 `.mock-cluster.json` 並拼接字符串 (per ULYS-190 §3.4 example)
5. **dispatcher 接入 mock_switch 讀取** — 新增 `_lib_mock_switch.py` 在 `_lib_aci_emit.py` 中 import (or 從 `.mock-cluster.json` 讀取 `aci_compat_version` 後 assertion 拼接 `mock_switch_trace`)
6. **文檔同步**:
   - `tools/star-flash-mock/README.md` 加 §Mock Cluster Switch 段落 (說明 `.mock-cluster.json` 用法 + 跟 `.aci.json` 對齊)
   - `tools/star-flash-mock/docs/regression-report-stage1-cluster-switch-2026-09-23.md` 新增 (驗證 `.mock-cluster.json` 合法 + 1 sample fixture `mock_switch_trace` 拼接正確)
7. **單 commit 收官** (per 守門 #10 author=Ulysses)

### 1.2 Out-of-Scope (本 brief 不做, 跨 session 續做)

- ❌ Star mock `plugins.<id>.modules.<id>` L3 module_switch 落地 (per ULYS-190 §4.4)
- ❌ Star mock 全量 5 sample fixture 加 `mock_switch_trace` 字段 (per ULYS-190 §4.4 + G-MS-04 跨項目 plugin_id 命名; 本 brief 只驗證 1 sample)
- ❌ IM1.0 plugin_switch 落地 (per ULYS-190 §4.2)
- ❌ RGS / CATs / IDE1.0 / GitGit plugin_switch 落地 (per ULYS-190 §4.3)
- ❌ 7 項目 module_switch 擴展 (per ULYS-190 §4.4)
- ❌ 跨項目 CI 加 `mock-switch-validate` (per ULYS-190 §4.5)
- ❌ 開關變更審計日誌 (per ULYS-190 §6 G-MS-05, Transaction audit SCD-2, 跨 session)
- ❌ mock_switch_trace 字段長度截斷到 ~80 字 (per ULYS-190 §6 G-MS-08, 跨 session)
- ❌ 守門 #15 限制: 1 sub-agent 1 切點, 不批量 (per AGENTS.md §4 #15)

---

## 2. 落地清單 (Deliverables)

| 序 | 路徑 | 類型 | 說明 |
|---|---|---|---|
| 1 | `tools/star-flash-mock/.mock-cluster.json` | 新文件 | L1 cluster_switch 完整契約 (per design-analysis §3.1) |
| 2 | `tools/star-flash-mock/scripts/_lib_mock_switch.py` | 新文件 | Python 讀取 helper (~80 LOC, `MockSwitchReader` class) |
| 3 | `tools/star-flash-mock/.aci.json` | 修改 | 加 `aci_compat_version: "0.1.0-draft"` 字段 |
| 4 | `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-1.json` | 修改 | 在 `aci_assertion` 內加 `mock_switch_trace` 字段 |
| 5 | `tools/star-flash-mock/scripts/_lib_aci_emit.py` | 修改 | import `MockSwitchReader` + 拼接 `mock_switch_trace` (per ULYS-191 §4.1.2 stage1 + ULYS-190 §3.4) |
| 6 | `tools/star-flash-mock/README.md` | 修改 | 加 §Mock Cluster Switch 段落 (~30 行) |
| 7 | `tools/star-flash-mock/docs/regression-report-stage1-cluster-switch-2026-09-23.md` | 新文件 | 驗證 `.mock-cluster.json` 合法 + 1 sample `mock_switch_trace` 拼接正確 |

**commit 數**: 1 commit (per 守門 #1 v19 5 守門全套跑 + 守門 #10 author=Ulysses)

---

## 3. 守門對齊 (per AGENTS.md §4)

| 守門 | 對齊 | 落地實證 |
|---|---|---|
| #1 docs 必含 | ✅ | 本 brief + 落地後 AGENTS.md §6.1 view 索引同步 + §8 修訂歷史 v0.84 |
| #3 5 域 Lead | ✅ (本 brief 範圍內) | Star mock 屬於 Star 倉內, 1 sub-agent 落地, Mavis 审核 author=Ulysses |
| #5 env hard ban | ✅ | 0 env 涉及, 純 schema + helper + fixture |
| #6 中文默認 | ✅ | 文檔簡中, fixture 字段含中英混排 |
| #7 0 unsafe | ✅ | reader + emitter 全 stdlib (json + argparse), 0 unsafe |
| #9 subprocess | ✅ | `mock_switch_trace` 拼接走 subprocess (per ULYS-189 v0.2 §6.1) |
| #10 author=Ulysses | ✅ | commit `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit` |
| #11 缺標比錯標 | ✅ | 本 brief §6 列已知缺口 (G-MS-BRIEF-01..04) |
| #12 docs 同步 | ✅ | commit 引用本 brief + design-analysis §4.1 |
| #13 W/T/M | ✅ | `.mock-cluster.json` = Master (SCD-2) / `mock_switch_trace` = Transaction (audit) |
| #14 v4 Mavis 审核 | ✅ | author=Ulysses + Mavis 审核 per 8/27 19:39 JST 授權 |
| #15 scope creep | ✅ | 本 brief = Star mock L1 cluster_switch 1 階段, 不跨項目不批量 |
| #19 v19 Python 化 | ✅ | `_lib_mock_switch.py` 走 Python (per 守門 #19 v19) |
| #20 子代理 brief | ✅ | 本 brief = 子代理 brief, 落 `docs/briefs/ulys-190-*.md` |
| #24 vendor 中立 | ✅ | 0 外部 vendor, stdlib only |

---

## 4. 驗收標準 (Acceptance Criteria)

落地完成後, 跑以下驗證 (per 守門 #1 v19 5 守門全套跑):

### 4.1 `.mock-cluster.json` schema 驗證

```bash
# .mock-cluster.json 自身是合法 JSON + 必填字段齊
python3 -c "
import json
d = json.load(open('tools/star-flash-mock/.mock-cluster.json'))
for f in ['cluster_version', 'project', 'cluster_id', 'enabled', 'mode', 'aci_compat_version']:
    assert f in d, f'missing {f}'
assert d['cluster_version'] == '0.1.0-draft', 'cluster_version mismatch'
assert d['project'] == 'star-flash-mock', 'project mismatch'
assert isinstance(d['enabled'], bool), 'enabled must be bool'
assert d['mode'] in ('offline', 'passthrough', 'proxy'), 'mode invalid'
print('OK')
"
```

### 4.2 `_lib_mock_switch.py` 驗證

```bash
python3 tools/star-flash-mock/scripts/_lib_mock_switch.py --help
# 期望輸出: usage + --cluster-config <path> + --aci-config <path> 子命令
```

### 4.3 sample fixture `mock_switch_trace` 拼接驗證

```bash
python3 -c "
import json
d = json.load(open('tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-1.json'))
assert 'aci_assertion' in d, 'missing aci_assertion'
a = d['aci_assertion']
assert 'mock_switch_trace' in a, 'missing mock_switch_trace'
trace = a['mock_switch_trace']
# 期望 trace 含 3 層 (cluster / plugin / module), 雖然 plugins section 尚未落地, trace 必含 cluster.enabled + cluster.mode
assert 'cluster.enabled' in trace, 'trace missing cluster.enabled'
assert 'cluster.mode' in trace, 'trace missing cluster.mode'
print(f'G-1 trace OK: {trace}')
"
```

### 4.4 既有 regression 不 regress

```bash
cd D:/Star/tools/star-flash-mock
bash scripts/regression-test-system.sh  # 13/13 PASS (per ULYS-140 v1.2)
bash scripts/run-all.sh                 # §UT 11 + §IT 1 + §ST 1 = 13/13 PASS
```

### 4.5 docs 同步

```bash
git -C D:/Star diff agent/minimaxm3/ulys-190~1..agent/minimaxm3/ulys-190 -- AGENTS.md | head -50
# 期望 §6.1 加 Mock Cluster Switch 集成 view 索引 + §8 v0.84 修訂歷史 row
```

---

## 5. 風險 + 已知缺口 (per 守門 #11)

| # | 風險 / 缺口 | 緩解 |
|---|---|---|
| **G-MS-BRIEF-01** | `.aci.json` 已 ship (per ULYS-191 §4.1.2 stage1 commit `cebba99c`), 加 `aci_compat_version` 字段需修改已 ship 文件 | 並存擴展 (per design-analysis §3.3), 0 改既有 schema 字段, 純加新字段 |
| **G-MS-BRIEF-02** | `_lib_aci_emit.py` 已 ship (per ULYS-191 §4.1.2 stage1), 加 `MockSwitchReader` import 需修改已 ship 文件 | 並存擴展, 0 改既有 function body, 純 import + helper 函數 |
| **G-MS-BRIEF-03** | `mock_switch_trace` 字段拼接規則: plugins section 尚未落地, 但 G-1 sample 屬於「guards」 plugin 的「task_queue_check」 operation, 應拼出 `plugin.guards.enabled=true, module.task_queue_check.enabled=true` 等價物 (per design-analysis §3.4 example 風格) | 本 brief 階段允許 trace 簡化為只含 cluster.enabled + cluster.mode, plugins 部分留 `[TBD plugins section 落地後擴充]` placeholder; 跨 session §4.4 module_switch 擴展時補完整 |
| **G-MS-BRIEF-04** | `.aci.json` 跟 `.mock-cluster.json` 兩個文件 schema 對齊驗證腳本未提供 | `_lib_mock_switch.py` 提供 `validate_aci_compat_version()` 函數, 兩個文件 `aci_compat_version` 字段必須一致, 不一致 raise `MockSwitchError::AciCompatVersionMismatch` |
| **R-BRIEF-01** | ULYS-191 §4.1.2 Stage 1 brief 的「G-ACI-04 LLM prompt template hint 是猜」風險, 本 brief `mock_switch_trace` 字段文案也是「猜」 | 落地後跑 10 條真實 LLM agent dry-run, 不達標跨 session 修字段文案 |
| **R-BRIEF-02** | `.mock-cluster.json` schema 鎖版後, §4.4 module_switch 擴展時若需改 schema, 跨 session breaking change | v0.1-draft 字段 final 名已鎖 (per design-analysis 決策 #3 rolling), 跨 session 加字段需起 v0.2 |

---

## 6. 下一步 (Next Steps, 跨 session 續做)

1. ✅ ULYS-190 v0.2 approved (per reply `01a0d073`「a」, 6 決策全部走推薦 A+A+B+A+B+A)
2. ✅ 本 brief v0.1 落檔 (本 commit)
3. 🟡 **派工觸發** (T1 D-Boy 明示「派吧」/ T2 5 域 Lead 真人到位 T3 / T3 D-Boy 修 brief 範圍, 任一即派, per AGENTS.md §4 #3 + #14 v3)
4. 🟡 **sub-agent 派工** (per 守門 #20 dispatcher.py brief(...) + #15 1 sub-agent 1 切點)
5. 🟡 **sub-agent 落地實裝** + 1 commit + 5 守門全套跑驗證
6. 🟡 §4.2 IM1.0 plugin_switch 雛形 brief 啟動派工 (跟 §4.1 並行, 阻力最小)
7. 🟡 §4.3 4 項目 (RGS / CATs / IDE1.0 / GitGit) plugin_switch brief 啟動派工 (順序: IM1.0 → CATs+IDE1.0 共享 Rust helper → RGS → GitGit)
8. 🟡 §4.4 7 項目 module_switch 擴展 (最大塊, 跨 session 續)
9. 🟡 §4.5 跨項目 CI 加 `mock-switch-validate` (收官, 1 commit)
10. 🟡 G-MS-01 Ada mock 項目盤點確認 (per design-analysis §4.6 + §6 G-MS-01)

**token 累計**: ~0.01M (本 brief v0.1 起草, 對齊 ULYS-191 §4.1.2 brief ~0.01M)
