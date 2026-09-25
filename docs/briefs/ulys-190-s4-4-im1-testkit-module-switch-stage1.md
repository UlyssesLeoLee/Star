# ULYS-190 §4.4 第 2 笔 brief — IM1.0 im-testkit L3 module_switch 雛形 (从 IM1.0 阻力最小开始)

> **狀態**: 🟡 Draft v0.1 (ULYS-190 v0.4 approved 後, 待 Ulysses 派工觸發)
> **日期**: 2026-09-25
> **制定者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代審
> **觸發**: ULYS-190 reply `01a0dafe` 2026-09-25 23:54 JST 「a」 = 接受 reply `01a0dafc` (a) §4.4 brief 派工 1 sub-agent 1 切点
> **設計稿引用**: [`171d3ade` v0.4 §4.4](../architecture/2026-09-22-mock-switches/00-design-analysis.md) + `5682d4c` IM1.0 PR #23 (已 merged to dev)
> **目標 repo / branch**: `D:/IM1.0` (跨 repo 落地), PR → `origin/dev`

## 0. 目的 (Purpose)

ULYS-190 §4.4 第 1 阶段: 给 IM1.0 `crates/im-testkit/.aci.json` 的 3 个 mock plugin (mock_grpc / mock_rest / mock_ws_frames) 填上 `modules` 雛形, 把现有的 `pub fn` mock 单元一一映射成 module_switch, 让 LLM 通过 `mock_switch_trace` 一眼能定位「是 mock 哪个具体函数失败」。

为什么从 IM1.0 开始 (per 设计稿 v0.4 §2.1 項目 5 ⭐):
- ✅ 阻力最小 — im-testkit 已有 8 个 `.rs` 文件按 module 拆好, `pub fn` 命名稳定
- ✅ IM1.0 PR #23 (commit `5682d4c`) 已 ship 5 plugin 到 dev (含 assertions/fixtures/mock_grpc/mock_rest/mock_ws_frames), 只需填 modules
- ✅ 已有 `aci_compat_version` 字段, 无 schema 变更
- ✅ 3 mock plugin 共 ~40+ `pub fn`, 足够做"第 1 笔 brief 雛形"参考实装范本 (后续 5 项目 mock_*/testkit 套用)

本 brief 是「§4.4 7 项目 module_switch 扩展」的**第 1 个 sub-agent brief**, 对齐 ULYS-190 §4.1 §4.2 brief 拆解範式 (per AGENTS.md 守門 #20 + 守門 #15 1 sub-agent 1 切点)。

---

## 1. 範圍 (Scope)

### 1.1 In-Scope (本 brief 必做, 全部在 IM1.0 repo 1 sub-agent 1 切点)

| 1 | **`crates/im-testkit/.aci.json` plugins section modules 填值** — 修改 `.aci.json` (已 ship in dev), 给 3 个 `modules: {}` placeholder plugin 填实际 module 列表:
   - **`mock_grpc`** (per `crates/im-testkit/src/mock_grpc.rs`, 13 pub fn): 填 4 RPC module (`send_message` / `list_messages` / `validate_access_token` / `mark_read`), 每个 module 对应一组 `_request` + `_response` mock fn
   - **`mock_rest`** (per `crates/im-testkit/src/mock_rest.rs`, 16 pub fn): 填 7 endpoint module (`login` / `guest_register` / `create_conversation` / `list_messages_rest` / `send_message_rest` / `presign_media` / `error_responses`), 包含 unauthorized / blocked / idempotent_replay / validation_error / not_found_error 5 个 error sub-module
   - **`mock_ws_frames`** (per `crates/im-testkit/src/mock_ws_frames.rs`, 32 pub fn): 填 8 frame module (`auth` / `send_message` / `edit_message` / `recall_message` / `react` / `mark_read` / `typing` / `ping` / `server_frames`), client_frame + json variant 按业务语义聚合
2. **`mock_switch_trace_format` 改真拼接** — 修改 `.mock-cluster.json` `mock_switch_trace_format` 字段, 从静态字符串改成「plugins=[...], modules={plugin_id: [count]={...}, [samples]}}」模板 (per 设计稿 §3.4 + G-MS-08 截断到 ~80 字)
3. **新增 `_lib_mock_switch_im.py` Python helper** — IM1.0 跨语言 dispatch helper (Star 有 `_lib_mock_switch.py`, IM1.0 需要 Rust-bound 等价物). 范围: 读 `.aci.json` + `.mock-cluster.json` → 输出 `mock_switch_trace` dict, ~50 LOC
4. **新增 `tests/im_testkit_module_switch.rs`** — IM1.0 module_switch 单元测试, 验证 19 个 module 都能被 switch reader 正确读出 (~80 LOC)
5. **更新 `crates/im-testkit/src/lib.rs` 模块文档** — 在 lib.rs 加 `## module_switch 接入` 段落, 说明 `_lib_mock_switch_im.py` 怎么用 (~15 行)
6. **新增 `crates/im-testkit/docs/regression-report-stage1-module-switch-2026-09-25.md`** — 验证报告 (per AGENTS.md §3), 含 19 module × enabled/mode 矩阵
7. **更新 design-analysis v0.5** — `docs/architecture/2026-09-22-mock-switches/00-design-analysis.md` v0.4 → v0.5, 加 §10 module_switch 落地回顧 (per IM1.0 §4.4 stage1 经验), 同步更新 §7 守门自检 #11 已知缺口表 (G-MS-04 跨项目命名一致性 ✅ 部分解)
8. **单 commit 收官** (per 守門 #10 author=Ulysses)

### 1.2 Out-of-Scope (本 brief 不做, 跨 session 续做)

- ❌ IM1.0 `mock_grpc` / `mock_rest` / `mock_ws_frames` 之外的 plugin (assertions / fixtures 已在 PR #23 填完)
- ❌ IM1.0 `_lib_mock_switch_im.py` 的 Rust native 版本 (Python helper 已够 §4.4 stage1, Rust native 跨 session)
- ❌ IM1.0 mock 数据生成器 (`_generate_*` 系列) 接入 module_switch (跨 session)
- ❌ IM1.0 IT/ST 测试包接入 module_switch (跨 session, 待 IT 套件稳定后)
- ❌ Star / RGS / CATs / IDE1.0 / GitGit module_switch 扩展 (per §4.4 stage2-7, 5 项目跨 session)
- ❌ Ada module_switch (per G-MS-01 降級模式 L1 only, 永久跳过)
- ❌ 跨项目 CI `mock-switch-validate` 验证 module_switch 字段 (per §4.5 已在 Star ship, IM1.0 path 验证跨 session)
- ❌ 开关变更审计日志 (per G-MS-05 Transaction audit SCD-2, 跨 session)
- ❌ mock_switch_trace 字段长度截断到 ~80 字 (per G-MS-08, 本 brief 输出 ~150 字, 截断跨 session)
- ❌ 守門 #15 限制: 1 sub-agent 1 切点, 不批量

---

## 2. 落地清单 (Deliverables)

| 序 | 路径 | 类型 | 说明 |
|---|---|---|---|
| 1 | `crates/im-testkit/.aci.json` | 修改 | 3 plugin × N module 全填 (mock_grpc 4 + mock_rest 7 + mock_ws_frames 8 = 19 module) |
| 2 | `crates/im-testkit/.mock-cluster.json` | 修改 | `mock_switch_trace_format` 改真拼接模板 |
| 3 | `crates/im-testkit/scripts/_lib_mock_switch_im.py` | 新文件 | Python module_switch reader (~50 LOC) |
| 4 | `crates/im-testkit/tests/im_testkit_module_switch.rs` | 新文件 | 单元测试 19 module × switch state (~80 LOC) |
| 5 | `crates/im-testkit/src/lib.rs` | 修改 | 加 `## module_switch 接入` 段落 (~15 行) |
| 6 | `crates/im-testkit/docs/regression-report-stage1-module-switch-2026-09-25.md` | 新文件 | 19 module 矩阵验证报告 (per AGENTS.md §3) |
| 7 | `docs/architecture/2026-09-22-mock-switches/00-design-analysis.md` | 修改 | v0.4 → v0.5 加 §10 module_switch 落地回顧 + G-MS-04 更新 |

**commit 数**: 1 commit (per 守門 #10 author=Ulysses) — 全部在 IM1.0 repo

---

## 3. module_switch 命名锁定 (per G-MS-04 跨项目一致性)

| Plugin | Module ID | 来源 (mock_*.rs `pub fn` 名, 去掉 `_request` / `_response` / `_frame` / `_json` 后缀) | mode 默认 |
|---|---|---|---|
| `mock_grpc` | `send_message` | `send_message_request` / `send_message_response` | offline |
| `mock_grpc` | `list_messages` | `list_messages_request` / `list_messages_response` | offline |
| `mock_grpc` | `validate_access_token` | `validate_access_token_request` / `validate_access_token_response` (+ `_invalid` variant) | offline |
| `mock_grpc` | `mark_read` | `mark_read_request` / `empty_response` | offline |
| `mock_rest` | `login` | `login_request` / `login_response` / `login_unauthorized_response` | offline |
| `mock_rest` | `guest_register` | `guest_register_request` / `guest_register_response` | offline |
| `mock_rest` | `create_conversation` | `create_conversation_request` / `create_conversation_response` / `create_conversation_blocked_response` | offline |
| `mock_rest` | `list_messages_rest` | `list_messages_response` (REST 版) | offline |
| `mock_rest` | `send_message_rest` | `send_message_rest_request` / `send_message_rest_response` / `send_message_rest_idempotent_replay` | offline |
| `mock_rest` | `presign_media` | `presign_media_request` / `presign_media_response` | offline |
| `mock_rest` | `error_responses` | `validation_error_response` / `not_found_error_response` | offline |
| `mock_ws_frames` | `auth` | `auth_request_frame` / `auth_request_json` | offline |
| `mock_ws_frames` | `send_message` | `send_message_frame` / `send_message_json` | offline |
| `mock_ws_frames` | `edit_message` | `edit_message_frame` | offline |
| `mock_ws_frames` | `recall_message` | `recall_message_frame` | offline |
| `mock_ws_frames` | `react` | `react_frame` | offline |
| `mock_ws_frames` | `mark_read` | `mark_read_frame` | offline |
| `mock_ws_frames` | `typing` | `typing_frame` | offline |
| `mock_ws_frames` | `ping` | `ping_frame` / `ping_json` / `pong_frame` / `pong_json` | offline |
| `mock_ws_frames` | `server_frames` | `connected_frame` / `ack_success_frame` / `ack_idempotent_replay_json` / `ack_error_json` / `message_new_frame` / `message_edited_frame` / `message_recalled_frame` / `reaction_added_frame` / `presence_update_frame` / `typing_broadcast_frame` / `force_disconnect_frame` / `validation_error_body` (12 server frame 归 1 module, 因业务场景共享) | offline |

**总计**: 19 module (mock_grpc 4 + mock_rest 7 + mock_ws_frames 8)

**命名约定 (跨项目锁定 per G-MS-04)**:
- 全部 `snake_case`
- 跟 source `pub fn` 名一一对应 (去除 `_request` / `_response` / `_frame` / `_json` / `_invalid` 变体后缀)
- `mock_ws_frames` 的 server frame group 用 `server_frames` 聚合, 避免 module 数量爆炸 (后续 §4.4 stage2-7 跨项目沿用此 pattern)

---

## 4. mock_switch_trace 真拼接模板 (per 设计稿 §3.4 + G-MS-08)

### 4.1 当前 (PR #23 ship 的静态 trace)

```json
"mock_switch_trace_format": "cluster.enabled={cluster.enabled}, cluster.mode={cluster.mode}, plugins=[assertions,fixtures,mock_grpc,mock_rest,mock_ws_frames], modules=per_plugin (TBD)"
```

### 4.2 落地后 (本 brief 改真拼接)

```json
"mock_switch_trace_format": "cluster.enabled={cluster.enabled}, mode={cluster.mode}, plugins=[assertions(3m),fixtures(5m),mock_grpc(4m),mock_rest(7m),mock_ws_frames(8m)]=27/27 total"
```

`_lib_mock_switch_im.py` 输出 dict 模板:

```python
{
    "cluster": {"enabled": True, "mode": "offline", "version": "0.1.0-draft"},
    "plugins": {
        "assertions": {"enabled": True, "modules_total": 3, "modules_enabled": 3},
        "fixtures": {"enabled": True, "modules_total": 5, "modules_enabled": 5},
        "mock_grpc": {"enabled": True, "modules_total": 4, "modules_enabled": 4},
        "mock_rest": {"enabled": True, "modules_total": 7, "modules_enabled": 7},
        "mock_ws_frames": {"enabled": True, "modules_total": 8, "modules_enabled": 8}
    },
    "summary": "27/27 modules enabled, trace ~120 chars"
}
```

(trace 字段 ~120 字, 超 G-MS-08 推荐 ~80 字阈值, 跨 session brief 加 truncate 截断; 本 brief 不做)

---

## 5. 验收 (Acceptance)

| 序 | 项目 | 命令 / 指标 | 预期 |
|---|---|---|---|
| 1 | `cargo fmt --check` (per 守門 #1 v25) | `cargo fmt -p im-testkit --check` | ✅ 干净 |
| 2 | `cargo check -p im-testkit --all-targets` (守門 #1 v25 + 6) | `cargo check -p im-testkit --all-targets` | ✅ 干净 |
| 3 | `cargo clippy -p im-testkit --all-targets -- -D warnings` (守門 #1 v25) | `cargo clippy ... -D warnings` | ✅ 干净 |
| 4 | `cargo test -p im-testkit --all-targets` (守門 #6) | `cargo test -p im-testkit --all-targets` | ✅ 87 + N 全过 (N = 19 module 测试) |
| 5 | `python tools/mock-switch-validate.py validate-one im1` (Star 已 ship) | validate-one im1 | ✅ 19 modules 全 enabled, cluster_ok=True |
| 6 | `bash scripts/aci-smoke.sh` (ULYS-191 §4.3.3 已 ship) | 4 step verify | ✅ PASS |
| 7 | doc 同步 (守門 #1 #12 #24) | `lib.rs` 文档 + regression report + design v0.5 | ✅ 三处同步 |

---

## 6. 已知缺口 (本 brief 范围, per 守門 #11 缺標比錯標)

- **G-MS-BRIEF-S44-01**: `_lib_mock_switch_im.py` 是 Python, IM1.0 主仓是 Rust — 跨语言调用通过 `subprocess` (per 守門 #9). Rust native 版本跨 session.
- **G-MS-BRIEF-S44-02**: `mock_switch_trace_format` 拼接 ~120 字, 超 G-MS-08 推荐 ~80 字. 截断跨 session.
- **G-MS-BRIEF-S44-03**: `mock_ws_frames` 的 server frame group (12 frame) 归到 `server_frames` 1 module, 简化粒度但失去 frame 级开关. 跟设计稿 §3.4 粒度原则有 trade-off, 跨 session 评估.
- **G-MS-BRIEF-S44-04**: `_lib_mock_switch_im.py` 不支持 hot reload (per G-MS-09 v0.1 不做), 改 `.aci.json` 后需重跑 test.
- **G-MS-BRIEF-S44-05**: 24 module 的命名跟 aux-13 协议 frame 名一致性, 跨项目 (Star/RGS/CATs/IDE1.0/GitGit) 是否沿用此 convention 待 §4.4 stage2 验证 (G-MS-04 跨项目 plugin_id/module_id 命名锁版).

---

## 7. 守门对齐 (per AGENTS.md §4)

| 守门 | 检查 |
|---|---|
| #1 docs 必含 | regression report + lib.rs 文档 + design v0.5 (本 brief 7 交付物包含) ✅ |
| #3 5 域 Lead | 1 sub-agent 1 切点 (本 brief), 派工触发需 D-Boy 明示 / 5 域 Lead 真人到位 🟡 |
| #5 env hard ban | 0 影响 |
| #6 PowerShell only | 0 bash `&&` ✅ |
| #7 0 unsafe | 0 unsafe Rust 代码 |
| #9 subprocess | `_lib_mock_switch_im.py` 跨 Rust/Python subprocess 读 ✅ |
| #10 author=Ulysses | commit author 统一 |
| #11 缺標比錯標 | §6 列 5 已知缺口 |
| #12 docs 同步 | regression report + lib.rs + design v0.5 三处 |
| #13 W/T/M | module_switch 配置属 Master SCD-2 |
| #14 v4 Mavis 审核 | author=Ulysses, 跨 IM1.0 repo brief 走 Mavis |
| #15 1 sub-agent 1 切点 | 本 brief 严格遵守, 1 sub-agent, 1 repo, 1 brief |
| #19 v19 conflict-of-interest | 0 conflict |
| #20 dispatcher brief | 本 brief 跟 §4.1 §4.2 範式一致 |
| #24 docs 同步 | lib.rs + README + design-analysis + regression report 4 处同步 |

---

## 8. 跨 session 续做入口 (本 brief 完成後)

1. 🟡 **§4.4 stage2-7** — 6 项目 (Star/RGS/CATs/IDE1.0/GitGit/Ada[降級]) module_switch 扩展, 每项目 1 brief, 共 ~5-6M tokens (跨 session, 大塊)
2. 🟡 **G-MS-08 mock_switch_trace 截断到 ~80 字** (本 brief 输出 ~120 字, 跨 session)
3. 🟡 **G-MS-05 開關變更審計日誌** (Transaction audit SCD-2, 跨 session)
4. 🟡 **G-MS-09 hot reload** (v0.2 評估, 跨 session)
5. 🟡 **Star _lib_mock_switch.py 真拼接升级** (跟 IM1.0 _lib_mock_switch_im.py 模板对齐, 跨 session)
6. 🟡 **CI `mock-switch-validate` 验证 module_switch 字段** (Star ship 时只校验 cluster, 加 module 级校验, 跨 session)
7. 🟡 **Layer 1→Layer 2→Layer 3 贯通验收** (per AGENTS.md §3, 跨 session, 顶层 sub-task)

---

**brief 制定**: 2026-09-25 JST, Ulysses (1 人公司, Mavis 接手代審 per DEC-008)
**brief 版本**: v0.1 Draft
**brief 等触发**: D-Boy 拍板 (a) 派工 1 sub-agent 1 切点 / (b) 调整 brief 范围 / (c) 暂搁 §4.4 优先其他 issue
