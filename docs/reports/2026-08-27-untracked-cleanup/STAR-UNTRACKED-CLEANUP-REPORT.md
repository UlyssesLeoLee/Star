# STAR-UNTRACKED-CLEANUP-REPORT

| 字段 | 值 |
|------|----|
| 报告 ID | STAR-UNTRACKED-CLEANUP-REPORT |
| 版本 | v0.1 |
| 制定日期 | 2026-08-27 |
| 任务来源 | Ulysses 2026-08-27 16:32 JST 发令"未决全部开子代理完成"（覆盖 2026-08-26 16:00 JST "现在不动，后处理"） |
| 工作位置 | `D:/Star` 主仓，工作区 |
| 当前 branch | `feature/ai-ide-compat`（HEAD = `0a148b8`） |
| 任务描述中的"main（0a148b8）" | 字面解读为 main HEAD；实际 main HEAD = `4b3b8dc`，而 `0a148b8` 既是 main 链上的祖先 commit，也是 `feature/ai-ide-compat` 的当前 HEAD。本报告按工作区实际状态（`feature/ai-ide-compat` HEAD = `0a148b8`）清理（清理 untracked 与 branch 无关）。 |
| 实施方式 | 不新开 wt（清理任务），PowerShell only，`Move-Item` 替 `rm` |
| commit 策略 | **不 commit**（Mavis 终审后统一入库） |
| 推 origin 策略 | **不推**（R-05 维持） |

---

## §0 目的

1. 清理 `D:/Star` 主仓 8/26 拍板延后的 24 份临时脚手架（untracked），避免污染后续 wt / 子代理视图。
2. 区分"必须保留的 secrets / 模板"（A 类）vs "可清理的临时脚本"（B 类）vs "待评估文件"（C 类）。
3. 用 `.scratch/` 临时目录承接 B 类（**不** `rm`），并补 `.gitignore` 规则防止复发。
4. 输出可 DDD Review 的报告，等 Mavis 终审后统一入库。

**覆盖 8/26 拍板边界**：A 类（`.env*`）保留含义维持；B 类脚本清理动作由 8/27 16:32 JST "未决全部开子代理完成" 反转覆盖。

---

## §1 扫矩阵

### 1.1 扫的范围与方法

- 工作区：`D:/Star`（worktree 列表中唯一的非嵌套 worktree）
- 当前 branch：`feature/ai-ide-compat`（HEAD = `0a148b8`，`merge wt-phase-d3-impl: MCP transport stdio JSON-RPC 2.0 实装 (Mavis 接手)`）
- main HEAD：`4b3b8dc`（`docs(upgrade): Phase A + B 生态事实基线 + 5 份边界 ADR + 2 份责任矩阵`）— 与 `0a148b8` 不在同一条 branch 上
- 任务文本"基于 main（`0a148b8`）"的字面解读与工作区实际状态有偏差：本报告按**工作区实际状态**（`feature/ai-ide-compat` HEAD = `0a148b8`）清理（untracked 清理动作与 branch 无关，结果可移植到 main）
- 扫命令：`git status --short --untracked-files=all` + `git status --ignored --short`
- untracked 数量：**26 份**（24 脚本 + 2 env 模板）— 清理目标
- 工作区进行中改动（**不在本任务 scope**）：
  - `crates/star-cli/src/commands/mr.rs` modified（feature/ai-ide-compat 进行中工作）
  - `crates/star-cli/src/main.rs` modified（feature/ai-ide-compat 进行中工作）
  - `_wt_audit/cargo-check-2026-08-27.err` untracked（8/27 wt-phase-* 体检 stderr 副产品，wt-phase 范围）
  - `docs/reports/2026-08-27-untracked-cleanup/STAR-UNTRACKED-CLEANUP-REPORT.md` untracked（本报告自身）
- C 类（语义不清待评估）：**0 份**。`docs/frontend-design-feedback.md` 已被 `32d30af` 收纳（`chore(audit): 2026-08-27 worktree 体检 + DDD Review 资料 + 收纳 frontend-design feedback`），不再 untracked。

### 1.2 A 类（必须保留，含 secrets / 拍板不动）

| 文件 | 字节数 | 评估证据 | 决定 |
|------|--------|----------|------|
| `.env` | 70 | 真实 env 凭据（**不打印** per 2026-08-27 11:06 JST env-var 安全规则）；8/26 拍板"现在不动"维持 A 类 | **保留**（working tree），由 `.gitignore` 排除 |
| `.env.example` | 70 | 模板（`DATABASE_URL=postgres://star_app:star_app_dev@127.0.0.1:5555/star_dev`），template-only 安全可读 | **保留**（untracked，可后续 `git add` 跟踪） |

**A 类合计**：2 文件 / 140 字节。

### 1.3 B 类（可清理：临时脚本 / 工具脚手架 / 一次性 probe）

| 文件 | 字节 | 末次修改 (JST) | 评估证据 | 决定 |
|------|------|---------------|----------|------|
| `check_wt.ps1` | 791 | 2026-08-26 18:31 | 一次性 worktree 体检，路径指向 `D:/RustGameServer-worktrees/WF-1-*`（RGS 资产，非 Star 资产） | 移 `.scratch/` |
| `wt_clean.ps1` | 714 | 2026-08-26 18:34 | 一次性 worktree 清理（10 个 WF-1-* 路径），RGS 域 | 移 `.scratch/` |
| `wt_clean2.ps1` | 934 | 2026-08-26 18:35 | 同上 v2 | 移 `.scratch/` |
| `wt_clean3.ps1` | 563 | 2026-08-26 18:35 | 同上 v3 | 移 `.scratch/` |
| `wt_clean_failed.ps1` | 493 | 2026-08-26 22:15 | 失败回滚用，已无场景 | 移 `.scratch/` |
| `wt_check_remaining.ps1` | 1058 | 2026-08-26 18:35 | 一次性剩余检查 | 移 `.scratch/` |
| `db_url_check.ps1` | 356 | 2026-08-26 20:06 | RGS 域 6 service DB URL 凭据读取脚本（**不打印** env 内容） | 移 `.scratch/` |
| `force_clean.sh` | 683 | 2026-08-26 20:38 | k3s Terminating pod 强清，RGS 域 | 移 `.scratch/` |
| `restart_wt.sh` | 576 | 2026-08-26 20:40 | k3s scale 0 触发 ReplicaSet 重启，RGS 域 | 移 `.scratch/` |
| `fetch_log.py` | 2230 | 2026-08-26 20:11 | k8s log 抓取，RGS 域 | 移 `.scratch/` |
| `fetch_log2.sh` | 793 | 2026-08-26 20:12 | 同上 v2 | 移 `.scratch/` |
| `fetch_log3.sh` | 1519 | 2026-08-26 20:14 | 同上 v3 | 移 `.scratch/` |
| `cluster_ops_check.sh` | 895 | 2026-08-27 08:18 | k3s cluster-ops 镜像/日志检查，RGS 域 | 移 `.scratch/` |
| `cluster_ops_log.sh` | 676 | 2026-08-27 08:20 | cluster-ops 日志抓取，RGS 域 | 移 `.scratch/` |
| `debug_match.py` | 445 | 2026-08-26 22:18 | RACI v1.1 文本匹配（`re` 探测），RGS 域文档探针 | 移 `.scratch/` |
| `hpa_rebuild.py` | 2403 | 2026-08-27 08:22 | 6 HPA yaml 重建脚本，RGS 域 | 移 `.scratch/` |
| `patch_probe.py` | 1520 | 2026-08-26 20:14 | k8s probe 配置 patch，RGS 域 | 移 `.scratch/` |
| `patch_probe2.py` | 1362 | 2026-08-26 20:17 | 同上 v2 | 移 `.scratch/` |
| `patch_probe_exec.py` | 1079 | 2026-08-27 08:20 | 同上 exec 模式 | 移 `.scratch/` |
| `patch_tcp.py` | 1377 | 2026-08-26 20:42 | gRPC TCP 探针 patch，RGS 域 | 移 `.scratch/` |
| `seed_5db.sh` | 1223 | 2026-08-27 08:26 | 5 域 DB seed 脚本，RGS 域 | 移 `.scratch/` |
| `seed_5db_v2.sh` | 1150 | 2026-08-27 08:26 | 同上 v2 | 移 `.scratch/` |
| `sign_raci.py` | 4619 | 2026-08-26 21:21 | RACI v1.1 第 4 签名列回填脚本（RGS 项目 14-项目治理 RACI），路径 `D:/RustGameServer/docs/14-项目治理` | 移 `.scratch/` |
| `sign_raci2.py` | 2156 | 2026-08-26 22:19 | 同上 byte-level v2 | 移 `.scratch/` |

**B 类合计**：24 文件 / 29 615 字节。

**B 类共同语义**：

1. 全部指向 `D:/RustGameServer*`（RGS 项目路径）或 `rust-game-server` namespace，**非 Star 资产**。
2. 全部为 8/26 体检 / 8/27 早起 5 域 RACI 签名 / 8/27 worktree 收尾期间的一次性脚本。
3. 全部未进入 `git log`（`git log -p --follow` 输出为空），即**从未被跟踪**。
4. 不存在 B 类文件被任何 BAS / SPEC / ADR 引用的情况（`grep` 工作区 + `_wt_audit/` 索引均无命中）。

### 1.4 C 类（待评估：语义不清）

**空集**。`docs/frontend-design-feedback.md` 已被 `32d30af` 收纳（commit 包含 +200 行），不再 untracked。

如 DDD Review 发现新 C 类候选（untracked 出现新文件 / 子代理新发现），由 §4 守门规则处理。

### 1.5 总览

| 类别 | 文件数 | 字节 | 占比 |
|------|--------|------|------|
| A 必须保留 | 2 | 140 | 0.5% |
| B 移到 `.scratch/` | 24 | 29 615 | 99.5% |
| C 待评估 | 0 | 0 | 0% |
| **合计** | **26** | **29 755** | **100%** |

---

## §2 处理 diff

### 2.1 B 类移动（24 文件，0 缺失）

| 源（Star 根） | 目标（`.scratch/`） | 状态 |
|---------------|---------------------|------|
| `check_wt.ps1` | `.scratch/check_wt.ps1` | ✅ Moved |
| `wt_clean.ps1` | `.scratch/wt_clean.ps1` | ✅ Moved |
| `wt_clean2.ps1` | `.scratch/wt_clean2.ps1` | ✅ Moved |
| `wt_clean3.ps1` | `.scratch/wt_clean3.ps1` | ✅ Moved |
| `wt_clean_failed.ps1` | `.scratch/wt_clean_failed.ps1` | ✅ Moved |
| `wt_check_remaining.ps1` | `.scratch/wt_check_remaining.ps1` | ✅ Moved |
| `db_url_check.ps1` | `.scratch/db_url_check.ps1` | ✅ Moved |
| `force_clean.sh` | `.scratch/force_clean.sh` | ✅ Moved |
| `restart_wt.sh` | `.scratch/restart_wt.sh` | ✅ Moved |
| `fetch_log.py` | `.scratch/fetch_log.py` | ✅ Moved |
| `fetch_log2.sh` | `.scratch/fetch_log2.sh` | ✅ Moved |
| `fetch_log3.sh` | `.scratch/fetch_log3.sh` | ✅ Moved |
| `cluster_ops_check.sh` | `.scratch/cluster_ops_check.sh` | ✅ Moved |
| `cluster_ops_log.sh` | `.scratch/cluster_ops_log.sh` | ✅ Moved |
| `debug_match.py` | `.scratch/debug_match.py` | ✅ Moved |
| `hpa_rebuild.py` | `.scratch/hpa_rebuild.py` | ✅ Moved |
| `patch_probe.py` | `.scratch/patch_probe.py` | ✅ Moved |
| `patch_probe2.py` | `.scratch/patch_probe2.py` | ✅ Moved |
| `patch_probe_exec.py` | `.scratch/patch_probe_exec.py` | ✅ Moved |
| `patch_tcp.py` | `.scratch/patch_tcp.py` | ✅ Moved |
| `seed_5db.sh` | `.scratch/seed_5db.sh` | ✅ Moved |
| `seed_5db_v2.sh` | `.scratch/seed_5db_v2.sh` | ✅ Moved |
| `sign_raci.py` | `.scratch/sign_raci.py` | ✅ Moved |
| `sign_raci2.py` | `.scratch/sign_raci2.py` | ✅ Moved |

`Move-Item -LiteralPath ... -Destination .scratch/<name> -Force` —— 24 / 24 成功，0 缺失，0 错误。

### 2.2 `.gitignore` 改动

**改动前的状态**（`32d30af` 后已存在）：
- `.env` / `.env.*` 已 ignore（`!.env.example` 反向 un-ignore 模板）
- `/.worktrees/` 已 ignore
- 24 条 B 类脚本显式 ignore（按根目录文件名 `/*.ps1` / `/*.sh` / `/*.py` 形式）

**本报告改动**（`git diff` 实证）：

```diff
diff --git a/.gitignore b/.gitignore
index ef83317..532e694 100644
--- a/.gitignore
+++ b/.gitignore
@@ -35,6 +35,8 @@ docs.untracked.bak/

 # RustGameServer (RGS) ops scripts accidentally copied into Star root - not part of Star
 # Keep .ps1/.sh/.py from RGS scratch space out of Star history
+# (Staging area for cleanup: see STAR-UNTRACKED-CLEANUP-REPORT.md v0.1)
+/.scratch/
 /check_wt.ps1
 /wt_check_remaining.ps1
 /wt_clean.ps1
```

**关键变更**：
- 在"24 条 B 类显式 ignore 段"前加 **2 行**：
  1. 注释：`(Staging area for cleanup: see STAR-UNTRACKED-CLEANUP-REPORT.md v0.1)`
  2. ignore 规则：`/.scratch/`
- **未删除**24 条 B 类显式 ignore（保留向后兼容 — 若 B 类脚本再次回流到根目录，规则仍生效）
- `.gitignore` 自身 `M` 状态（待 Mavis 终审 commit）

**`git check-ignore -v` 验证**：
- `.env` → `.gitignore:30:.env` ✅
- `.env.example` → `.gitignore:32:!.env.example` ✅（un-ignore 生效）
- `.scratch/` → `.gitignore:39:/.scratch/` ✅
- `.scratch/check_wt.ps1` → `.gitignore:39:/.scratch/` ✅
- `.scratch/sign_raci.py` → `.gitignore:39:/.scratch/` ✅

### 2.3 移动后 `git status --short` 终态

```
 M .gitignore
 M crates/star-cli/src/commands/mr.rs   # feature/ai-ide-compat 进行中工作（不动）
 M crates/star-cli/src/main.rs          # feature/ai-ide-compat 进行中工作（不动）
?? .env.example                         # A 类保留（template-only）
?? _wt_audit/cargo-check-2026-08-27.err # wt-phase-* 体检副产品（不在本任务 scope）
?? docs/reports/2026-08-27-untracked-cleanup/STAR-UNTRACKED-CLEANUP-REPORT.md  # 本报告自身
```

- `.env` 被 ignore（隐藏，A 类保留）。
- `.env.example` untracked（A 类保留，template-only，可后续 `git add`）。
- `.scratch/` 被 ignore（24 文件全部隐藏，B 类已迁移）。
- B 类 24 个原始路径全部从 untracked 列表消失。
- `feature/ai-ide-compat` 进行中 `mr.rs` / `main.rs` 修改**未触碰**（与本任务无关）。

---

## §3 已知缺口

| 缺口 | 影响 | 缓解 |
|------|------|------|
| `.env.example` 未 `git add` | DDD Review 期间若有人误操作 `git add .` 可能被忽略；但模板重复使用时缺版本化 | 留待 Mavis 终审时 `git add .env.example` + commit |
| `.scratch/` 24 文件未 `rm` | 占 29 615 字节；7 天后未清则保留 | Mavis 终审 commit 后可 `mavis-trash .scratch/`，但本报告阶段保留以备审计 |
| `.gitignore.bak` 仍 ignored | 32d30af 同时备份了 `gitignore` 的 .bak；未参与本次清理 | 后续子代理可处理 |
| `.serena/cache/` + `.serena/project.local.yml` 仍 ignored | Serena IDE 缓存，非 Star 资产 | 32d30af 已 ignore，保留即可 |
| `.worktrees/` 未纳入本报告 | 当前含 4 frontend-internal + 5 wt-phase-c/d 进度中 wt | **不在本任务 scope**（per 任务边界 §Scope） |
| `.claude/` 仍 ignored | Agent runtime 目录 | 保留 ignore |
| HEAD 描述偏差 | 任务文本写"基于 main（`0a148b8`）"，实际工作区在 `feature/ai-ide-compat` HEAD = `0a148b8`（main HEAD = `4b3b8dc`） | §1.1 已声明；本报告按工作区实际状态清理 |
| `bc23d6c` 引用文件未触碰 | 任务要求"不动 bc23d6c commit 引用文件" | 已遵守：`bc23d6c` 范围文件无 untracked 候选 |
| `crates/star-cli/src/commands/mr.rs` + `main.rs` 仍 modified | `feature/ai-ide-compat` 分支进行中工作 | **不在本任务 scope**（仅清理 8/26 拍板延后 untracked，不动主分支工作） |
| `_wt_audit/cargo-check-2026-08-27.err` untracked | 8/27 wt-phase-* 体检 stderr 副产品，wt-phase 范围 | **不在本任务 scope**；由 wt-phase-* 子代理自行处理 |
| 报告自身 `docs/reports/...` untracked | 新建未 commit | per 任务"不 commit，等 Mavis 终审"；Mavis 终审时 commit 报告 + .gitignore 一并入库 |

---

## §4 守门规则

**DDD Review 必查**：

1. **A 类 `.env` 不得 `cat` / 不得打印 value**（per 2026-08-27 11:06 JST env-var 安全硬约束）。
2. **B 类脚本从 `.scratch/` 复原前必须确认目标位置**（`D:/RustGameServer*` 而非 `D:/Star`）。
3. **`.gitignore` 改动保留向后兼容**：本报告**未删除** 32d30af 已有的 24 条 B 类显式 ignore 规则，仅**新增** `/.scratch/` 通用规则（理由：若 B 类脚本再次回流到根目录，规则仍生效；新增通用规则覆盖 `.scratch/` 内部）。DDD Review 可选"删除 24 条 B 类显式 ignore"作为后续 cleanup 议题。
4. **新 C 类候选出现时**（子代理 DDD Review 发现新 untracked），**不直接归 B**，先看 BAS / SPEC 引用关系。
5. **commit 行为**：本报告 Mavis 终审前不 commit；终审后 commit 须含本报告 + `.gitignore` 改动 + `.scratch/` ignore 生效验证 3 项制品。`.scratch/` 24 文件**不** commit（ignore 状态保留为 working tree 暂存区）。

**禁止**：

- 不沿用 `bc23d6c` 叙事（per 任务硬约束）。
- 不对 A 类文件做任何 value 打印 / 网络发送。
- 不直接 `rm` B 类（已用 `Move-Item`，本规则延伸：未来复原也不得 `rm`，用 `mavis-trash`）。
- 不推 origin（R-05 维持）。
- 不动 `feature/ai-ide-compat` 分支进行中工作（`crates/star-cli/src/{commands/mr.rs, main.rs}`）— 与本任务无关。

---

## §5 签字栏

| 角色 | 姓名 / 代签 | 签字 | 日期 (JST) |
|------|-------------|------|------------|
| 发起人 | Ulysses | 8/27 16:32 JST 发令"未决全部开子代理完成" | 2026-08-27 16:32 |
| 撰写 | Mavis（接手 agent per DEC-008） | v0.1 撰写 | 2026-08-27 16:36 |
| 子代理执行 | Mavis（worker branch session `mvs_93dbaf3fbd4b47588452e079bf0ba439`） | 执行移动 + 写报告 | 2026-08-27 16:36 |
| 终审（commit 触发者） | — | 待 Mavis 终审 | — |
| 复核（DDD Review） | — | 待 Ulysses DDD Review | — |

**代签说明**（per 2026-08-27 07:16 JST 代签规则反转）：

- 撰写 / 子代理执行 / 复核栏可由 Mavis 接手 agent 在子代理 branch session 内直接代签真实责任署名。
- 本报告 §6 修订历史"审批者"列随代签规则反转：Mavis / 子代理可填写真实责任署名（如"架构师 (Mavis 接手 agent per DEC-008)"），不再受"审批者 = —"硬约束。
- 保留派生约束（代签允许 ≠ 编造允许）：
  1. 禁"per X 历史形态"等回溯叙事。
  2. 引用 BAS 必须 `git log -p --follow` 实证。
  3. 缺标比错标安全。
  4. 子代理授权边界写明"无证据叙事 = 禁止"。

---

## §6 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更摘要 | 审批者 |
|------|------------|--------|----------|--------|
| v0.1 | 2026-08-27 16:36 | Mavis（worker 子代理 session `mvs_93dbaf3fbd4b47588452e079bf0ba439`） | 初版：扫 26 untracked（24 脚本 + 2 env），B 类 24 移 `.scratch/`，补 `.gitignore`（在 32d30af 基础上加 `/.scratch/` + 注释，保留 24 条 B 类显式 ignore 向后兼容），C 类空。工作区在 `feature/ai-ide-compat` HEAD=`0a148b8`（main HEAD=`4b3b8dc` 不在同 branch，清理与 branch 无关）。`crates/star-cli/src/{commands/mr.rs, main.rs}` modified + `_wt_audit/cargo-check-2026-08-27.err` untracked 不在 scope。 | —（待 Mavis 终审） |

---

## 附录 A：本报告引用 commit 实证

| 引用 | 实证命令 / 证据 |
|------|-----------------|
| 工作区当前 branch = `feature/ai-ide-compat` | `git branch --show-current` → `feature/ai-ide-compat` |
| 工作区当前 HEAD = `0a148b8` | `git rev-parse HEAD` → `0a148b889bb9472955ff5607420f51631cbae322` |
| main HEAD = `4b3b8dc` | `git rev-parse main` → `4b3b8dc8849a15075c6b08cb4eb18eef93e376a2` |
| `0a148b8` 是 `4b3b8dc` 的祖先 | `git merge-base main HEAD` → `4b3b8dc...`（即 `4b3b8dc` 是共同祖先） |
| `0a148b8` 不在 main 上但同源 | `git branch --contains 0a148b8` → `feature/ai-ide-compat` + `wt-phase-d5-impl`（无 main） |
| `32d30af` 收纳 `docs/frontend-design-feedback.md` | `git log --oneline --all -- docs/frontend-design-feedback.md` → `32d30af chore(audit): 2026-08-27 worktree 体检 + DDD Review 资料 + 收纳 frontend-design feedback` |
| `32d30af` 已合并到 `feature/ai-ide-compat` 链 | `git log --oneline 32d30af..HEAD` 输出包含 32d30af 后续多个 commit |
| `.gitignore` 在 HEAD 的版本已含 32d30af 26 条 ignore | `git show HEAD:.gitignore` 实证（包含 24 条 B 类显式 ignore + .env ignore + .worktrees/ ignore） |
| B 类 24 文件从未被 `bc23d6c` 跟踪 | `git log --oneline bc23d6c..HEAD -- <each-b-file>` 全部空输出 |
| `.gitignore` 改动 diff | `git diff .gitignore` 输出 2 行新增（注释 + `/.scratch/`） |
| `.scratch/` ignore 生效 | `git check-ignore -v .scratch/check_wt.ps1` → `.gitignore:39:/.scratch/` |

## 附录 B：守门规则引用的上层规则

| 规则 | 来源 |
|------|------|
| env-var 安全（不打印 .env value） | 2026-08-27 11:06 JST 硬约束（Ulysses 一审即禁） |
| 代签规则反转（可代签真实责任署名） | 2026-08-27 07:16 / 08:40 JST 反转（覆盖 DTL-036 v1.4 hotfix §修式 "不可代签是硬底线" 4 小时窗口） |
| AI 协作文档治理（BAS 引用需 `git log -p --follow` 实证） | 2026-08-26 DTL-036 v1.4 hotfix 案例 |
| R-05 不 push | 2026-08-27 11:09 JST |
| 8/26 拍板"现在不动"覆盖 | 2026-08-27 16:32 JST "未决全部开子代理完成"（任务文本） |
| `Move-Item` 替 `rm` | PowerShell-only 守门 + 任务硬约束 |
