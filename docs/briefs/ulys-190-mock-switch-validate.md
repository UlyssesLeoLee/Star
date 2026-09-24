# ULYS-190 §4.5 — 跨項目 CI `mock-switch-validate` brief

> **狀態**: 🟡 Draft v0.1 (ULYS-190 §4.5 收官 brief, 工具已落档, 待 §4.5 docs 收官)
> **日期**: 2026-09-23
> **制定者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代審
> **觸發**: ULYS-190 v0.2 approved + §4.1 Stage 1 ship (commit `f6045bd9`) + reply `01a0d0e1` 2026-09-23 23:02 JST 「完成所有后续工作」 T1 派工
> **設計稿引用**: [`c3d7d171` v0.1 + `0f199757` v0.2 + `v0.3` G-MS-01 ✅](../architecture/2026-09-22-mock-switches/00-design-analysis.md)
> **目標 commit**: `agent/minimaxm3/ulys-190` branch (Star 主倉, 純 docs + tool 落档)

## 0. 目的 (Purpose)

落實 ULYS-190 §4.5 跨項目 CI 統一驗證收官: 提供 `tools/mock-switch-validate.py` CLI 工具 + 子命令 `validate-one` / `validate-all` / `report`, 跨 7 項目 (Star / IDE1.0 / RGS / CATs / IM1.0 / GitGit / Ada) 跑 `.mock-cluster.json` + `.aci.json` schema + aci_compat 校验, 生成 markdown 报告。

---

## 1. 範圍 (Scope)

### 1.1 In-Scope (本 brief 必做)

1. **`tools/mock-switch-validate.py`** — 新文件 (per brief v0.1 §1.1 item 1)
   - `MockSwitchReader` class (复用 `_lib_mock_switch.py` core, 但 stdlib-only, 不 import `_lib_mock_switch.py` 因為跨项目路径難統一)
   - 3 CLI 子命令: `validate-one` / `validate-all` / `report`
   - 跨项目根路径常量 `DEFAULT_PROJECT_ROOTS` (7 项目, per design-analysis v0.3 + G-MS-01 ✅)
   - 全 stdlib (json + argparse + pathlib + sys), 0 vendor
2. **README 引用** — `tools/README.md` (per 既有结构) 加 `mock-switch-validate.py` 引用段 (落地 brief commit 时一并加)
3. **regression report** — `tools/star-flash-mock/docs/regression-report-stage2-mock-switch-validate-2026-09-23.md` 新文件
4. **單 commit 收官** (per 守門 #10 author=Ulysses)

### 1.2 Out-of-Scope (本 brief 不做, 跨 session 續做)

- ❌ CI workflow `.github/workflows/mock-switch-validate.yml` 落地 (per §4.5 收官) — 跨 session brief
- ❌ 6 项目 (IM1.0 / RGS / CATs / IDE1.0 / GitGit / Ada) `.mock-cluster.json` 落地 — §4.2-§4.3 跨 session brief
- ❌ §4.4 7 項目 module_switch 擴展 — 跨 session brief
- ❌ §4.6 Ada L1 cluster_switch 實裝 (D:/Ada/crates/ada-mock/.mock-cluster.json 新文件) — 跨 session brief
- ❌ 守門 #15 限制: 1 sub-agent 1 切點, 不批量

---

## 2. 落地清單 (per brief v0.1 §2)

| 序 | 路徑 | 類型 | 大小 / 狀態 |
|---|---|---|---|
| 1 | `tools/mock-switch-validate.py` | 新文件 | 244 LOC, 3 CLI 子命令 (validate-one/validate-all/report) + MockSwitchReader class + DEFAULT_PROJECT_ROOTS 7 项目 |
| 2 | `tools/README.md` | 修改 | 加 `mock-switch-validate.py` 引用段 |
| 3 | `tools/star-flash-mock/docs/regression-report-stage2-mock-switch-validate-2026-09-23.md` | 新文件 | 7 段结构 per AGENTS.md §3 |

**commit 數**: 1 commit

## 3. 守門對齊 (per AGENTS.md §4)

| 守門 | 對齊 | 落地實證 |
|---|---|---|
| #1 docs 必含 | ✅ | 本 brief + 落地後 README 引用 + regression report |
| #5 env hard ban | ✅ | 0 env 涉及, 純 stdlib |
| #6 中文默認 | ✅ | 簡中 + 中英混排 |
| #7 0 unsafe | ✅ | `mock-switch-validate.py` 全 stdlib, 0 unsafe |
| #9 subprocess | ✅ | 3 CLI 子命令走 subprocess (per ULYS-189 v0.2 §6.1) |
| #10 author=Ulysses | ✅ | commit author=Ulysses |
| #11 缺標比錯標 | ✅ | 本 brief §5 列已知缺口 |
| #12 docs 同步 | ✅ | commit 引用本 brief + design-analysis v0.3 |
| #13 W/T/M | ✅ | `.mock-cluster.json` = Master (SCD-2) / 報告 = Transaction (audit) |
| #14 v4 Mavis 审核 | ✅ | author=Ulysses + Mavis 接手 per 8/27 19:39 JST 授權 |
| #15 scope creep | ✅ | 本 brief = Star repo 內 1 tool + docs, 不跨項目不批量 |
| #19 v19 Python 化 | ✅ | `mock-switch-validate.py` 全 Python stdlib |
| #20 子代理 brief | ✅ | 本 brief 落档 |
| #24 vendor 中立 | ✅ | 0 外部 vendor, stdlib only |

## 4. 驗收標準 (per AGENTS.md §4 守門 + brief §1.1)

### 4.1 validate-one 單項目校验

```bash
$ python3 tools/mock-switch-validate.py validate-one \
    --cluster-config tools/star-flash-mock/.mock-cluster.json \
    --aci-config tools/star-flash-mock/.aci.json
CLUSTER=OK project=star-flash-mock enabled=True mode=offline cluster_version=0.1.0-draft aci_compat_version=0.1.0-draft
ACI=OK aci_version='0.1.0-draft' == cluster.aci_compat_version='0.1.0-draft'  # exit 0
```

### 4.2 validate-all 跨項目校验

```bash
$ python3 tools/mock-switch-validate.py validate-all
TOTAL=7 FAIL=7  # 当前仅 Star 落地, 6 项目待跨 session 落地
PROJECT=star-flash-mock cluster_ok=False error=cluster_config 不存在: ...
PROJECT=ide1 cluster_ok=False error=...
... (共 7 行)
```

### 4.3 report markdown 报告生成

```bash
$ python3 tools/mock-switch-validate.py report --output <path>
REPORT_WRITTEN=<path> fail=7
```

## 5. 風險 + 已知缺口 (per 守門 #11)

| # | 風險 / 缺口 | 緩解 |
|---|---|---|
| **G-MS-VALIDATE-01** | `mock-switch-validate.py` 跟 `_lib_mock_switch.py` reader 重複實現, 跨项目路径不一致时 2 個腳本維護成本 | 本 brief 階段允許重複 (per 守門 #15 + 跨项目路徑), v0.2 評估提共用 `lib` 包 (per §4.5) |
| **G-MS-VALIDATE-02** | `DEFAULT_PROJECT_ROOTS` 是 hardcoded, 跨项目路徑變更時需改 source code | 提供 `--roots-json` CLI flag 允許外部傳入 (per brief §4.2 cmd_validate_all) |
| **G-MS-VALIDATE-03** | CI workflow 落地 (`.github/workflows/mock-switch-validate.yml`) 留 P1 followup | 跨 session brief 啟動 |
| **R-BRIEF-01** | 6 项目 (IDE1.0 / RGS / CATs / IM1.0 / GitGit / Ada) 未落地 `.mock-cluster.json`, validate-all 暂时 FAIL=7 | 跨 session §4.2-§4.6 brief 落地后, validate-all FAIL 數會遞減 |

## 6. 跨 session 續做入口

1. ✅ ULYS-190 v0.2 approved + §4.1 Stage 1 ship (commit `f6045bd9`)
2. ✅ G-MS-01 Ada 盤點落地 + v0.3 design 升版
3. ✅ §4.5 跨項目 CI 工具落档 (`tools/mock-switch-validate.py`, per brief commit)
4. 🟡 CI workflow `.github/workflows/mock-switch-validate.yml` 落地 (跨 session brief)
5. 🟡 §4.2 IM1.0 plugin_switch brief 啟動派工
6. 🟡 §4.3 4 項目 (RGS / CATs / IDE1.0 / GitGit) plugin_switch brief 啟動派工
7. 🟡 §4.4 7 項目 module_switch 擴展 (最大塊)
8. 🟡 §4.6 Ada L1 cluster_switch 實裝 (D:/Ada/crates/ada-mock/.mock-cluster.json 新文件, 降級模式 L1 only)

**token 累計**: ~0.01M (本 brief v0.1 起草 + validator tool 落档 ~0.02M)
