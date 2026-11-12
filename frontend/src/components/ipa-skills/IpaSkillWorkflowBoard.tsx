/*
@cypher schema=1 source_sha256=32119228309a28faf80e92c616c17b2f7bef53cfe4f7e35153dc78a5a3495335
MERGE (self:File {path:"frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.tsx"})
MERGE (board:Function {name:"IpaSkillWorkflowBoard", language:"typescript", visibility:"public"})
MERGE (slot:Function {name:"IpaSkillSlot", language:"typescript", visibility:"private"})
MERGE (column:Function {name:"IpaPhaseColumnView", language:"typescript", visibility:"private"})
MERGE (self)-[:DEFINES]->(board)
MERGE (self)-[:DEFINES]->(slot)
MERGE (self)-[:DEFINES]->(column)
@endcypher
*/
"use client";

// =====================================================================
// IpaSkillWorkflowBoard — IPA 技能工作流拆分视图 (per 2026-11-11 用户发令)
// 项目界面按 D:/IPA-Atomic-Skills 目录拆分: 每个技能在其专属日本 IPA
// 开发流程工作流列 (WBS P1~P9 + P6.1~P6.4 + 横断 PX 组) 中有专属栏位。
// 数据源: frontend/src/lib/ipa-skills/catalog.ts (生成自 IPA-Atomic-Skills)。
// =====================================================================

import { useMemo, useState } from "react";
import { FileText, Users, GitBranch, ShieldAlert } from "lucide-react";
import {
  IPA_SKILL_CATALOG,
  IPA_PHASE_COLUMNS,
  type IpaSkillEntry,
} from "@/lib/ipa-skills/catalog";

function PhaseColumn({
  phase,
  ja,
  zh,
  skills,
  selectedId,
  onSelect,
}: {
  phase: string;
  ja: string;
  zh: string;
  skills: IpaSkillEntry[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
}) {
  return (
    <div
      data-testid={`ipa-phase-column-${phase}`}
      className="w-56 shrink-0 rounded-lg border border-line bg-[var(--cel-surface-card,#0f1422)] flex flex-col max-h-[62vh]"
    >
      <div className="px-3 py-2 border-b border-line shrink-0">
        <div className="flex items-center gap-1.5">
          <span className="font-mono text-xs font-bold text-cyan-300">{phase}</span>
          <span className="text-[11px] text-ink font-semibold truncate">{ja}</span>
        </div>
        <div className="text-[9px] font-mono text-ink-mute truncate">
          {zh} · {skills.length} skills
        </div>
      </div>
      <div className="flex-1 overflow-y-auto p-2 space-y-1.5">
        {skills.map((skill) => (
          <IpaSkillSlot
            key={skill.id}
            skill={skill}
            selected={selectedId === skill.id}
            onSelect={onSelect}
          />
        ))}
      </div>
    </div>
  );
}

function IpaSkillSlot({
  skill,
  selected,
  onSelect,
}: {
  skill: IpaSkillEntry;
  selected: boolean;
  onSelect: (id: string | null) => void;
}) {
  return (
    <button
      data-testid={`ipa-skill-slot-${skill.id}`}
      onClick={() => onSelect(selected ? null : skill.id)}
      title={skill.purpose}
      className={`w-full text-left rounded border px-2 py-1.5 transition-colors ${
        selected
          ? "border-cyan-400/80 bg-cyan-400/10"
          : "border-line/60 bg-bg-soft/30 hover:border-cyan-400/50"
      }`}
    >
      <div className="flex items-center gap-1.5">
        <span className="font-mono text-[9px] text-cyan-400/90 shrink-0">{skill.id}</span>
        {skill.hasHumanGate && (
          <span title="人工审批门" className="text-[9px] text-amber-400 shrink-0">
            <ShieldAlert size={10} className="inline" />
          </span>
        )}
        <span className="text-[10px] text-ink truncate flex-1">{skill.titleJa}</span>
      </div>
      <div className="flex items-center gap-2 mt-0.5 text-[8px] font-mono text-ink-mute">
        <span className="text-emerald-400/90">in{skill.inputs.length}</span>
        <span className="text-sky-400/90">out{skill.outputs.length}</span>
        <span className="truncate flex-1">{skill.mode}</span>
      </div>
      {selected && (
        <div className="mt-1.5 pt-1.5 border-t border-line/40 text-[9px] text-ink-dim space-y-1">
          <div className="leading-snug">{skill.purpose || "—"}</div>
          <div className="font-mono text-ink-mute">
            输入: {skill.inputs.length > 0 ? skill.inputs.join(", ") : "—"}
          </div>
          <div className="font-mono text-ink-mute">
            输出: {skill.outputs.length > 0 ? skill.outputs.join(", ") : "—"}
          </div>
        </div>
      )}
    </button>
  );
}

export function IpaSkillWorkflowBoard() {
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const byPhase = useMemo(() => {
    const map = new Map<string, IpaSkillEntry[]>();
    for (const skill of IPA_SKILL_CATALOG) {
      const list = map.get(skill.phase) ?? [];
      list.push(skill);
      map.set(skill.phase, list);
    }
    return map;
  }, []);

  const humanGated = IPA_SKILL_CATALOG.filter((s) => s.hasHumanGate).length;

  return (
    <div data-testid="ipa-skill-workflow-board" className="space-y-3">
      <div className="flex items-center gap-2 flex-wrap text-[10px] font-mono text-ink-mute">
        <span className="px-2 py-0.5 rounded border border-line bg-bg-soft/40">
          总技能 {IPA_SKILL_CATALOG.length}
        </span>
        <span className="px-2 py-0.5 rounded border border-line bg-bg-soft/40">
          工作流列 {IPA_PHASE_COLUMNS.length}
        </span>
        <span className="px-2 py-0.5 rounded border border-amber-400/40 bg-amber-400/10 text-amber-300">
          人工审批门 {humanGated}
        </span>
        <span className="flex items-center gap-1">
          <FileText size={10} /> 每技能专属栏位 · 点击展开契约
        </span>
      </div>

      <div className="flex gap-3 overflow-x-auto pb-3" data-testid="ipa-phase-board">
        {IPA_PHASE_COLUMNS.map((col) => (
          <PhaseColumn
            key={col.phase}
            phase={col.phase}
            ja={col.ja}
            zh={col.zh}
            skills={byPhase.get(col.phase) ?? []}
            selectedId={selectedId}
            onSelect={setSelectedId}
          />
        ))}
      </div>

      <div className="text-[10px] font-mono text-ink-mute flex items-center gap-2">
        <GitBranch size={10} />
        层级: 日本 IPA 开发流程 (超上游→要件→基本→详细→实现→测试→移行→运维→终结) · 横断 XMG/XRV/XTT/XDL · per WBS 设计
        <Users size={10} className="ml-2" />
        技能契约来自 ../IPA-Atomic-Skills workflow-contract.json (required_inputs → 输入端口, required_outputs → 输出端口)
      </div>
    </div>
  );
}
