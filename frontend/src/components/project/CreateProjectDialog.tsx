"use client";

// =====================================================================
// CreateProjectDialog — 项目新建对话框 (per 2026-10-01 12:43 JST 用户发令)
//   - 醒目位置入口 (ProjectsClient header 的 [+ 新建项目] 主按钮)
//   - 字段全有默认值, 默认 Kanban 模板 = "ipa-vmodel" (IPA V字モデル 12 工程)
//   - 提交后调 store.addProject, 自动派生 id + key + 默认 6 列状态槽
// =====================================================================

import { useMemo, useState } from "react";
import { useStore } from "@/lib/store";
import type { Project, Uuid } from "@/types/ids";
import { X, FolderPlus, ChevronDown, ChevronRight, Sparkles } from "lucide-react";

export interface CreateProjectDialogProps {
  open: boolean;
  onClose: () => void;
  /** 提交后回调 (返回新创建的项目) */
  onCreated?: (project: Project) => void;
}

// ---- 默认值 (per IPA V字モデル 9 工程 + テスト 4 サブ → 12 段階) ----
const DEFAULT_PHASE_COLORS = ["#a78bfa", "#818cf8", "#22d3ee", "#34d399", "#fbbf24", "#f59e0b", "#f97316", "#ef4444", "#ec4899", "#f43f5e", "#a3e635", "#94a3b8"];
const DEFAULT_PHASES = [
  { id: "P1",   num: "01",   label: "超上流工程", ja: "超上流", side: "left"   as const },
  { id: "P2",   num: "02",   label: "要件定義",   ja: "要件定義", side: "left"   as const },
  { id: "P3",   num: "03",   label: "基本設計",   ja: "基本設計", side: "left"   as const },
  { id: "P4",   num: "04",   label: "詳細設計",   ja: "詳細設計", side: "left"   as const },
  { id: "P5",   num: "05",   label: "実装",       ja: "実装",     side: "bottom" as const },
  { id: "P6.1", num: "06.1", label: "単体試験",   ja: "単体試験", side: "right"  as const },
  { id: "P6.2", num: "06.2", label: "結合試験",   ja: "結合試験", side: "right"  as const },
  { id: "P6.3", num: "06.3", label: "システム試験", ja: "システム試験", side: "right" as const },
  { id: "P6.4", num: "06.4", label: "受入試験",   ja: "受入試験", side: "right"  as const },
  { id: "P7",   num: "07",   label: "移行・リリース", ja: "移行・リリース", side: "right" as const },
  { id: "P8",   num: "08",   label: "運用・保守", ja: "運用・保守", side: "right"  as const },
  { id: "P9",   num: "09",   label: "終結",       ja: "終結",     side: "right"  as const },
];

const KANBAN_TEMPLATES = [
  {
    id: "ipa-vmodel",
    label: "IPA V字モデル (推奨)",
    description: "12 工程 × 5 状態槽. V字左辺 (上流→詳細) → V字底辺 (実装) → V字右辺 (テスト→運用→終結). 日本 IPA V字モデル 9 段階 + テスト 4 サブ工程.",
    recommended: true,
    defaultPhases: 12,
  },
  {
    id: "jira-simplex",
    label: "Jira シンプル",
    description: "5 状態カラム (To Do / 進行中 / レビュー / ブロック / 完了) 単層. シンプルタスク管理向け.",
    recommended: false,
    defaultPhases: 0,
  },
  {
    id: "sprint-flat",
    label: "スプリント平面",
    description: "Sprint 単位の平面ボード. バックログ + To Do + 進行中 + 完了 のみ. Scrum 単発向け.",
    recommended: false,
    defaultPhases: 0,
  },
] as const;

const VISIBILITY_OPTIONS: Array<{ id: Project["visibility"]; label: string; description: string }> = [
  { id: "private", label: "Private", description: "オーナー + 明示メンバー" },
  { id: "internal", label: "Internal", description: "テナント内全員" },
  { id: "public", label: "Public", description: "リンクを知る全員" },
];

const LOCALE_OPTIONS: Array<{ id: "ja" | "zh-CN" | "en"; label: string; flag: string }> = [
  { id: "ja", label: "日本語", flag: "🇯🇵" },
  { id: "zh-CN", label: "中文", flag: "🇨🇳" },
  { id: "en", label: "English", flag: "🇺🇸" },
];

export function CreateProjectDialog({ open, onClose, onCreated }: CreateProjectDialogProps) {
  const identities = useStore((s) => s.identities);
  const addProject = useStore((s) => s.addProject);
  const tenantId = useStore((s) => s.tenantId);
  // 默认 owner = identities[0]
  const defaultOwner = identities[0]?.id as Uuid | undefined;

  // 表单状态 (全部有默认值)
  const [name, setName] = useState("新規プロジェクト / New Project");
  const [key, setKey] = useState("");
  const [description, setDescription] = useState("");
  const [visibility, setVisibility] = useState<Project["visibility"]>("private");
  const [ownerId, setOwnerId] = useState<Uuid | "">(defaultOwner ?? "");
  const [memberCount, setMemberCount] = useState(5);
  const [kanbanTemplate, setKanbanTemplate] = useState<typeof KANBAN_TEMPLATES[number]["id"]>("ipa-vmodel");
  const [initialPhaseCount, setInitialPhaseCount] = useState(12);
  const [locale, setLocale] = useState<"ja" | "zh-CN" | "en">("ja");
  const [iconColor, setIconColor] = useState<string>(DEFAULT_PHASE_COLORS[0]);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showAdvanced, setShowAdvanced] = useState(false);

  const enabledPhases = useMemo(
    () => DEFAULT_PHASES.slice(0, Math.max(1, Math.min(initialPhaseCount, DEFAULT_PHASES.length))),
    [initialPhaseCount],
  );

  if (!open) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      setError("プロジェクト名は必須です");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const project = addProject({
        name: name.trim(),
        key: key.trim() || undefined,
        visibility,
        owner_id: ownerId || undefined,
        member_count: memberCount,
        kanbanTemplate,
        initialPhaseCount,
        defaultLocale: locale,
        description: description.trim() || undefined,
        iconColor,
      });
      onCreated?.(project);
      // 重置表单
      setName("新規プロジェクト / New Project");
      setKey("");
      setDescription("");
      setVisibility("private");
      setOwnerId(defaultOwner ?? "");
      setMemberCount(5);
      setKanbanTemplate("ipa-vmodel");
      setInitialPhaseCount(12);
      setLocale("ja");
      setIconColor(DEFAULT_PHASE_COLORS[0]);
      setShowAdvanced(false);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : "作成失敗");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 grid place-items-center bg-black/50 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="create-project-title"
      data-testid="create-project-dialog"
    >
      <form
        onSubmit={handleSubmit}
        className="card relative w-[min(720px,92vw)] max-h-[90vh] overflow-y-auto p-6 shadow-2xl"
        data-testid="create-project-form"
      >
        <button
          type="button"
          aria-label="閉じる"
          onClick={onClose}
          className="absolute right-4 top-4 text-ink-mute hover:text-ink"
        >
          <X size={16} />
        </button>

        <header className="mb-4 flex items-center gap-2">
          <FolderPlus className="text-accent" size={20} />
          <h2 id="create-project-title" className="text-lg font-semibold">新規プロジェクト作成</h2>
          <span className="rounded border border-line px-2 py-0.5 text-[10px] text-ink-mute font-mono">tenant: {tenantId ?? "—"}</span>
        </header>

        {error && (
          <div role="alert" className="mb-3 rounded border border-err/40 bg-err/10 p-2 text-xs text-err">
            {error}
          </div>
        )}

        {/* ---- 基本情報 ---- */}
        <section className="space-y-3">
          <div className="grid gap-3 sm:grid-cols-[1fr_140px]">
            <label className="block">
              <span className="mb-1 block text-xs font-semibold">プロジェクト名 <span className="text-err">*</span></span>
              <input
                type="text"
                required
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="input w-full"
                placeholder="My Awesome Project"
                data-testid="create-project-name"
              />
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-semibold">キー (大文字 2-3 文字)</span>
              <input
                type="text"
                value={key}
                onChange={(e) => setKey(e.target.value.toUpperCase())}
                className="input w-full font-mono uppercase"
                placeholder="自動派生"
                maxLength={4}
                data-testid="create-project-key"
              />
              <span className="mt-0.5 block text-[10px] text-ink-mute">空白時は名から自動派生</span>
            </label>
          </div>

          <label className="block">
            <span className="mb-1 block text-xs font-semibold">説明</span>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="input w-full"
              rows={2}
              placeholder="(オプション) プロジェクト概要・目標・スコープ"
              data-testid="create-project-description"
            />
          </label>

          <div className="grid gap-3 sm:grid-cols-2">
            <label className="block">
              <span className="mb-1 block text-xs font-semibold">可視性</span>
              <div className="flex gap-1">
                {VISIBILITY_OPTIONS.map((v) => (
                  <button
                    key={v.id}
                    type="button"
                    onClick={() => setVisibility(v.id)}
                    data-testid={`create-project-visibility-${v.id}`}
                    className={`flex-1 rounded border px-2 py-1.5 text-xs font-medium transition ${
                      visibility === v.id
                        ? "border-accent/60 bg-accent/15 text-accent"
                        : "border-line bg-bg-soft/40 text-ink-dim hover:border-accent/40 hover:text-ink"
                    }`}
                    title={v.description}
                  >
                    {v.label}
                  </button>
                ))}
              </div>
            </label>

            <label className="block">
              <span className="mb-1 block text-xs font-semibold">言語</span>
              <div className="flex gap-1">
                {LOCALE_OPTIONS.map((l) => (
                  <button
                    key={l.id}
                    type="button"
                    onClick={() => setLocale(l.id)}
                    data-testid={`create-project-locale-${l.id}`}
                    className={`flex-1 rounded border px-2 py-1.5 text-xs font-medium transition ${
                      locale === l.id
                        ? "border-accent/60 bg-accent/15 text-accent"
                        : "border-line bg-bg-soft/40 text-ink-dim hover:border-accent/40 hover:text-ink"
                    }`}
                  >
                    {l.flag} {l.label}
                  </button>
                ))}
              </div>
            </label>
          </div>

          <div className="grid gap-3 sm:grid-cols-2">
            <label className="block">
              <span className="mb-1 block text-xs font-semibold">オーナー</span>
              <select
                className="input w-full"
                value={ownerId}
                onChange={(e) => setOwnerId(e.target.value as Uuid)}
                data-testid="create-project-owner"
              >
                <option value="">未選択 (自分)</option>
                {identities.slice(0, 12).map((u) => (
                  <option key={u.id} value={u.id}>{u.display_name} ({u.id})</option>
                ))}
              </select>
            </label>
            <label className="block">
              <span className="mb-1 flex items-center justify-between text-xs font-semibold">
                <span>初期メンバー数</span>
                <span className="font-mono text-accent">{memberCount}</span>
              </span>
              <input
                type="range"
                min={1}
                max={20}
                value={memberCount}
                onChange={(e) => setMemberCount(Number(e.target.value))}
                className="w-full"
                data-testid="create-project-members"
              />
            </label>
          </div>
        </section>

        {/* ---- Kanban テンプレート (主選) ---- */}
        <section className="mt-5">
          <h3 className="mb-2 flex items-center gap-1 text-xs font-semibold">
            <Sparkles size={12} className="text-accent" /> Kanban テンプレート
            <span className="text-[10px] text-ink-mute">(デフォルト: IPA V字モデル 12 工程)</span>
          </h3>
          <div className="space-y-1.5">
            {KANBAN_TEMPLATES.map((tpl) => (
              <button
                key={tpl.id}
                type="button"
                onClick={() => {
                  setKanbanTemplate(tpl.id);
                  if (tpl.id === "ipa-vmodel") setInitialPhaseCount(12);
                  if (tpl.id === "jira-simplex") setInitialPhaseCount(0);
                  if (tpl.id === "sprint-flat") setInitialPhaseCount(0);
                }}
                data-testid={`create-project-template-${tpl.id}`}
                className={`block w-full rounded border p-2.5 text-left transition ${
                  kanbanTemplate === tpl.id
                    ? "border-accent/60 bg-accent/10"
                    : "border-line bg-bg-soft/40 hover:border-accent/40"
                }`}
              >
                <div className="flex items-center justify-between gap-2">
                  <span className="flex items-center gap-1.5 text-xs font-semibold">
                    {tpl.label}
                    {tpl.recommended && (
                      <span className="rounded bg-accent/20 px-1.5 py-0 text-[9px] font-mono text-accent">推奨</span>
                    )}
                  </span>
                  <span className={`size-3 rounded-full border ${kanbanTemplate === tpl.id ? "border-accent bg-accent" : "border-line"}`} />
                </div>
                <p className="mt-1 text-[10px] leading-relaxed text-ink-dim">{tpl.description}</p>
              </button>
            ))}
          </div>

          {/* IPA プレビュー (初期段階数スライダ) */}
          {kanbanTemplate === "ipa-vmodel" && (
            <div className="mt-3 rounded border border-line bg-bg-soft/40 p-2.5">
              <div className="mb-2 flex items-center justify-between">
                <span className="text-xs font-semibold">初期工程数</span>
                <span className="font-mono text-xs text-accent">{initialPhaseCount} / 12</span>
              </div>
              <input
                type="range"
                min={1}
                max={12}
                value={initialPhaseCount}
                onChange={(e) => setInitialPhaseCount(Number(e.target.value))}
                className="w-full"
                data-testid="create-project-phase-count"
              />
              <div className="mt-2 flex flex-wrap gap-1">
                {enabledPhases.map((phase, i) => (
                  <span
                    key={phase.id}
                    className="rounded border px-2 py-0.5 text-[9px] font-mono"
                    style={{
                      borderColor: DEFAULT_PHASE_COLORS[i % DEFAULT_PHASE_COLORS.length],
                      color: DEFAULT_PHASE_COLORS[i % DEFAULT_PHASE_COLORS.length],
                    }}
                  >
                    {phase.num} {phase.label}
                  </span>
                ))}
              </div>
            </div>
          )}
        </section>

        {/* ---- 詳細 (折りたたみ) ---- */}
        <section className="mt-4">
          <button
            type="button"
            onClick={() => setShowAdvanced(!showAdvanced)}
            className="flex items-center gap-1 text-xs font-medium text-ink-mute hover:text-ink"
            data-testid="create-project-advanced-toggle"
          >
            {showAdvanced ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
            詳細設定
          </button>
          {showAdvanced && (
            <div className="mt-2 space-y-2 rounded border border-line bg-bg-soft/40 p-2.5">
              <label className="block">
                <span className="mb-1 block text-xs font-semibold">アイコン色</span>
                <div className="flex gap-1.5">
                  {DEFAULT_PHASE_COLORS.map((c) => (
                    <button
                      key={c}
                      type="button"
                      onClick={() => setIconColor(c)}
                      className={`size-6 rounded-full border-2 transition ${iconColor === c ? "border-ink scale-110" : "border-transparent"}`}
                      style={{ backgroundColor: c }}
                      aria-label={`色 ${c}`}
                    />
                  ))}
                </div>
              </label>
            </div>
          )}
        </section>

        {/* ---- 送信 ---- */}
        <footer className="mt-5 flex items-center justify-end gap-2 border-t border-line pt-3">
          <button
            type="button"
            onClick={onClose}
            className="btn text-xs"
            disabled={submitting}
          >
            キャンセル
          </button>
          <button
            type="submit"
            className="btn-primary flex items-center gap-1.5 text-xs"
            disabled={submitting || !name.trim()}
            data-testid="create-project-submit"
          >
            <FolderPlus size={13} />
            {submitting ? "作成中…" : "プロジェクト作成"}
          </button>
        </footer>
      </form>
    </div>
  );
}

export default CreateProjectDialog;
