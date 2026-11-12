/*
@cypher schema=1 source_sha256=f667487249fc83bfe3a9f8e0c888fe02cd2ed8d5a7af16fcbece9f001a958ff1
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
// 每个技能 = 一个可用任务卡: 专属栏位 → 可创建/打开任务卡, 关联该技能的
// 任务卡出现在技能的子任务列表 (与 kanban 共用同一套 WorkItem 内容模型)。
// =====================================================================

import { useMemo, useState } from "react";
import { FileText, Users, GitBranch, ShieldAlert, Play, Plus } from "lucide-react";
import { useStore } from "@/lib/store";
import { useTranslation } from "@/lib/i18n";
import {
  IPA_SKILL_CATALOG,
  IPA_PHASE_COLUMNS,
  type IpaSkillEntry,
} from "@/lib/ipa-skills/catalog";

export interface IpaSkillWorkflowBoardProps {
  projectId: string;
  tenantId: string;
  reporterId?: string;
  onOpenTaskCard: (workItemId: string) => void;
}

function PhaseColumn({
  phase,
  ja,
  zh,
  skills,
  selectedId,
  onSelect,
  projectId,
  tenantId,
  reporterId,
  onOpenTaskCard,
}: {
  phase: string;
  ja: string;
  zh: string;
  skills: IpaSkillEntry[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
} & Pick<IpaSkillWorkflowBoardProps, "projectId" | "tenantId" | "reporterId" | "onOpenTaskCard">) {
  return (
    <div
      data-testid={`ipa-phase-column-${phase}`}
      className="w-64 shrink-0 rounded-lg border border-line bg-[var(--cel-surface-card,#0f1422)] flex flex-col max-h-[64vh]"
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
            projectId={projectId}
            tenantId={tenantId}
            reporterId={reporterId}
            onOpenTaskCard={onOpenTaskCard}
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
  projectId,
  tenantId,
  reporterId,
  onOpenTaskCard,
}: {
  skill: IpaSkillEntry;
  selected: boolean;
  onSelect: (id: string | null) => void;
} & Pick<IpaSkillWorkflowBoardProps, "projectId" | "tenantId" | "reporterId" | "onOpenTaskCard">) {
  const { t, tx } = useTranslation();
  const addWorkItem = useStore((s) => s.addWorkItem);
  const workItems = useStore((s) => s.workItems);
  // 该技能的子任务 = 关联了此技能的 WorkItem (与 kanban 同一套内容模型)
  const linkedTasks = workItems.filter((w) => w.ipa_skill_ids?.includes(skill.id));

  const createTaskCard = () => {
    const id = addWorkItem({
      tenant_id: tenantId,
      project_id: projectId,
      title: skill.titleJa,
      kind: "task",
      status: "todo",
      priority: "p2",
      reporter_id: reporterId ?? "usr-001",
      labels: ["ipa-skill", skill.phase],
      ipa_skill_ids: [skill.id],
    });
    onOpenTaskCard(id);
  };

  return (
    <div
      data-testid={`ipa-skill-slot-${skill.id}`}
      className={`rounded border transition-colors ${
        selected ? "border-cyan-400/80 bg-cyan-400/10" : "border-line/60 bg-bg-soft/30"
      }`}
    >
      <button
        data-testid={`ipa-skill-slot-toggle-${skill.id}`}
        onClick={() => onSelect(selected ? null : skill.id)}
        title={skill.purpose}
        className="w-full text-left px-2 pt-1.5 pb-1"
      >
        <div className="flex items-center gap-1.5">
          <span className="font-mono text-[9px] text-cyan-400/90 shrink-0">{skill.id}</span>
          {skill.hasHumanGate && (
            <span title={t.ipaSkills.humanGate} className="text-[9px] text-amber-400 shrink-0">
              <ShieldAlert size={10} className="inline" />
            </span>
          )}
          <span className="text-[10px] text-ink truncate flex-1">{skill.titleJa}</span>
          {linkedTasks.length > 0 && (
            <span
              data-testid={`ipa-skill-subtask-count-${skill.id}`}
              className="text-[9px] font-mono text-emerald-300 border border-emerald-400/40 rounded px-1 shrink-0"
            >
              {tx(t.ipaSkills.subtaskCount, { count: linkedTasks.length })}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2 mt-0.5 text-[8px] font-mono text-ink-mute">
          <span className="text-emerald-400/90">in{skill.inputs.length}</span>
          <span className="text-sky-400/90">out{skill.outputs.length}</span>
          <span className="truncate flex-1">{skill.mode}</span>
        </div>
      </button>

      {selected && (
        <div className="px-2 pb-2 space-y-1.5">
          <div className="text-[9px] text-ink-dim leading-snug pt-1 border-t border-line/40">
            {skill.purpose || "—"}
          </div>
          <div className="text-[9px] font-mono text-ink-mute">
            {t.ipaSkills.inputs}: {skill.inputs.length > 0 ? skill.inputs.join(", ") : "—"}
          </div>
          <div className="text-[9px] font-mono text-ink-mute">
            {t.ipaSkills.outputs}: {skill.outputs.length > 0 ? skill.outputs.join(", ") : "—"}
          </div>
          <button
            data-testid={`ipa-skill-create-task-${skill.id}`}
            onClick={createTaskCard}
            className="btn text-[10px] w-full py-1"
          >
            <Plus size={10} /> {t.ipaSkills.useAsTaskCard}
          </button>
          {linkedTasks.length > 0 && (
            <div className="space-y-1" data-testid={`ipa-skill-subtasks-${skill.id}`}>
              <div className="text-[9px] font-mono text-ink-mute">{t.ipaSkills.linkedSubtasks}:</div>
              {linkedTasks.map((w) => (
                <button
                  key={w.id}
                  data-testid={`ipa-skill-subtask-${skill.id}-${w.id}`}
                  onClick={() => onOpenTaskCard(w.id)}
                  className="w-full text-left text-[9px] font-mono px-2 py-1 rounded border border-line/50 bg-bg-soft/40 hover:border-cyan-400/60 flex items-center gap-1.5"
                >
                  <Play size={8} className="text-cyan-400 shrink-0" />
                  <span className="text-ink truncate flex-1">{w.key}</span>
                  <span className="text-ink-dim truncate max-w-[55%]">{w.title}</span>
                </button>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export function IpaSkillWorkflowBoard({
  projectId,
  tenantId,
  reporterId,
  onOpenTaskCard,
}: IpaSkillWorkflowBoardProps) {
  const { t, tx } = useTranslation();
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
  const workItems = useStore((s) => s.workItems);
  const linkedWorkItems = workItems.filter((w) => (w.ipa_skill_ids?.length ?? 0) > 0);

  return (
    <div data-testid="ipa-skill-workflow-board" className="space-y-3">
      <div className="flex items-center gap-2 flex-wrap text-[10px] font-mono text-ink-mute">
        <span className="px-2 py-0.5 rounded border border-line bg-bg-soft/40">
          {tx(t.ipaSkills.totalSkills, { count: IPA_SKILL_CATALOG.length })}
        </span>
        <span className="px-2 py-0.5 rounded border border-line bg-bg-soft/40">
          {tx(t.ipaSkills.phaseColumns, { count: IPA_PHASE_COLUMNS.length })}
        </span>
        <span className="px-2 py-0.5 rounded border border-amber-400/40 bg-amber-400/10 text-amber-300">
          {tx(t.ipaSkills.humanGated, { count: humanGated })}
        </span>
        <span className="px-2 py-0.5 rounded border border-cyan-400/40 bg-cyan-400/10 text-cyan-300">
          {tx(t.ipaSkills.linkedTaskCards, { count: linkedWorkItems.length })}
        </span>
        <span className="flex items-center gap-1">
          <FileText size={10} /> {t.ipaSkills.everySkillIsTaskCard}
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
            projectId={projectId}
            tenantId={tenantId}
            reporterId={reporterId}
            onOpenTaskCard={onOpenTaskCard}
          />
        ))}
      </div>

      <div className="text-[10px] font-mono text-ink-mute flex items-center gap-2">
        <GitBranch size={10} />
        {t.ipaSkills.hierarchyNote}
        <Users size={10} className="ml-2" />
        {t.ipaSkills.contractSource}
      </div>
    </div>
  );
}
