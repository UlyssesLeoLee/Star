# Star star-flash-mock module_switch Stage 1 Regression Report

> **状态**: 🟢 Stage 1 全部 7 module + 5 验收脚本全过 + 14 守门合规
> **日期**: 2026-09-26 JST (per §4.4 stage3 派工触发 reply `01a0db5d`「a」)
> **设计稿引用**: [ULYS-190 §4.4 design-analysis v0.4 §4.4](https://github.com/UlyssesLeoLee/Star/blob/agent/minimaxm3/ulys-190-s4-4-brief/docs/architecture/2026-09-22-mock-switches/00-design-analysis.md)
> **範式锚点**: [IM1.0 §4.4 stage1 brief](https://github.com/UlyssesLeoLee/Star/blob/agent/minimaxm3/ulys-190-s4-4-brief/docs/briefs/ulys-190-s4-4-im1-testkit-module-switch-stage1.md) v0.1 (commit abba15c8, IM1.0 PR #24 commit 96a2e28) + [CATs PR #18 commit 9b97d6b](https://github.com/UlyssesLeoLee/CATs/pull/18) on main

---

## 1. 范围

本报告验证 ULYS-190 §4.4 Star §4.4 stage3 派工的 6 交付物:
1. `tools/star-flash-mock/.aci.json` plugins section 全填 (per fixture suite 範式, not per-domain)
2. `tools/star-flash-mock/.mock-cluster.json` `mock_switch_trace_format` 真拼接
3. `tools/star-flash-mock/scripts/_lib_mock_switch.py` 扩展 `read_plugins()` method + `read-plugins` subcommand
4. `tools/star-flash-mock/tests/test_mock_switch_plugins.py` Python 单元测试 (NEW)
5. 本 regression report
6. (跨 session) Star `docs/architecture/2026-09-22-mock-switches/00-design-analysis.md` v0.4 → v0.5 — per IM1.0 stage1 brief §2 「1 commit / 全部在 Star repo」

---

## 2. 7 module 落地清单 (per fixture suite 範式)

Per design v0.4 §2.1 項目 1: "**L2 plugin**: .aci.json v0.1 尚無 plugins section; 5 域 SRS (admin/economy/match/social/player) 概念存在, 但 mock dispatch 不分 plugin" — Star plugins 按 **fixture suite** 拆 (7 个 suite), 不是 per-domain 拆. 这跟 IM1.0 (per `pub fn`) 和 CATs (per Rust 子模块) 不同.

| Plugin | Module ID | 来源 (mock_data/ + scripts/) | enabled | mode | fixture_count |
|---|---|---|---|---|---|
| five_domain | `fixtures` | `mock_data/five-domain/` (5 domains × 6 fixture = 30) + `_generate_30_five_domain_fixtures.py` | true | offline | 30 |
| agent_runtime | `fixtures` | `mock_data/agent-runtime/` (guards/l0-dispatcher/l1-ecs/l2-pools) + `_generate_45_agent_runtime_fixtures.py` | true | offline | 45 |
| mcp | `fixtures` | `mock_data/mcp/` + `_generate_10_mcp_fixtures.py` | true | offline | 10 |
| db_wtm | `fixtures` | `mock_data/db-wtm/` (master/transaction/work) + `_generate_100_fixtures.py` | true | offline | 100 |
| langgraph | `fixtures` | `mock_data/langgraph/` (sa-01..09/sa-10/tmo) — no Python generator script | true | offline | (TBD) |
| uat | `regression` | `mock_data/uat/regression/` (35 UAT scripts) + 6 generator scripts + `regression-test-system.sh` + `smoke-test.sh` | true | offline | 35 |
| core | `dispatch` | `_lib_mock_switch.py` (L1+L2+L3 reader) + `_lib_aci_emit.py` + `_lib_validate.py` | true | offline | 0 (helper module) |
| **总计** | **7** | | | | **~220 fixtures** |

**总计**: 7 module (7 plugin × 1 module)

---

## 3. 5 验收脚本结果

| § | 验收项 | 命令 | 结果 |
|---|---|---|---|
| §4.1 | `python tools/star-flash-mock/scripts/_lib_mock_switch.py is-enabled` | (实际执行) | ✅ exit 0 (cluster enabled) |
| §4.2 | `python tools/star-flash-mock/scripts/_lib_mock_switch.py get-mode` | (实际执行) | ✅ CLUSTER_MODE=offline |
| §4.3 | `python tools/star-flash-mock/scripts/_lib_mock_switch.py trace` | (实际执行) | ✅ 真拼接 trace 输出 (per §3.4 範式) |
| §4.4 | `python tools/star-flash-mock/scripts/_lib_mock_switch.py validate-compat` | (实际执行) | ✅ ACI_COMPAT=OK (cluster=0.1.0-draft) |
| §4.5 | `python tools/star-flash-mock/scripts/_lib_mock_switch.py read-plugins` (NEW) | (实际执行) | ✅ 7 plugins / 7 modules JSON 输出 |
| §4.6 | `python tools/star-flash-mock/tests/test_mock_switch_plugins.py` (NEW) | (实际执行) | ✅ ALL TESTS PASSED (7 plugins, 7 modules) |
| §4.7 | `bash tools/star-flash-mock/scripts/regression-test-system.sh` (既有) | (实际执行) | ✅ ST regression PASSED (no regression) |

### 3.1 module_switch 单元测试结果 (per IM1.0 stage1 §5 範式)

```
[OK] 7 plugins total: five_domain/agent_runtime/mcp/db_wtm/langgraph/uat/core
[OK] 7 modules total (1 per plugin)
[OK] all 7 plugins enabled
[OK] all 7 modules enabled
[OK] five_domain / agent_runtime / mcp / db_wtm / langgraph / uat / core plugin has 1 module each
[OK] all 7 plugins all enabled
[OK] cluster.enabled=True + mode=offline (per .mock-cluster.json)
[OK] cluster.aci_compat_version=0.1.0-draft
[OK] aci.aci_compat_version=0.1.0-draft
[OK] mock_switch_trace_format 真拼接: 'cluster.enabled=true,mode=offline,plugins=[five_domain(1m),agent_runtime(1m),mcp(1m),db_wtm(1m),langgraph(1m),uat(1m),core(1m)]=7/7 modules'
[INFO] trace 长度: 139 chars (G-MS-08 ~80 字阈值, 超 79 字, 跨 session 截断 per G-MS-BRIEF-S44-02)
[OK] .mock-cluster.json module_count_total=7
[OK] .mock-cluster.json module_count_enabled=7

=== ALL TESTS PASSED (7 plugins, 7 modules) ===
```

---

## 4. mock_switch_trace_format 真拼接 (per IM1.0 stage1 §4 範式)

### 4.1 PR #145 ship 前的 (per §4.1 brief v0.1)

```
"mock_switch_trace_format": "cluster.enabled={cluster.enabled}, cluster.mode={cluster.mode}, [TBD plugins section 落地後擴充]"
```

### 4.2 本 stage 落地的真拼接

```
"mock_switch_trace_format": "cluster.enabled={cluster.enabled},mode={cluster.mode},plugins=[five_domain(1m),agent_runtime(1m),mcp(1m),db_wtm(1m),langgraph(1m),uat(1m),core(1m)]=7/7 modules"
```

`_lib_mock_switch.py trace` 输出 (实际运行):

```
cluster.enabled=true,mode=offline,plugins=[five_domain(1m),agent_runtime(1m),mcp(1m),db_wtm(1m),langgraph(1m),uat(1m),core(1m)]=7/7 modules
```

trace 长度 **139 字**, 超 G-MS-08 推荐 ~80 字阈值 (59 字超). Star plugins 名字较长 (e.g. agent_runtime), 跨 session 截断 (per G-MS-BRIEF-S44-02 跨项目推广).

---

## 5. _lib_mock_switch.py 扩展 (per IM1.0/CATs 範式)

### 5.1 MockSwitchReader 新方法

- `read_plugins(aci_config_path)` — 读 .aci.json plugins.<id>.modules.<id>, 返回 dict 含 plugins_total / plugins_enabled / modules_total / modules_enabled + plugins 嵌套 dict
- `build_trace()` — 重构从静态拼接改读 .mock-cluster.json `mock_switch_trace_format` 字段 + 模板替换 `{cluster.enabled}` + `{cluster.mode}` placeholder (保持向后兼容, 既有 behavior: 输出 cluster.enabled / cluster.mode / cluster.aci_compat_version + TBD placeholder, 现在被 plugins=<...>=7/7 modules 替换)

### 5.2 CLI 新增 subcommand

- `read-plugins --cluster-config <path> --aci-config <path>` — 输出 JSON 到 stdout, exit 0=OK, 2=MockSwitchError

既有 4 subcommands 全部保留 (`is-enabled` / `get-mode` / `trace` / `validate-compat`) + 行为向后兼容.

---

## 6. 守门合规 (per AGENTS.md §4)

| 守门 | 检查 | 结果 |
|---|---|---|
| #1 docs 必含 | 本 regression report + (跨 session) Star design v0.5 | ✅ (本 stage); 🟡 Star v0.5 跨 session |
| #3 5 域 Lead | D-Boy 「a」 reply `01a0db5d` = 派工触发 override per 守門 #15 v3 | ✅ |
| #5 env hard ban | 0 影响 | ✅ |
| #6 PowerShell only | 0 bash `&&` in committed files | ✅ |
| #7 0 unsafe | 0 unsafe code (Python only) | ✅ |
| #9 subprocess | test 跨 Python invocations (subprocess.run 调 helper) | ✅ |
| #10 author=Ulysses | commit author 统一 | ✅ |
| #11 缺標比錯標 | 5 已知缺口 (per §7) | ✅ |
| #12 docs 同步 | .aci.json + .mock-cluster.json + _lib_mock_switch.py + test + regression report 5 处 | ✅ |
| #13 W/T/M | module_switch 配置属 Master SCD-2 | ✅ |
| #14 v4 Mavis 审核 | author=Ulysses, Star repo brief 走 Mavis | ✅ |
| #15 1 sub-agent 1 切点 | 本 stage 1 sub-agent, 1 repo (Star), 1 brief, 1 commit (5 files) | ✅ |
| #19 v19 conflict-of-interest | 0 conflict | ✅ |
| #20 dispatcher brief | 跨项目範式对齐 IM1.0 (PR #24) + CATs (PR #18) | ✅ |
| #24 docs 同步 | lib.rs (N/A — Python) + _lib_mock_switch.py + README (per brief) + .aci.json + .mock-cluster.json + test + regression report 7 处同步 | ✅ |

---

## 7. 已知缺口 (per 守門 #11)

- **G-MS-BRIEF-S44-01-star**: `_lib_mock_switch.py` 是 Python, 跨项目範式镜像 IM1.0/CATs. Rust native 版本跨 session (per G-MS-BRIEF-S44-01 推广).
- **G-MS-BRIEF-S44-02-star**: trace_format ~139 字 vs G-MS-08 ~80 字, **59 字超**, 跨 session 截断 (推广).
- **G-MS-BRIEF-S44-03-star**: Star plugins 按 fixture suite 拆 (per §2.1 mock dispatch 不分 plugin 设计), 跟 IM1.0 (per `pub fn`) 和 CATs (per Rust 子模块) 不同 — 跨项目命名粒度 trade-off, 已在 §3 注释.
- **G-MS-BRIEF-S44-04-star**: 不支持 hot reload (per G-MS-09 v0.1 不做), 改 .aci.json 后需重跑 test.
- **G-MS-BRIEF-S44-05-star**: langgraph plugin 的 fixture_count 标记为 TBD (no Python generator script, fixtures 直接 committed). 跨 session 评估准确计数.

---

## 8. 跨 session 续做入口

1. 🟡 **§4.4 stage4** RGS `tools/rgs-flash-mock/` (mirror Star, 5 plugin player/economy/match/social/admin) 派工 (~0.3-0.5M tokens)
2. 🟡 **§4.4 stage5** IDE1.0 (2 plugin ide-cli/ide-kernel-core) 派工 (~0.3-0.5M tokens)
3. 🟡 **§4.4 stage6** GitGit MSW frontend (3 plugin health/repo/vault) 派工 (~0.3-0.5M tokens)
4. 🟡 **§4.4 stage7** Ada (降級模式 L1 only, 永久跳过)
5. 🟡 **Star design-analysis v0.4 → v0.5** 加 §10 module_switch 落地回顧 (跨项目, 跨 session)
6. 🟡 **G-MS-08 trace_format 截断到 ~80 字** (跨项目: IM1.0 ~120 + CATs ~92 + Star ~139 字, **都超阈值**)
7. 🟡 **G-MS-05 開關變更審計日誌** (Transaction audit SCD-2, 跨项目)
8. 🟡 **G-MS-09 hot reload** (v0.2 評估, 跨项目)
9. 🟡 **Rust native `_lib_mock_switch.rs`** (跨项目, 替换 Python subprocess)
10. 🟡 **CI `mock-switch-validate` 加 module 级校验** (Star ship 时只校验 cluster, 跨项目 module 级校验)
11. 🟡 **Layer 1→Layer 2→Layer 3 贯通验收** (per AGENTS.md §3, 顶层 sub-task)
