/*
@cypher schema=1 source_sha256=a60f144eb03bfd853f26a0a0a0c63f16dd9021ad0707ae73e2e17017a8e8f987
MERGE (self:File {path:"frontend/src/components/canvas/CanvasScopeSelector.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/canvas/CanvasScopeSelector.test.tsx::CanvasScopeSelector 三级范围选择器",kind:"suite"})
MERGE (comp:Symbol {id:"frontend/src/components/canvas/CanvasScopeSelector.tsx::CanvasScopeSelector",kind:"component"})
MERGE (navStore:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:IMPORTS]->(comp)
MERGE (self)-[:IMPORTS]->(navStore)
MERGE (suite)-[:TESTS]->(comp)
@endcypher
*/

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { CanvasScopeSelector, UNCLASSIFIED_LANE_ID } from "./CanvasScopeSelector";
import { useNavStore } from "@/lib/nav/navStore";
import { useStore } from "@/lib/store";
import { lanes as seedLanes, projects as seedProjects, worktrees as seedWorktrees } from "@/lib/seed";

const PROJECT_ID = "prj-physis";
const STAR_GATE = "prj-stargate";
const LANE_CORE = "lane-001";
const LANE_PHYSIS3D = "lane-002";

const resetNav = () =>
  useNavStore.setState({
    selectedProjectId: "",
    selectedLaneId: "",
    selectedWorktreeId: "",
    selectedBranch: "",
  });

const optionValues = (el: HTMLElement) =>
  Array.from((el as HTMLSelectElement).options).map((o) => o.value);

describe("CanvasScopeSelector 三级范围选择器", () => {
  beforeEach(() => {
    // 主 store 初始为诚实空状态 (无 seed 兜底) — 注入测试所需实体
    window.localStorage.clear();
    useStore.setState({
      projects: seedProjects,
      lanes: seedLanes,
      worktrees: seedWorktrees,
    });
    resetNav();
  });
  afterEach(() => resetNav());

  it("1. 渲染三级选择器 + 徽标; navStore 为空时 Lane/Worktree 禁用, 徽标=未指定范围", () => {
    render(<CanvasScopeSelector />);
    expect(screen.getByTestId("canvas-scope-project")).toBeTruthy();
    expect((screen.getByTestId("canvas-scope-lane") as HTMLSelectElement).disabled).toBe(true);
    expect((screen.getByTestId("canvas-scope-worktree") as HTMLSelectElement).disabled).toBe(
      true,
    );
    expect(screen.getByTestId("canvas-scope-badge").textContent).toBe("未指定范围");
  });

  it("2. 默认值取 navStore 选中值", () => {
    useNavStore.setState({
      selectedProjectId: PROJECT_ID,
      selectedLaneId: LANE_CORE,
      selectedWorktreeId: "wt-001",
    });
    render(<CanvasScopeSelector />);
    expect((screen.getByTestId("canvas-scope-project") as HTMLSelectElement).value).toBe(
      PROJECT_ID,
    );
    expect((screen.getByTestId("canvas-scope-lane") as HTMLSelectElement).value).toBe(LANE_CORE);
    expect((screen.getByTestId("canvas-scope-worktree") as HTMLSelectElement).value).toBe(
      "wt-001",
    );
    expect(screen.getByTestId("canvas-scope-badge").textContent).toContain("wt-worktree-sm");
  });

  it("3. Project → Lane → Worktree 联动过滤 (lane-003 属于 stargate, 不混入 physis)", () => {
    render(<CanvasScopeSelector />);
    fireEvent.change(screen.getByTestId("canvas-scope-project"), {
      target: { value: PROJECT_ID },
    });
    const laneSelect = screen.getByTestId("canvas-scope-lane") as HTMLSelectElement;
    expect(optionValues(laneSelect)).toEqual(["", LANE_CORE, LANE_PHYSIS3D]);
    expect(laneSelect.disabled).toBe(false);

    fireEvent.change(laneSelect, { target: { value: LANE_CORE } });
    const wtSelect = screen.getByTestId("canvas-scope-worktree") as HTMLSelectElement;
    expect(wtSelect.disabled).toBe(false);
    const wtIds = optionValues(wtSelect).filter(Boolean).sort();
    expect(wtIds).toEqual(
      ["wt-001", "wt-002", "wt-003", "wt-004", "wt-009", "wt-010", "wt-011", "wt-012"].sort(),
    );
  });

  it("4. 选 Worktree 写回 navStore 并回调 onScopeChange", () => {
    useNavStore.setState({ selectedProjectId: PROJECT_ID, selectedLaneId: LANE_CORE });
    const onScopeChange = vi.fn();
    render(<CanvasScopeSelector onScopeChange={onScopeChange} />);
    fireEvent.change(screen.getByTestId("canvas-scope-worktree"), {
      target: { value: "wt-001" },
    });
    const nav = useNavStore.getState();
    expect(nav.selectedWorktreeId).toBe("wt-001");
    expect(nav.selectedLaneId).toBe(LANE_CORE);
    expect(nav.selectedBranch).toBe("feat/worktree-sm");
    expect(onScopeChange).toHaveBeenCalledWith({
      projectId: PROJECT_ID,
      laneId: LANE_CORE,
      worktreeId: "wt-001",
    });
  });

  it("5. 切换 Project 清空 Lane/Worktree (navStore + UI 禁用)", () => {
    useNavStore.setState({
      selectedProjectId: PROJECT_ID,
      selectedLaneId: LANE_CORE,
      selectedWorktreeId: "wt-001",
    });
    render(<CanvasScopeSelector />);
    fireEvent.change(screen.getByTestId("canvas-scope-project"), {
      target: { value: STAR_GATE },
    });
    const nav = useNavStore.getState();
    expect(nav.selectedProjectId).toBe(STAR_GATE);
    expect(nav.selectedLaneId).toBe("");
    expect(nav.selectedWorktreeId).toBe("");
    expect((screen.getByTestId("canvas-scope-worktree") as HTMLSelectElement).disabled).toBe(
      true,
    );
  });

  it("6. 未归类 worktree 归入 默认 LANE (stargate: wt-006/wt-007 无 lane_id)", () => {
    render(<CanvasScopeSelector />);
    fireEvent.change(screen.getByTestId("canvas-scope-project"), {
      target: { value: STAR_GATE },
    });
    const laneSelect = screen.getByTestId("canvas-scope-lane") as HTMLSelectElement;
    expect(optionValues(laneSelect)).toEqual(["", "lane-003", UNCLASSIFIED_LANE_ID]);
    fireEvent.change(laneSelect, { target: { value: UNCLASSIFIED_LANE_ID } });
    const wtSelect = screen.getByTestId("canvas-scope-worktree") as HTMLSelectElement;
    expect(optionValues(wtSelect).filter(Boolean).sort()).toEqual(["wt-006", "wt-007"]);
  });

  it("7. 切换 Lane 清空 Worktree 选择并回调", () => {
    useNavStore.setState({
      selectedProjectId: PROJECT_ID,
      selectedLaneId: LANE_CORE,
      selectedWorktreeId: "wt-001",
    });
    const onScopeChange = vi.fn();
    render(<CanvasScopeSelector onScopeChange={onScopeChange} />);
    fireEvent.change(screen.getByTestId("canvas-scope-lane"), {
      target: { value: LANE_PHYSIS3D },
    });
    const nav = useNavStore.getState();
    expect(nav.selectedWorktreeId).toBe("");
    expect(onScopeChange).toHaveBeenCalledWith({
      projectId: PROJECT_ID,
      laneId: LANE_PHYSIS3D,
      worktreeId: "",
    });
    expect((screen.getByTestId("canvas-scope-worktree") as HTMLSelectElement).value).toBe("");
  });

  it("8. 脏持久化值 (lane 不属于当前 project) 回退为未选择", () => {
    useNavStore.setState({
      selectedProjectId: PROJECT_ID,
      selectedLaneId: "lane-003",
      selectedWorktreeId: "wt-007",
    });
    render(<CanvasScopeSelector />);
    expect((screen.getByTestId("canvas-scope-lane") as HTMLSelectElement).value).toBe("");
    expect((screen.getByTestId("canvas-scope-worktree") as HTMLSelectElement).value).toBe("");
  });
});
