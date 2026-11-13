/*
@cypher schema=1 source_sha256=c966f8c60ba63d2214c41dca9f9aab31dd68267d2c698cde129dcecf5ff53fb3
MERGE (self:File {path:"frontend/src/components/CanvasView.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/CanvasView.test.tsx::CanvasViewPorts",kind:"suite"})
MERGE (view:File {path:"frontend/src/components/CanvasView.tsx"})
MERGE (self)-[:IMPORTS]->(view)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(view)
@endcypher
*/
import { describe, expect, it } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import type { ReactNode } from "react";
import { CanvasView } from "@/components/CanvasView";
import { I18nProvider } from "@/lib/i18n";
import type { Canvas, CanvasConnector, CanvasElement } from "@/types/ids";
import { useStore } from "@/lib/store";

const baseCanvas: Pick<Canvas, "id" | "viewport" | "frames"> = {
  id: "canvas-001",
  viewport: { x: 0, y: 0, zoom: 1 },
  frames: [],
};

function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

describe("CanvasView 自由画笔 (per 2026-10-01 OOB 恢复丢失的画笔功能)", () => {
  it("A. 默认 toolbar 含 brush 按钮", () => {
    renderWithI18n(<CanvasView canvas={baseCanvas} elements={[]} connectors={[]} onCreateElement={() => undefined} />);
    expect(screen.getByTestId("canvas-tool-brush")).toBeTruthy();
  });

  it("B. 未选 brush 时不显示 palette / size 控件", () => {
    renderWithI18n(<CanvasView canvas={baseCanvas} elements={[]} connectors={[]} onCreateElement={() => undefined} />);
    expect(screen.queryByTestId("canvas-brush-palette")).toBeNull();
    expect(screen.queryByTestId("canvas-brush-size")).toBeNull();
  });

  it("C. path element 渲染时含 <path> 元素, stroke 跟随 brush_color", () => {
    const elements: CanvasElement[] = [
      {
        id: "path-001",
        canvas_id: "canvas-001",
        kind: "path",
        x: 10,
        y: 20,
        width: 100,
        height: 50,
        rotation: 0,
        z_index: 1,
        content: {
          path_data: "M 0 0 L 50 25 L 100 50",
          brush_size: 6,
          brush_color: "#79c0ff",
        },
        locked: false,
        hidden: false,
        created_by: "usr-001",
        created_at: "2026-10-01T00:00:00Z",
        updated_at: "2026-10-01T00:00:00Z",
      },
    ];
    renderWithI18n(<CanvasView canvas={baseCanvas} elements={elements} connectors={[]} onCreateElement={() => undefined} />);
    const wrapper = screen.getByTestId("canvas-element-path-001");
    expect(wrapper).toBeTruthy();
    const pathEl = wrapper.querySelector("path");
    expect(pathEl).toBeTruthy();
    expect(pathEl?.getAttribute("d")).toBe("M 0 0 L 50 25 L 100 50");
    expect(pathEl?.getAttribute("stroke")).toBe("#79c0ff");
  });

  it("D. readOnly 模式下 brush 按钮不显示", () => {
    renderWithI18n(<CanvasView canvas={baseCanvas} elements={[]} connectors={[]} readOnly onCreateElement={() => undefined} />);
    expect(screen.queryByTestId("canvas-tool-brush")).toBeNull();
  });
});

describe("CanvasView 输入/输出端点 (per WBS §23.35)", () => {
  const nodeEl = (id: string, x: number, y: number): CanvasElement => ({
    id,
    canvas_id: "canvas-001",
    kind: "worktree_node",
    x,
    y,
    width: 160,
    height: 100,
    rotation: 0,
    z_index: 1,
    content: { worktree_id: "wt-001" },
    locked: false,
    hidden: false,
    created_by: "usr-001",
    created_at: "2026-10-01T00:00:00Z",
    updated_at: "2026-10-01T00:00:00Z",
  });

  it("E. 节点元素渲染 input/output 端点, 无连接且未选中时不可见", () => {
    renderWithI18n(<CanvasView canvas={baseCanvas} elements={[nodeEl("n1", 0, 0)]} connectors={[]} onCreateElement={() => undefined} />);
    const input = screen.getByTestId("canvas-port-n1-input");
    const output = screen.getByTestId("canvas-port-n1-output");
    expect(input).toBeTruthy();
    expect(output).toBeTruthy();
    // 未选中且无连接 → 隐藏 (opacity 0)
    const group = screen.getByTestId("canvas-ports-n1");
    expect(group.getAttribute("style")).toContain("opacity: 0");
  });

  it("F. 有连接的元素端口可见 (吸附状态常显)", () => {
    const connectors: CanvasConnector[] = [{
      id: "cn-t1",
      canvas_id: "canvas-001",
      kind: "agent_handoff",
      from_element_id: "n1",
      to_element_id: "n2",
      from_port: "output",
      to_port: "input",
      routing: "curved",
      arrow_start: false,
      arrow_end: true,
      color: "#3fb950",
      width: 2,
    }];
    renderWithI18n(
      <CanvasView
        canvas={baseCanvas}
        elements={[nodeEl("n1", 0, 0), nodeEl("n2", 300, 0)]}
        connectors={connectors}
        onCreateElement={() => undefined}
      />,
    );
    expect(screen.getByTestId("canvas-ports-n1").getAttribute("style")).toContain("opacity: 1");
    expect(screen.getByTestId("canvas-ports-n2").getAttribute("style")).toContain("opacity: 1");
    // 端点吸附: 连接线从 n1 右侧输出端出发, 到 n2 左侧输入端
    const path = document.querySelector("path[d*='C']");
    expect(path).toBeTruthy();
    const d = path?.getAttribute("d") ?? "";
    // 起点 = n1 输出端 x=160 (n1.x+width), 终点 = n2 输入端 x=300 (n2.x)
    expect(d.startsWith("M 160 50")).toBe(true);
    expect(d.includes("300 50")).toBe(true);
  });

  it("G. 输出端点 mousedown 开始拖线 → 显示预览虚线; 输入端点 mouseup 创建连接", () => {
    renderWithI18n(
      <CanvasView
        canvas={baseCanvas}
        elements={[nodeEl("n1", 0, 0), nodeEl("n2", 300, 0)]}
        connectors={[]}
        onCreateElement={() => undefined}
      />,
    );
    // 端口在未选中时 pointerEvents=none, jsdom fireEvent 仍可直接调用 handler
    fireEvent.mouseDown(screen.getByTestId("canvas-port-n1-output"), { clientX: 160, clientY: 50 });
    // 拖线预览出现
    expect(screen.getByTestId("canvas-port-drag-preview")).toBeTruthy();
    // 释放到 n2 输入端点 → 创建 connection (from_port=output, to_port=input)
    fireEvent.mouseUp(screen.getByTestId("canvas-port-n2-input"), { clientX: 300, clientY:50 });
    const created = useStore.getState().canvasConnectors.find(
      (c) => c.from_element_id === "n1" && c.to_element_id === "n2",
    );
    expect(created).toBeTruthy();
    expect(created?.from_port).toBe("output");
    expect(created?.to_port).toBe("input");
    // 预览消失
    expect(screen.queryByTestId("canvas-port-drag-preview")).toBeNull();
  });

  it("H. 拖线释放到空白 SVG → 取消, 不创建连接", () => {
    renderWithI18n(
      <CanvasView
        canvas={baseCanvas}
        elements={[nodeEl("n1", 0, 0)]}
        connectors={[]}
        onCreateElement={() => undefined}
      />,
    );
    const before = useStore.getState().canvasConnectors.length;
    fireEvent.mouseDown(screen.getByTestId("canvas-port-n1-output"), { clientX: 160, clientY: 50 });
    fireEvent.mouseUp(screen.getByTestId("canvas-svg"), { clientX: 500, clientY: 300 });
    expect(useStore.getState().canvasConnectors.length).toBe(before);
    expect(screen.queryByTestId("canvas-port-drag-preview")).toBeNull();
  });
});
