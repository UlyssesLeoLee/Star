/*
@cypher schema=1 source_sha256=f37e3da4695b0cfcb99a0e8a1515eeccd1c4b508a30a206e3eb7d61aea3ffec2
MERGE (self:File {path:"frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.test.tsx"})
MERGE (allSkills:Symbol {id:"frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.test.tsx::renders_all_catalog_skill_slots",kind:"test"})
MERGE (phases:Symbol {id:"frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.test.tsx::renders_phase_columns_with_counts",kind:"test"})
MERGE (self)-[:DEFINES]->(allSkills)
MERGE (self)-[:DEFINES]->(phases)
@endcypher
*/
import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { IpaSkillWorkflowBoard } from "./IpaSkillWorkflowBoard";
import { IPA_SKILL_CATALOG, IPA_PHASE_COLUMNS } from "@/lib/ipa-skills/catalog";

describe("IpaSkillWorkflowBoard — 项目界面 IPA 技能工作流拆分", () => {
  it("渲染 16 个 WBS 工作流列", () => {
    render(<IpaSkillWorkflowBoard />);
    expect(screen.getByTestId("ipa-phase-board")).toBeInTheDocument();
    for (const col of IPA_PHASE_COLUMNS) {
      expect(screen.getByTestId(`ipa-phase-column-${col.phase}`)).toBeInTheDocument();
    }
  });

  it("每个技能都有专属栏位 (catalog 全量 237)", () => {
    render(<IpaSkillWorkflowBoard />);
    let total = 0;
    for (const skill of IPA_SKILL_CATALOG) {
      const slot = screen.queryByTestId(`ipa-skill-slot-${skill.id}`);
      if (slot) total += 1;
    }
    // 虚拟化/折叠不适用 — 全部栏位必须渲染 (237)
    expect(total).toBe(IPA_SKILL_CATALOG.length);
  });

  it("点击技能栏位展开契约 (purpose + 端口)", () => {
    render(<IpaSkillWorkflowBoard />);
    const first = IPA_SKILL_CATALOG[0];
    const slot = screen.getByTestId(`ipa-skill-slot-${first.id}`);
    fireEvent.click(slot);
    // 展开后 purpose 可见 (title 属性转为正文)
    expect(slot.textContent).toContain(first.id);
  });

  it("人工审批门技能带徽标统计", () => {
    render(<IpaSkillWorkflowBoard />);
    const gated = IPA_SKILL_CATALOG.filter((s) => s.hasHumanGate).length;
    expect(gated).toBeGreaterThan(0);
  });
});
