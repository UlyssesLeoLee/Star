/*
@cypher schema=1 source_sha256=244e237654e39d8a3bff8957a09fd965967fb1d38f8fbb750913dc4d248f549d
MERGE (self:File {path:"frontend/src/components/canvas-game/CanvasGameOverlay.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/canvas-game/CanvasGameOverlay.test.tsx::CanvasGameOverlay",kind:"suite"})
MERGE (overlay:Symbol {id:"frontend/src/components/canvas-game/CanvasGameOverlay.tsx::CanvasGameOverlay",kind:"component"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(overlay)
@endcypher
*/
// per WBS §23.36: CanvasGameOverlay 渲染测试
import { describe, it, expect, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { I18nProvider } from "@/lib/i18n";
import { CanvasGameOverlay } from "./CanvasGameOverlay";

const seeds = [
  { id: "el-wt-001", x: 400, y: 300, kind: "work_item" as const, difficulty: 1 },
  { id: "el-ag-001", x: 700, y: 300, kind: "agent" as const, difficulty: 2 },
];

function renderOverlay(enabled = true) {
  return render(
    <I18nProvider initialLanguage="zh-CN">
      <CanvasGameOverlay seeds={seeds} enabled={enabled} onExit={() => undefined} />
    </I18nProvider>,
  );
}

describe("CanvasGameOverlay (per WBS §23.36)", () => {
  afterEach(() => cleanup());

  it("A. enabled=false 时不渲染", () => {
    renderOverlay(false);
    expect(screen.queryByTestId("canvas-game-overlay")).toBeNull();
  });

  it("B. 游戏层渲染: 玩家机器人 + 敌人精灵 + HUD (HP/SP/击杀)", () => {
    renderOverlay();
    expect(screen.getByTestId("canvas-game-overlay")).toBeTruthy();
    expect(screen.getByTestId("canvas-game-svg")).toBeTruthy();
    expect(screen.getByTestId("game-robot")).toBeTruthy();
    // 2 个种子 → 2 个敌人
    const enemies = screen.getAllByTestId(/^game-enemy-/);
    expect(enemies).toHaveLength(2);
    // HUD
    expect(screen.getByTestId("game-hud-hp")).toBeTruthy();
    expect(screen.getByTestId("game-hud-sp")).toBeTruthy();
    expect(screen.getByTestId("game-hud-kills")).toBeTruthy();
    // 操作提示
    expect(screen.getByTestId("game-controls-hint")).toBeTruthy();
  });

  it("C. 退出按钮触发 onExit", () => {
    let exited = false;
    render(
      <I18nProvider initialLanguage="zh-CN">
        <CanvasGameOverlay seeds={seeds} enabled onExit={() => { exited = true; }} />
      </I18nProvider>,
    );
    fireEvent.click(screen.getByTestId("game-exit"));
    expect(exited).toBe(true);
  });
});
