/*
@cypher schema=1 source_sha256=47b6999dd9b2293e5a8a494a055e405e0a8a4047f05692a40532b972f59e7605
MERGE (self:File {path:"frontend/src/components/settings/ModDirectory.tsx"})
MERGE (modDirectory:Symbol {id:"frontend/src/components/settings/ModDirectory.tsx::ModDirectory",kind:"component"})
MERGE (modEntry:Type {id:"frontend/src/components/settings/ModDirectory.tsx::ModEntry"})
MERGE (react:ExternalService {name:"react"})
MERGE (clsx:ExternalService {name:"clsx"})
MERGE (self)-[:DEFINES]->(modDirectory)
MERGE (self)-[:DEFINES]->(modEntry)
MERGE (self)-[:IMPORTS]->(react)
MERGE (self)-[:IMPORTS]->(clsx)
@endcypher
*/
"use client";

// =====================================================================
// ModDirectory — 设置 › Mod 面板的本地模式实现 (ModPanel 3 个禁用按钮的真实替代)
// ① 浏览 Mod 目录: 本地 seed Mod 列表 + 安装 → 加入已安装列表
// ② 从文件安装: input type=file accept=.json, 解析 {name,type,abilities,version}
// ③ 用模板新建: 名称+类型 → 生成骨架对象加入已安装
// ④ 顶部诚实徽标: 本地目录预览 — 目录服务未接入 (后端接入后替换)
// ⑤ 已安装列表: 本地 useState 持有 (标注: 未持久化, 刷新即丢;
//    后端 Mod 目录/安装服务接入后可切换为 useStore 持久化)
// =====================================================================
import { useState, type ChangeEvent } from "react";
import { clsx } from "clsx";

export interface ModEntry {
  id: string;
  name: string;
  type: string;
  abilities: string[];
  version: string;
  source: "directory" | "file" | "template";
}

// 本地目录预览: Mod 目录服务后端未接入, 此处用本地常量模拟官方目录,
// 仅用于前端本地预览与交互验证。后端接入后替换为 API 拉取。
const LOCAL_DIRECTORY_MODS: ModEntry[] = [
  { id: "midnight-theme", name: "Midnight 主题", type: "主题", abilities: ["theme.apply", "ui.override"], version: "1.2.0", source: "directory" },
  { id: "kanban-stats", name: "看板统计面板", type: "面板", abilities: ["board.read", "chart.render"], version: "0.9.1", source: "directory" },
  { id: "git-guard", name: "Git 守护", type: "命令", abilities: ["git.hook", "shell.exec"], version: "2.0.0", source: "directory" },
  { id: "i18n-pack", name: "i18n 语言包", type: "资源包", abilities: ["dict.override"], version: "1.0.3", source: "directory" },
];

type FileModPayload = { name?: unknown; type?: unknown; abilities?: unknown; version?: unknown };

function parseFileModPayload(raw: unknown): Pick<ModEntry, "name" | "type" | "abilities" | "version"> | null {
  if (typeof raw !== "object" || raw === null) return null;
  const p = raw as FileModPayload;
  if (typeof p.name !== "string" || p.name.trim() === "") return null;
  if (typeof p.type !== "string" || p.type.trim() === "") return null;
  if (typeof p.version !== "string" || p.version.trim() === "") return null;
  if (!Array.isArray(p.abilities) || !p.abilities.every((a) => typeof a === "string")) return null;
  return { name: p.name, type: p.type, abilities: p.abilities, version: p.version };
}

function slugify(name: string): string {
  return name.trim().toLowerCase().replace(/\s+/g, "-").replace(/[^a-z0-9\u4e00-\u9fff-]/g, "");
}

export function ModDirectory() {
  const [installed, setInstalled] = useState<ModEntry[]>([]);
  const [fileError, setFileError] = useState("");
  const [templateName, setTemplateName] = useState("");
  const [templateType, setTemplateType] = useState("工具");

  const installMod = (mod: ModEntry) => {
    setInstalled((prev) => (prev.some((m) => m.id === mod.id) ? prev : [...prev, mod]));
  };

  const handleFileInstall = async (e: ChangeEvent<HTMLInputElement>) => {
    setFileError("");
    const file = e.target.files?.[0];
    e.target.value = ""; // 允许重复选择同一文件
    if (!file) return;
    let raw: unknown;
    try {
      raw = JSON.parse(await file.text());
    } catch {
      setFileError("解析失败: 文件不是有效的 JSON");
      return;
    }
    const parsed = parseFileModPayload(raw);
    if (!parsed) {
      setFileError("解析失败: 必须包含 name / type / abilities / version 字段");
      return;
    }
    const mod: ModEntry = { id: `file-${slugify(parsed.name)}`, ...parsed, source: "file" };
    if (installed.some((m) => m.id === mod.id)) {
      setFileError("解析失败: 该 Mod 已安装");
      return;
    }
    installMod(mod);
  };

  const handleTemplateCreate = () => {
    setFileError("");
    const name = templateName.trim();
    if (!name) return;
    const mod: ModEntry = {
      id: `tpl-${slugify(name)}`,
      name,
      type: templateType.trim() || "工具",
      // 模板骨架: 空能力列表 = 默认最小权限, 能力由 Mod 实现时自行声明
      abilities: [],
      version: "0.1.0",
      source: "template",
    };
    if (installed.some((m) => m.id === mod.id)) {
      setFileError("该 Mod 已安装");
      return;
    }
    installMod(mod);
    setTemplateName("");
  };

  return (
    <div data-testid="mod-directory" className="space-y-4">
      {/* ④ 诚实徽标: 本地预览, 目录服务未接入 */}
      <div className="flex flex-wrap items-center gap-2">
        <span data-testid="mod-dir-badge" className="rounded-full border border-warn/40 bg-warn/10 px-2.5 py-1 text-[10px] font-semibold text-warn">
          本地目录预览 — 目录服务未接入
        </span>
        <span className="text-[10px] text-ink-mute">安装/新建仅在前端本地列表生效, 不写入磁盘或运行时</span>
      </div>

      {/* ① 浏览 Mod 目录 */}
      <section>
        <h3 className="text-xs font-bold text-ink">浏览 Mod 目录</h3>
        <ul className="mt-2 grid gap-2">
          {LOCAL_DIRECTORY_MODS.map((mod) => {
            const isInstalled = installed.some((m) => m.id === mod.id);
            return (
              <li
                key={mod.id}
                data-testid={`mod-dir-item-${mod.id}`}
                className="flex items-center gap-3 rounded-xl border border-line bg-bg-card px-3.5 py-3"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="text-sm font-semibold text-ink">{mod.name}</span>
                    <span className="rounded bg-accent/10 px-1.5 py-0.5 text-[10px] text-accent">{mod.type}</span>
                    <span className="font-mono text-[10px] text-ink-mute">v{mod.version}</span>
                  </div>
                  <div className="mt-0.5 text-[11px] text-ink-dim">
                    {mod.abilities.length > 0 ? mod.abilities.join(" · ") : "无声明能力"}
                  </div>
                </div>
                <button
                  type="button"
                  data-testid={`mod-dir-install-${mod.id}`}
                  onClick={() => installMod(mod)}
                  disabled={isInstalled}
                  className={clsx(
                    "shrink-0 rounded-lg px-2.5 py-1.5 text-xs",
                    isInstalled
                      ? "cursor-default border border-line text-ink-mute"
                      : "bg-accent text-white hover:bg-accent-soft",
                  )}
                >
                  {isInstalled ? "已安装" : "安装"}
                </button>
              </li>
            );
          })}
        </ul>
      </section>

      {/* ② 从文件安装 */}
      <section className="rounded-xl border border-line bg-bg-card p-4">
        <h3 className="text-xs font-bold text-ink">从文件安装</h3>
        <p className="mt-1 text-[10px] text-ink-mute">
          选择 .mod.json 文件, 需包含 name / type / abilities / version 字段
        </p>
        <input
          type="file"
          accept=".json,application/json"
          data-testid="mod-dir-file-install"
          onChange={handleFileInstall}
          className="mt-2 block text-[11px] text-ink-dim file:mr-2 file:rounded-lg file:border file:border-line file:bg-bg-soft file:px-2.5 file:py-1 file:text-[11px] file:text-ink"
        />
        {fileError && (
          <p data-testid="mod-dir-file-error" className="mt-2 text-[11px] text-err">
            {fileError}
          </p>
        )}
      </section>

      {/* ③ 用模板新建 */}
      <section>
        <h3 className="text-xs font-bold text-ink">用模板新建 Mod</h3>
        <div className="mt-2 flex flex-wrap items-center gap-2">
          <input
            type="text"
            data-testid="mod-dir-template-name"
            value={templateName}
            onChange={(e) => setTemplateName(e.target.value)}
            placeholder="Mod 名称"
            className="rounded-lg border border-line bg-bg-card px-3 py-1.5 text-xs text-ink outline-none placeholder:text-ink-mute focus:border-accent"
          />
          <input
            type="text"
            data-testid="mod-dir-template-type"
            value={templateType}
            onChange={(e) => setTemplateType(e.target.value)}
            placeholder="类型, 如 工具"
            className="rounded-lg border border-line bg-bg-card px-3 py-1.5 text-xs text-ink outline-none placeholder:text-ink-mute focus:border-accent"
          />
          <button
            type="button"
            data-testid="mod-dir-template-new"
            onClick={handleTemplateCreate}
            disabled={templateName.trim() === ""}
            className={clsx(
              "rounded-lg px-3 py-1.5 text-xs",
              templateName.trim() === ""
                ? "cursor-not-allowed border border-line bg-bg-soft/60 text-ink-mute"
                : "bg-accent text-white hover:bg-accent-soft",
            )}
          >
            用模板新建 Mod
          </button>
        </div>
      </section>

      {/* 已安装列表 (本地 state) */}
      <section>
        <h3 className="text-xs font-bold text-ink">已安装 ({installed.length})</h3>
        {installed.length === 0 ? (
          <p data-testid="mod-dir-installed-empty" className="mt-2 text-[11px] text-ink-mute">
            暂无已安装 Mod — 从上方目录 / 文件 / 模板添加
          </p>
        ) : (
          <ul className="mt-2 grid gap-2">
            {installed.map((mod) => (
              <li
                key={mod.id}
                data-testid={`mod-dir-installed-item-${mod.id}`}
                className="flex items-center gap-3 rounded-xl border border-line bg-bg-card px-3.5 py-2.5"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="text-sm font-semibold text-ink">{mod.name}</span>
                    <span className="rounded bg-accent/10 px-1.5 py-0.5 text-[10px] text-accent">{mod.type}</span>
                    <span className="font-mono text-[10px] text-ink-mute">v{mod.version}</span>
                    <span className="text-[10px] text-ink-mute">
                      {mod.source === "directory" ? "来自目录" : mod.source === "file" ? "来自文件" : "来自模板"}
                    </span>
                  </div>
                  <div className="mt-0.5 text-[11px] text-ink-dim">
                    {mod.abilities.length > 0 ? mod.abilities.join(" · ") : "无声明能力"}
                  </div>
                </div>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
