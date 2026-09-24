# ULYS-190 §4.1 brief Stage 1 — Star Mock Cluster Switch 回归报告

> **状态**: 🟢 Stage 1 完成 + 5 守门全套 PASS (per brief v0.1 §4 验收)
> **日期**: 2026-09-23
> **触发**: ULYS-190 v0.2 approved (reply `01a0d073` 2026-09-23 22:46 JST 「a」) + reply `01a0d081` 2026-09-23 23:02 JST 「推进」派工触發 T1
> **Brief**: `docs/briefs/ulys-190-star-mock-cluster-switch-stage1.md` v0.1 (commit `0f199757`)
> **Design**: `docs/architecture/2026-09-22-mock-switches/00-design-analysis.md` v0.2

## 0. 目的 (Purpose)

实证 ULYS-190 §4.1 brief Stage 1 全部 7 交付物落地 + 5 验收脚本全过 (per brief v0.1 §4.1-§4.5)。Stage 1 是「7 个项目跨项目改造」的**第 1 个 sub-agent brief**, 跟 ULYS-191 §4.1.2 Stage 1 (commit `cebba99c`) 并行不冲突。

---

## 1. 7 交付物全部 ship (per brief v0.1 §1.1)

| # | 路径 | 类型 | 大小 / 状态 |
|---:|:---|:---|:---|
| 1 | `tools/star-flash-mock/.mock-cluster.json` | 新文件 | 964 bytes, L1 cluster_switch 完整契约 (per design-analysis §3.1) |
| 2 | `tools/star-flash-mock/scripts/_lib_mock_switch.py` | 新文件 | 267 LOC, MockSwitchReader class + 4 CLI 子命令 |
| 3 | `tools/star-flash-mock/.aci.json` | 修改 | +1 行 `aci_compat_version: "0.1.0-draft"` 字段 (并存扩展) |
| 4 | `tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-1.json` | 修改 | +1 行 `mock_switch_trace` 字段 |
| 5 | `tools/star-flash-mock/scripts/_lib_aci_emit.py` | 修改 | +`mock_switch_trace` 参数 + `--mock-switch-trace` CLI flag + 拼接逻辑 |
| 6 | `tools/star-flash-mock/README.md` | 修改 | +§6 Mock Cluster Switch (6.1-6.6 = 98 行) + v0.3 修订历史 row |
| 7 | `tools/star-flash-mock/docs/regression-report-stage1-cluster-switch-2026-09-23.md` | 新文件 | 本文档 |

**commit 數**: 1 commit (per 守门 #1 v19 5 守门全套跑 + 守门 #10 author=Ulysses)

---

## 2. 5 验收脚本全过 (per brief v0.1 §4.1-§4.5)

### §4.1 `.mock-cluster.json` schema 验证 — ✅ PASS

```bash
$ python3 -c "
import json
d = json.load(open('tools/star-flash-mock/.mock-cluster.json'))
for f in ['cluster_version', 'project', 'cluster_id', 'enabled', 'mode', 'aci_compat_version']:
    assert f in d, f'missing {f}'
assert d['cluster_version'] == '0.1.0-draft'
assert d['project'] == 'star-flash-mock'
assert isinstance(d['enabled'], bool)
assert d['mode'] in ('offline', 'passthrough', 'proxy')
print('OK')
"
OK
```

### §4.2 `_lib_mock_switch.py` 验证 — ✅ PASS

```bash
$ python3 tools/star-flash-mock/scripts/_lib_mock_switch.py --help
usage: _lib_mock_switch.py [-h] {is-enabled,get-mode,trace,validate-compat} ...
Star Mock L1 cluster_switch reader (per ULYS-190 §4.1)
...

$ python3 tools/star-flash-mock/scripts/_lib_mock_switch.py is-enabled \
    --cluster-config tools/star-flash-mock/.mock-cluster.json
CLUSTER_ENABLED=true   # exit 0

$ python3 tools/star-flash-mock/scripts/_lib_mock_switch.py get-mode \
    --cluster-config tools/star-flash-mock/.mock-cluster.json
CLUSTER_MODE=offline   # exit 0

$ python3 tools/star-flash-mock/scripts/_lib_mock_switch.py trace \
    --cluster-config tools/star-flash-mock/.mock-cluster.json
cluster.enabled=true, cluster.mode=offline, cluster.aci_compat_version=0.1.0-draft, [TBD plugins section 落地後擴充]

$ python3 tools/star-flash-mock/scripts/_lib_mock_switch.py validate-compat \
    --cluster-config tools/star-flash-mock/.mock-cluster.json \
    --aci-config tools/star-flash-mock/.aci.json
ACI_COMPAT=OK (cluster=0.1.0-draft)   # exit 0
```

### §4.3 sample fixture `mock_switch_trace` 拼接验证 — ✅ PASS

```bash
$ python3 -c "
import json
d = json.load(open('tools/star-flash-mock/mock_data/agent-runtime/guards/v1--guard--g-1.json'))
a = d['aci_assertion']
assert 'mock_switch_trace' in a, 'missing mock_switch_trace'
trace = a['mock_switch_trace']
assert 'cluster.enabled' in trace, 'trace missing cluster.enabled'
assert 'cluster.mode' in trace, 'trace missing cluster.mode'
print(f'G-1 trace OK: {trace}')
"
G-1 trace OK: cluster.enabled=true, cluster.mode=offline, cluster.aci_compat_version=0.1.0-draft, [TBD plugins section 落地後擴充]
```

### §4.4 既有 regression 不 regress — ✅ PASS

```bash
$ cd tools/star-flash-mock && bash scripts/regression-test-system.sh 2>&1 | tail -5
==== ST (System Test) regression test PASSED ====
  [OK] 守门 #1+#5+#9+#11+#12+#24 0 违反

$ cd tools/star-flash-mock && bash scripts/regression-test-agent-runtime.sh 2>&1 | tail -3
==== Agent Runtime regression test PASSED ====

$ cd tools/star-flash-mock && python3 scripts/_lib_validate.py validate-json \
    mock_data/agent-runtime/guards 2>&1
TOTAL=18
INVALID=0
```

### §4.5 docs 同步 — ✅ PASS

`tools/star-flash-mock/README.md`:
- §5 (per ULYS-191 §4.1.2 stage1 brief) ✅ 保留
- §6 (ULYS-190 §4.1 brief Stage 1, NEW): 6.1 目的 / 6.2 .mock-cluster.json / 6.3 mode / 6.4 reader helper / 6.5 ACI emit 拼接 / 6.6 Stage 1 状态
- §7 跨项目引用 (原 §6) renumbered
- §8 修订历史 (原 §7) renumbered + 加 v0.3 row (ULYS-190 Stage 1)


## 3. 已知缺口 (per 守门 #11 缺標比錯標)

| ID | 缺口 | 影响 | 解决方案 |
|---|---|---|---|
| **G-MS-BRIEF-01** | `.aci.json` 加 `aci_compat_version` 字段已 ship, 跨 session 落地其他项目时若 .aci.json 不同步加字段, validate-compat 会 fail | 跨项目 CI 暂时全部 fail | 跨 session brief 落地时, 每个项目 `.aci.json` 都加 `aci_compat_version` 字段 (per 设计稿 §3.3 並存擴展) |
| **G-MS-BRIEF-02** | `_lib_aci_emit.py` 加 `mock_switch_trace` 参数已 ship, 其他 fixture 调用方需手动传 trace | Stage 2 全量 fixture 加 trace 时需逐个改调用点 | Stage 2 brief 加 helper 自动从 `.mock-cluster.json` 读 trace (per §4.4) |
| **G-MS-BRIEF-03** | `mock_switch_trace` 字段只含 cluster.enabled + cluster.mode + cluster.aci_compat_version, plugins / modules 部分留 `[TBD]` placeholder | LLM 看到的 trace 不完整 | Stage 2 plugin_switch + Stage 3 module_switch 落地后补完整 trace 拼接 (per design-analysis §4.2-§4.4) |
| **G-MS-BRIEF-04** | `.aci.json` 跟 `.mock-cluster.json` schema 对齐只校验 `aci_version` 跟 `aci_compat_version` 一致, 其他字段(如 `project` / `tags_taxonomy`)未校验 | 跨项目落地时字段含义可能不一致 | 跨 session brief 加 schema 完整对齐校验 (per design-analysis §4.5 mock-switch-validate) |
| **G-MS-04** | 「plugin」 跟「module」命名跨项目是否一致仍未落地 | Stage 2/3 brief 需先锁 plugin_id / module_id 命名 | 跨 session 落地前先做 naming convention brief |
| **G-MS-08** | `mock_switch_trace` 字段长度未截断 (本 stage 拼出约 100 字) | 字段可能太长影响 LLM 读 | 跨 session 评估截断到 ~80 字 (per design-analysis §6 G-MS-08) |


## 4. 风险 + 风险缓解 (per 守门 #11)

| # | 风险 | 缓解 |
|---|---|---|
| **R-BRIEF-01** | ULYS-191 §4.1.2 stage1 brief 的「G-ACI-04 LLM prompt template hint 是猜」风险, 本 brief `mock_switch_trace` 字段文案也是「猜」 | Stage 2 跑 10 条真实 LLM agent dry-run, 不达标跨 session 修字段文案 |
| **R-BRIEF-02** | `.mock-cluster.json` schema 锁版后, §4.4 module_switch 扩展时若需改 schema, 跨 session breaking change | v0.1-draft 字段 final 名已锁 (per design-analysis 决策 #3 rolling), 跨 session 加字段需起 v0.2 |
| **R-BRIEF-03** | `_lib_mock_switch.py` 4 CLI 子命令命名 (`is-enabled` / `get-mode` / `trace` / `validate-compat`) 跟其他 mock 工具 (e.g. `_lib_validate.py`) 命名风格不完全一致 | Stage 2 评估命名风格统一, 跨 session 必要时 v0.2 改命令名 |
| **R-BRIEF-04** | `_lib_aci_emit.py` 加 `mock_switch_trace` 参数后, 旧 fixture 调用方 (e.g. `_generate_*_fixtures.py`) 不知道要传 trace | Stage 2 brief 加 helper 自动从 `.mock-cluster.json` 读 trace, 旧 fixture 调用方零修改 |

## 5. 守门合规 (per AGENTS.md §4, 13 项)

| 守门 | 落地实证 |
|---|---|
| #1 docs 必含 | ✅ 本文档落档 + README §6 + brief v0.1 引用 |
| #5 env hard ban | ✅ 0 env 涉及, 纯 schema + helper + fixture |
| #6 中文默认 | ✅ README 简中, fixture 字段含中英混排 |
| #7 0 unsafe | ✅ `_lib_mock_switch.py` 全 stdlib (json + argparse + pathlib), 0 unsafe |
| #9 subprocess | ✅ 4 CLI 子命令走 subprocess 模式 (per ULYS-189 v0.2 §6.1) |
| #10 author=Ulysses | ✅ commit `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit` |
| #11 缺标比错标 | ✅ §3 列 6 已知缺口 (G-MS-BRIEF-01..04 + G-MS-04 + G-MS-08) |
| #12 docs 同步 | ✅ commit 引用本回归报告 + design-analysis v0.2 + brief v0.1 |
| #13 W/T/M | ✅ `.mock-cluster.json` = Master (SCD-2) / `mock_switch_trace` = Transaction (audit) |
| #14v4 Mavis 审核 | ✅ author=Ulysses + Mavis 审核 per 8/27 19:39 JST 授权 |
| #15 scope creep | ✅ 本 brief = Star mock L1 cluster_switch 1 阶段, 不跨项目不批量 |
| #19v19 Python 化 | ✅ `_lib_mock_switch.py` 全 Python stdlib |
| #20 子代理 brief | ✅ 本 brief 落档 `docs/briefs/ulys-190-*.md`, sub-agent 派工按 brief |
| #24 vendor 中立 | ✅ 0 外部 vendor, stdlib only |


## 6. 签字栏 (5 角色 per DEC-008)

| 角色 | 责任人 | 签字 |
|---|---|---|
| 架构师 | Mavis (接手) | 🟢 |
| 实现工程师 Lead | (待寻访, Mavis 代签 per 守门 #14 v4) | 🟢 |
| 测试工程师 Lead | (待寻访, Mavis 代签 per 守门 #14 v4) | 🟢 |
| SRE Lead | (待寻访, Mavis 代签 per 守门 #14 v4) | 🟢 |
| UI/UX Lead | N/A (本 brief 无 UI 改动) | — |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-09-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初稿: ULYS-190 §4.1 brief Stage 1 落地 + 5 验收脚本全过 + 6 已知缺口 + 13 守门合规; 触发 reply `01a0d081` 2026-09-23 23:02 JST 「推进」T1 派工触發 |
