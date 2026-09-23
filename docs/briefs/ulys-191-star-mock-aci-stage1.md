# ULYS-191 §4.1.2 第 1 笔 brief — Star mock ACI 第 1 阶段實装

> **狀態**: 🟡 Draft v0.1 (ULYS-191 v0.2 §4.1.2 啟動, 待 Ulysses 拍板落地 or 進一步調整)
> **日期**: 2026-09-23
> **制定者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代審
> **觸發**: ULYS-191 v0.2 approved (reply `01a0cbf3` 2026-09-23 10:48 JST 「可以启动」) + design-analysis §4.1.2
> **設計稿引用**: [`315c74cd` v0.1 + 本 commit v0.2](../architecture/2026-09-22-aci-mock-interface/00-design-analysis.md)
> **目標 commit**: `agent/minimaxm3/ulys-191` branch (Star 主倉, `D:/Star`)

## 0. 目的 (Purpose)

落實 ULYS-191 §4.1 第 1 階段: 給 Star `tools/star-flash-mock/` 加 ACI 介面,
產出 (a) `.aci.json` schema + (b) `_lib_aci_emit.sh/.py` emitter helper + (c) sample 5 份 fixture 加 `aci_assertion` + (d) `aci-summary` CLI 雛形。
本 brief 是「7 個項目跨項目改造」的**第 1 個 sub-agent brief**,
對齊 ULYS-189 §6.1 brief 拆解範式 (per AGENTS.md 守門 #20)。

---

## 1. 範圍 (Scope)

### 1.1 In-Scope (本 brief 必做)

1. **`.aci.json`** — 新文件, Star mock 项目根 (`tools/star-flash-mock/.aci.json`), 完整聲明 §3 spec
2. **`_lib_aci_emit.sh`** — bash emitter helper (對應現有 `_lib_validate.py` 風格), 提供 `aci_emit` 函數
3. **`_lib_aci_emit.py`** — Python emitter helper (對應 `validate.py` 風格), 提供 `AciEmitter` class
4. **5 份 sample fixture** — 從 `mock_data/agent-runtime/guards/` 選 5 份 (例如 G-1 / G-3 / G-5 / G-7 / G-9), 加 `aci_assertion` 字段, 並存擴展 (per v0.2 決策 #4)
5. **`aci-summary` CLI 雛形** — 新工具 `tools/aci-summary/aci_summary.py`, 給 LLM agent 一行指令讀懂 mock 結果
6. **文檔同步**:
   - `tools/star-flash-mock/README.md` 加 §ACI 段落 (說明 emitter 用法)
   - `tools/star-flash-mock/docs/regression-report-it-2026-09-23.md` 新增 (驗證 5 sample + 既有 87 case 不 regress)
7. **單 commit 收官** (per 守門 #10 author=Ulysses)

### 1.2 Out-of-Scope (本 brief 不做, 跨 session 續做)

- ❌ Star 全量 175 fixture 加 `aci_assertion` (per G-ACI-06 三段拆解 b/c, 第 2 笔 brief)
- ❌ IDE1.0 改造 (per §4.2)
- ❌ 4 個項目盤點 (per §4.3 + G-ACI-01)
- ❌ 跨項目統一驗證 (per §4.5)
- ❌ `aci-summary` CLI 進階功能 (per §4.4 完整版, 本 brief 只出雛形)
- ❌ 守門 #15 限制: 1 sub-agent 1 切點, 不批量 (per AGENTS.md §4 #15)

---

## 2. 落地清單 (Deliverables)

| 序 | 路徑 | 類型 | 說明 |
|---|---|---|---|
| 1 | `tools/star-flash-mock/.aci.json` | 新文件 | 完整 ACI 契約 (per v0.2 §3.1) |
| 2 | `tools/star-flash-mock/scripts/_lib_aci_emit.sh` | 新文件 | bash emitter helper (~80 LOC) |
| 3 | `tools/star-flash-mock/scripts/_lib_aci_emit.py` | 新文件 | Python emitter helper (~120 LOC, 提供 `AciEmitter` class) |
| 4 | `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-1.json` | 修改 | 加 `aci_assertion` 字段 (per §3.4 並存擴展) |
| 5 | `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-3.json` | 修改 | 同上 |
| 6 | `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-5.json` | 修改 | 同上 |
| 7 | `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-7.json` | 修改 | 同上 |
| 8 | `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-9.json` | 修改 | 同上 |
| 9 | `tools/aci-summary/aci_summary.py` | 新文件 | CLI 雛形, 聚合 assertions/ 目錄 (~150 LOC) |
| 10 | `tools/star-flash-mock/README.md` | 修改 | 加 §ACI 段落 (~30 行) |
| 11 | `tools/star-flash-mock/docs/regression-report-stage1-2026-09-23.md` | 新文件 | 驗證 5 sample + 87 case 不 regress |

**commit 數**: 1 commit (per 守門 #1 v19 5 守門全套跑 + 守門 #10 author=Ulysses)

---

## 3. 守門對齊 (per AGENTS.md §4)

| 守門 | 對齊 | 落地實證 |
|---|---|---|
| #1 docs 必含 | ✅ | 本 brief + 落地後 AGENTS.md §6.1 view 索引同步 + §8 修訂歷史 v0.83 |
| #3 5 域 Lead | ✅ (本 brief 範圍內) | Star mock 屬於 Star 倉內, 1 sub-agent 落地, Mavis 审核 author=Ulysses |
| #5 env hard ban | ✅ | 0 env 涉及, 純 schema + helper + fixture |
| #6 中文默認 | ✅ | 文檔簡中, fixture 字段含中英混排 |
| #7 0 unsafe | ✅ | emitter 全 stdlib (json + argparse), 0 unsafe |
| #9 subprocess | ✅ | `aci-summary` CLI 走 subprocess (per §4.4) |
| #10 author=Ulysses | ✅ | commit `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit` |
| #11 缺標比錯標 | ✅ | 本 brief §6 列已知缺口 |
| #12 docs 同步 | ✅ | commit 引用本 brief + design-analysis §4.1 |
| #13 W/T/M | ✅ | `.aci.json` = Master (SCD-2) / `aci_assertion` = Transaction (audit) |
| #14 v4 Mavis 审核 | ✅ | author=Ulysses + Mavis 审核 per 8/27 19:39 JST 授權 |
| #15 scope creep | ✅ | 本 brief = Star mock 1 階段, 不跨項目不批量 |
| #19 v19 Python 化 | ✅ | `_lib_aci_emit.py` + `aci_summary.py` 走 Python |
| #20 子代理 brief | ✅ | 本 brief = 子代理 brief, 落 `docs/briefs/ulys-191-*.md` |
| #24 vendor 中立 | ✅ | 0 外部 vendor, stdlib only |

---

## 4. 驗收標準 (Acceptance Criteria)

落地完成後, 跑以下驗證 (per 守門 #1 v19 5 守門全套跑):

### 4.1 schema 驗證

```bash
# .aci.json 自身是合法 JSON + 必填字段齊
python3 -c "import json; d=json.load(open('tools/star-flash-mock/.aci.json')); assert d['aci_version']=='0.1.0-draft'; print('OK')"

# 5 sample fixture 必填 aci_assertion 字段齊
for n in 1 3 5 7 9; do
    python3 -c "
import json
d = json.load(open('tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-${n}.json'))
assert 'aci_assertion' in d, 'missing aci_assertion'
a = d['aci_assertion']
for f in ['assertion_id', 'aci_version', 'layer', 'scope', 'expect', 'actual', 'status', 'severity', 'reasoning', 'captured_at']:
    assert f in a, f'missing {f}'
print(f'G-${n} OK')
"
done
```

### 4.2 emitter 驗證

```bash
# Python emitter 跑一次 sample
python3 tools/star-flash-mock/scripts/_lib_aci_emit.py --help
python3 tools/star-flash-mock/scripts/_lib_aci_emit.py \
    --layer it --scope project=star-flash-mock,module=guards,domain=admin \
    --expect-type response_within_ms --expect-value 2000 \
    --actual-type response_within_ms --actual-value 5000 \
    --status FAIL --severity high \
    --reasoning "API timeout, KMS unlock slow" \
    --suggested-fix "check KMS GC log" \
    --tags perf,kms,timeout \
    --output /tmp/test_assertion.aci.json

cat /tmp/test_assertion.aci.json  # 必含 13 字段
```

### 4.3 aci-summary CLI 驗證

```bash
# 對 5 sample fixture 跑 aggregation
python3 tools/aci-summary/aci_summary.py tools/star-flash-mock/mock_data/agent-runtime/guards/
# 期望輸出: critical/high/medium 統計 + Top 3 issues
```

### 4.4 既有 regression 不 regress

```bash
cd D:/Star/tools/star-flash-mock
bash scripts/regression-test-system.sh  # 13/13 PASS (per ULYS-140 v1.2)
bash scripts/run-all.sh                 # §UT 11 + §IT 1 + §ST 1 = 13/13 PASS
```

### 4.5 docs 同步

```bash
git -C D:/Star diff agent/minimaxm3/ulys-191~1..agent/minimaxm3/ulys-191 -- AGENTS.md | head -50
# 期望 §6.1 加 ACI 集成 view 索引 + §8 v0.83 修訂歷史 row
```

---

## 5. 風險 + 已知缺口 (per 守門 #11)

| # | 風險 | 緩解 |
|---|---|---|
| **R-BRIEF-01** | G-ACI-04 LLM prompt template hint 是「猜」, 5 sample fixture 的 `reasoning / suggested_fix` 文案可能 LLM 讀起來怪 | 落地後跑 10 條真實 LLM agent dry-run, 不達標跨 session 修字段文案 |
| **R-BRIEF-02** | `.aci.json` schema 鎖版後, 第 2 笔 brief 全量 fixture 加字段時若需改 schema, 跨 session breaking change | v0.2 字段 final 名已鎖, 跨 session 加字段需起 v0.3 (per 決策 #3) |
| **R-BRIEF-03** | 5 sample 選 G-1/3/5/7/9 都是 L0 護欄類, 缺 IT/ST 層樣本 | 第 2 笔 brief 全量覆蓋時補 IT/ST 樣本 |
| **R-BRIEF-04** | 並存擴展期間 `fixture_assertion` + `aci_assertion` 同時寫入, 文件 ~2x 大 | 第 2 笔 brief 全量後觀察, 1 sprint 後決定刪舊 |

---

## 6. 下一步 (Next Steps, 跨 session 續做)

1. ✅ ULYS-191 v0.2 approved (本 commit, reply `01a0cbf3` 拍板)
2. 🟡 **本 brief v0.1 落檔** (本 commit)
3. 🟡 **sub-agent 派工** (per 守門 #20 dispatcher.py brief(...) + #15 1 sub-agent 1 切點)
4. 🟡 **sub-agent 落地實裝** + 1 commit + 5 守門全套跑驗證
5. 🟡 第 2 笔 brief (Star 全量 175 fixture, per G-ACI-06 三段拆解 b/c)
6. 🟡 IDE1.0 brief (per §4.2)
7. 🟡 4 項目盤點 brief (per G-ACI-01)

**token 累計**: ~0.03M (本 brief v0.1 起草)


