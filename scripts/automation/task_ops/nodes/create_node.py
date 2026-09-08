# scripts/automation/task_ops/nodes/create_node.py
# M-N8 create_node (TMO-08, per ADR-0049 + docs/architecture/2026-08-26-upgrade/adr/0049-task-card-auto-worktree-agent.md)
#
# 职责 (核心功能, per 2026-09-09 04:57 JST 用户拍板):
#   - 任务卡创建时检测有效 agent → 自动建 worktree + dispatch SA-XX sub-agent
#   - 1 WorkItem → 1 Worktree (per 拍板 + INV-WT-07 + ARG §14.11 worktree-as-agent-anchor)
#   - worktree Ready 后自动 AgentSession 接管 (per 拍板: 自动接管, 任务卡 5s 内 in_progress)
#   - 守门 #13 a: L0 唯一创建入口, 跟 TMO 7 节点 (M-N1..M-N7) 同层
#   - 守门 #13 d: worktree 状态变更 = Transaction (append-only audit, 物理删除禁止)
#   - 守门 #19: Python 化
#   - 守门 #22: 不污染 main 编译, mock shell wrapper 走 subprocess
#
# 输入: CreateTaskRequest (per protocols.py)
# 输出: CreateTaskResponse (task_id + worktree_id + agent_session_id + status 流)
#
# 阻塞 (per HANDOFF-ST-001):
#   - H2 stage 2 5 domain 改造未闭, 跨域字段 (workspace_ids / tenant_policy_id) 暂走 stub
#   - 真实 Git worktree CLI 走 _mock_git_worktree.py, 不进 main 编译链 (守门 #22)
#   - Agent Runtime 深度整合 (per ADR-0045) 是后续 P3-C, 本期走内存版 SubAgentPool
#
# 派生 (per 守门 #21 docs 同步):
#   - docs/automation-design.md §4 任务卡表 + scripts/automation/registry.md 已同步
#   - commit message 含 brief 路径 (守门 #20) + 脚本相对路径 (守门 #19)

from __future__ import annotations

import asyncio
import logging
import subprocess
import time
import uuid
from pathlib import Path
from typing import Optional

logger = logging.getLogger("task_ops.create")

# Master RLS 必携字段 (per 守门 #13 c) — 跟 metadata_node.py 对齐
REQUIRED_RLS_FIELDS = ("tenant_id",)

# 17 WorktreeStatus 子集 (per crates/domain-worktree/src/lib.rs v0.0.1 §状态机):
#   Created → Initializing → Ready → Assigned → AgentRunning
# 本节点职责: Created → Initializing → Ready (M-N8 出口, Assigned/AgentRunning 由后续 Orchestrator 节点触发)
WORKTREE_INIT_STATUSES = ("Created", "Initializing", "Ready")

# mock_git_worktree.py 路径 (per 守门 #22 不污染 main 编译, 走 subprocess)
_MOCK_WT_SCRIPT = Path(__file__).parent.parent.parent / "_mock_git_worktree.py"


async def create_node(
    sub_agent_pool,  # SubAgentPool 实例 (L0 唯一入口 per 守门 #13 a)
    worktree_registry,  # WorktreeRegistry 实例 (PoC 内存版, 真实 DB 走 G-WT-01)
    metadata_registry,  # MetadataRegistry (per M-N7) — 创建时同时落 metadata
    request: dict,
) -> dict:
    """M-N8 create_node (per TMO-08 + ADR-0049 §3)

    最小可行骨架 (per V2-6 5 子代理 + Mavis 跨域协调模式):
      1. 验证必携字段 (tenant_id + assignee_type=agent + agent_id 有效)
      2. 验证 agent 有效 (在 sub_agent_pool 注册表中存在)
      3. checkpoint stash (preserved, per 守门 #13 d Transaction append-only)
      4. 创建 Worktree (1 WorkItem → 1 Worktree, 走 mock git worktree CLI 守门 #22)
      5. worktree Created → Initializing → Ready (L0 状态机推进)
      6. 自动 dispatch SA-XX sub-agent (per 拍板: 自动接管, 5s 内任务卡 in_progress)
      7. worktree Assigned → AgentRunning (L0 状态机推进)
      8. 返 task_id + worktree_id + agent_session_id + checkpoint_id (Transaction 追加)

    Args:
        sub_agent_pool: L0 SubAgentPool 实例 (跨 9 SA Archetype 共享)
        worktree_registry: WorktreeRegistry 实例 (PoC 内存版, 真实 DB 拍板后接入)
        metadata_registry: MetadataRegistry 实例 (per M-N7, 创建时同时落 metadata)
        request: CreateTaskRequest dict, 字段:
            - operation: 固定 "create"
            - title (str, required): 任务卡标题
            - kind (str, optional): task / bug / story / epic, default "task"
            - priority (str, optional): p0/p1/p2/p3, default "p2"
            - sprint_id (str, optional): sprint 关联 (per frontend sprint 视图)
            - project_id (str, optional): project 关联 (per frontend board 视图)
            - tenant_id (str, required): Master RLS 必携 (per 守门 #13 c)
            - workspace_ids (list[str], optional): 多 workspace 隔离 (per H2-EXT 5 domain)
            - assignee_id (str, required): 当前是 agent id (per 拍板: 仅 agent 触发, 非 agent 走普通 store.createWorkItem)
            - assignee_type (str, required): 固定 "agent"
            - sa_type (str, optional): SA-01..SA-10 (per ADR-0046 §6.1), default "SA-01" (task 通用型)
            - actor_session_id (str, optional): 创建者 session_id (per L0 chat bar)

    Returns:
        CreateTaskResponse dict, 字段:
            - operation: "create"
            - task_id (str): 新建 WorkItem id (Work 类型, 短 TTL 30d per 守门 #13 a)
            - worktree_id (str): 新建 Worktree id (per INV-WT-08 必带 tenant_id)
            - agent_session_id (str): 新建 AgentSession id (per INV-WT-07 1 Worktree → 0..N AgentSession)
            - worktree_status (str): "AgentRunning" (出口, 已自动接管)
            - task_status (str): "in_progress" (出口, 已自动接管)
            - checkpoint_id (str): stash checkpoint id (Transaction append-only)
            - created_at_ms (int): 创建时间戳 ms
            - actor_session_id (str): 创建者 session_id
    """
    operation = request.get("operation")
    if operation != "create":
        raise ValueError(f"create_node: operation must be 'create', got {operation!r}")

    # 1. 验证必携字段
    title = request.get("title")
    if not title:
        raise ValueError("create_node: title required (non-empty)")

    tenant_id = request.get("tenant_id")
    if not tenant_id:
        raise ValueError("create_node: tenant_id required (Master RLS 必携 per 守门 #13 c)")

    assignee_id = request.get("assignee_id")
    assignee_type = request.get("assignee_type")
    if not assignee_id or assignee_type != "agent":
        # 守门 (per 拍板): M-N8 仅处理 agent 任务, 人类任务走普通 store.createWorkItem
        raise ValueError(
            f"create_node: assignee_type must be 'agent' for auto-worktree, "
            f"got {assignee_type!r} (human tasks use store.createWorkItem directly)"
        )

    actor_session_id = request.get("actor_session_id")
    project_id = request.get("project_id", "default")
    sprint_id = request.get("sprint_id")
    sa_type = request.get("sa_type", "SA-01")  # SA-01..SA-10, default 通用 task 型
    kind = request.get("kind", "task")
    priority = request.get("priority", "p2")
    workspace_ids = request.get("workspace_ids", [])

    # 2. 验证 agent 有效 (在 sub_agent_pool 注册表中存在)
    if not sub_agent_pool.has_agent(assignee_id):
        raise ValueError(
            f"create_node: agent_id {assignee_id!r} not found in SubAgentPool "
            f"(per 拍板: '存在有效agent' 强约束, 无效 agent 不触发 M-N8)"
        )

    # 3. checkpoint stash (preserved, per 守门 #13 d Transaction append-only)
    # PoC: 内存版 stash, 真实 DDL 推 G-WT-01
    stash_id = f"stash-{uuid.uuid4().hex[:12]}"
    logger.info(
        f"M-N8 create_node: stash checkpoint {stash_id} "
        f"tenant={tenant_id} project={project_id} sprint={sprint_id}"
    )

    # 4. 创建 WorkItem (Work 类型, 短 TTL 30d per 守门 #13 a)
    task_id = f"task-{uuid.uuid4().hex[:12]}"
    work_item = {
        "id": task_id,
        "title": title,
        "kind": kind,
        "priority": priority,
        "status": "todo",  # 初始 todo, 6. 步后 auto → in_progress
        "tenant_id": tenant_id,
        "project_id": project_id,
        "sprint_id": sprint_id,
        "workspace_ids": workspace_ids,
        "assignee_id": assignee_id,
        "assignee_type": assignee_type,
        "sa_type": sa_type,
        "worktree_id": None,  # 5. 步后填
        "agent_session_id": None,  # 6. 步后填
        "created_at_ms": int(time.time() * 1000),
        "actor_session_id": actor_session_id,
    }

    # 5. 创建 Worktree (1 WorkItem → 1 Worktree, 走 mock git worktree CLI 守门 #22)
    worktree_id = f"wt-{uuid.uuid4().hex[:12]}"
    worktree = {
        "id": worktree_id,
        "task_id": task_id,
        "tenant_id": tenant_id,  # INV-WT-08 必带
        "workspace_ids": workspace_ids,
        "status": "Created",  # 17 状态机起点
        "agent_id": assignee_id,
        "sa_type": sa_type,
        "created_at_ms": int(time.time() * 1000),
    }
    worktree_registry.add(worktree)

    # mock git worktree CLI (守门 #22 不污染 main 编译, 走 subprocess 异步)
    asyncio.create_task(
        _invoke_mock_git_worktree(
            worktree_id=worktree_id,
            task_id=task_id,
            tenant_id=tenant_id,
        )
    )

    # worktree 状态机推进: Created → Initializing → Ready (PoC 同步推进, 真实走 L0 状态机)
    worktree["status"] = "Initializing"
    worktree_registry.update(worktree_id, {"status": "Initializing"})
    await asyncio.sleep(0.05)  # 模拟初始化延迟
    worktree["status"] = "Ready"
    worktree_registry.update(worktree_id, {"status": "Ready"})

    # 6. 自动 dispatch SA-XX sub-agent (per 拍板: 自动接管, 5s 内任务卡 in_progress)
    agent_session_id = await sub_agent_pool.dispatch(
        agent_id=assignee_id,
        sa_type=sa_type,
        task_id=task_id,
        worktree_id=worktree_id,
        tenant_id=tenant_id,
    )

    # worktree 状态机推进: Ready → Assigned → AgentRunning
    worktree["status"] = "Assigned"
    worktree_registry.update(worktree_id, {"status": "Assigned"})
    await asyncio.sleep(0.02)  # 模拟 dispatch 延迟
    worktree["status"] = "AgentRunning"
    worktree_registry.update(worktree_id, {"status": "AgentRunning"})

    # 7. 任务卡状态 auto → in_progress (per 拍板: 5s 内可见到 in_progress)
    work_item["status"] = "in_progress"
    work_item["worktree_id"] = worktree_id
    work_item["agent_session_id"] = agent_session_id
    work_item["in_progress_at_ms"] = int(time.time() * 1000)

    # 8. 同时落 metadata (per M-N7 协同, 创建即有 metadata)
    # PoC: TaskMetadataRepository.upsert_metadata 签名 (per task_metadata_repo.py:102)
    #   upsert_metadata(task_id, tenant_id, workspace_id, metadata, actor_session_id)
    # workspace_id 取 workspace_ids[0] (per H2-EXT 5 domain 阻塞, 暂取第 1 个 workspace)
    if metadata_registry is not None and hasattr(metadata_registry, "upsert_metadata"):
        primary_workspace_id = workspace_ids[0] if workspace_ids else "default"
        try:
            metadata_registry.upsert_metadata(
                task_id=task_id,
                tenant_id=tenant_id,
                workspace_id=primary_workspace_id,
                metadata={
                    "name": title,
                    "labels": [],
                    "notes": None,
                    "priority": priority,
                    "sa_type": sa_type,
                    "agent_id": assignee_id,
                    "worktree_id": worktree_id,
                    "agent_session_id": agent_session_id,
                    "created_via": "M-N8",  # 审计: 明确由 TMO 创建
                },
                actor_session_id=actor_session_id,
            )
        except Exception as exc:  # noqa: BLE001
            # PoC: metadata 落档失败不阻塞主流程 (核心功能是 worktree + agent 接管)
            logger.warning(
                f"M-N8 create_node: metadata upsert failed {exc!r} "
                f"(PoC 非阻塞, 真实 metadata 落档推 G-WT-01)"
            )

    # PoC: 真实 TaskRepository 走 G-WT-01 DB 接入, 本期内存版
    logger.info(
        f"M-N8 create_node: COMPLETE task={task_id} wt={worktree_id} "
        f"agent_session={agent_session_id} sa={sa_type} tenant={tenant_id}"
    )

    return {
        "operation": "create",
        "task_id": task_id,
        "worktree_id": worktree_id,
        "agent_session_id": agent_session_id,
        "worktree_status": "AgentRunning",
        "task_status": "in_progress",
        "checkpoint_id": stash_id,
        "created_at_ms": work_item["created_at_ms"],
        "in_progress_at_ms": work_item["in_progress_at_ms"],
        "actor_session_id": actor_session_id,
    }


async def _invoke_mock_git_worktree(
    worktree_id: str,
    task_id: str,
    tenant_id: str,
) -> None:
    """调用 _mock_git_worktree.py 模拟 git worktree add (守门 #22 不污染 main 编译)

    真实 Git worktree CLI 调用走 subprocess, 失败不阻塞主流程 (PoC 异步 fire-and-forget)
    真实 Git 集成推 G-WT-02 (后续 P0-1/H2 阻塞解除后启动)
    """
    if not _MOCK_WT_SCRIPT.exists():
        logger.warning(
            f"M-N8 create_node: mock_git_worktree.py not found at {_MOCK_WT_SCRIPT}, "
            f"skipping (per 守门 #22 不污染 main 编译)"
        )
        return

    try:
        proc = await asyncio.create_subprocess_exec(
            "python",
            str(_MOCK_WT_SCRIPT),
            "add",
            f"--worktree-id={worktree_id}",
            f"--task-id={task_id}",
            f"--tenant-id={tenant_id}",
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        stdout, stderr = await proc.communicate()
        if proc.returncode != 0:
            logger.warning(
                f"M-N8 create_node: mock_git_worktree exit {proc.returncode}, "
                f"stderr={stderr.decode()[:200]!r} (PoC 非阻塞, 真实 Git 推 G-WT-02)"
            )
        else:
            logger.info(f"M-N8 create_node: mock_git_worktree OK wt={worktree_id}")
    except Exception as exc:  # noqa: BLE001
        # PoC: 真实 Git 集成阻塞解除前, 失败不阻塞主流程
        logger.warning(f"M-N8 create_node: mock_git_worktree exception {exc!r} (PoC 非阻塞)")
