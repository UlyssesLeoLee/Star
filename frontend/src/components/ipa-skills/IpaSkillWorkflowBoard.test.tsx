/*
@cypher schema=1 source_sha256=a23690f59b90a193ce70a09104de17ad34955b5b16d063ae2396792b4ea499fd
MERGE (self:File {path:"frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.test.tsx"})
MERGE (allSkills:Symbol {id:"frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.test.tsx::renders_all_catalog_skill_slots",kind:"test"})
MERGE (phases:Symbol {id:"frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.test.tsx::renders_phase_columns_with_counts",kind:"test"})
MERGE (createTask:Symbol {id:"frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.test.tsx::skill_creates_task_card_linked",kind:"test"})
MERGE (self)-[:DEFINES]->(allSkills)
MERGE (self)-[:DEFINES]->(phases)
MERGE (self)-[:DEFINES]->(createTask)
@endcypher
*/
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { IpaSkillWorkflowBoard } from "./IpaSkillWorkflowBoard";
import { IPA_SKILL_CATALOG, IPA_PHASE_COLUMNS } from "@/lib/ipa-skills/catalog";
import { useStore } from "@/lib/store";

const PROPS = {
  projectId: "00000000-0000-0000-0000-000000000001",
  tenantId: "00000000-0000-0000-0000-000000000001",
  reporterId: "usr-001",
  onOpenTaskCard: () => {},
};

describe("IpaSkillWorkflowBoard — 项目界面 IPA 技能工作流拆分", () => {
  beforeEach(() => {
    useStore.setState((s) => ({
      workItems: s.workItems.filter((w) => !(w.ipa_skill_ids?.length)),
    }));
  });

  it("渲染 16 个 WBS 工作流列", () => {
    render(<IpaSkillWorkflowBoard {...PROPS} />);
    expect(screen.getByTestId("ipa-phase-board")).toBeInTheDocument();
    for (const col of IPA_PHASE_COLUMNS) {
      expect(screen.getByTestId(`ipa-phase-column-${col.phase}`)).toBeInTheDocument();
    }
  });

  it("每个技能都有专属栏位 (catalog 全量 237)", () => {
    render(<IpaSkillWorkflowBoard {...PROPS} />);
    let total = 0;
    for (const skill of IPA_SKILL_CATALOG) {
      const slot = screen.queryByTestId(`ipa-skill-slot-${skill.id}`);
      if (slot) total += 1;
    }
    expect(total).toBe(IPA_SKILL_CATALOG.length);
  });

  it("技能可作为任务卡使用: 创建 WorkItem 并绑定技能 (同一套内容模型)", () => {
    const first = IPA_SKILL_CATALOG[0];
    let openedId = "";
    render(
      <IpaSkillWorkflowBoard {...PROPS} onOpenTaskCard={(id) => (openedId = id)} />,
    );
    fireEvent.click(screen.getByTestId(`ipa-skill-slot-toggle-${first.id}`));
    fireEvent.click(screen.getByTestId(`ipa-skill-create-task-${first.id}`));

    expect(openedId).toBeTruthy();
    const created = useStore.getState().workItems.find((w) => w.id === openedId);
    expect(created?.ipa_skill_ids).toContain(first.id);
    expect(created?.title).toBe(first.titleJa);
    expect(screen.getByTestId(`ipa-skill-subtask-count-${first.id}`)).toBeInTheDocument();
  });

  it("关联技能的任务卡出现在该技能的子任务列表", () => {
    const skill = IPA_SKILL_CATALOG[0];
    useStore.getState().addWorkItem({
      tenant_id: PROPS.tenantId,
      project_id: PROPS.projectId,
      title: "LinkedTask",
      kind: "task",
      status: "todo",
      priority: "p2",
      reporter_id: "usr-001",
      ipa_skill_ids: [skill.id],
    });
    render(<IpaSkillWorkflowBoard {...PROPS} />);
    fireEvent.click(screen.getByTestId(`ipa-skill-slot-toggle-${skill.id}`));
    const subtasks = screen.getByTestId(`ipa-skill-subtasks-${skill.id}`);
    expect(subtasks.textContent).toContain("LinkedTask");
  });
});
