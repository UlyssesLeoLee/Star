import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { CanvasView } from "@/components/CanvasView";
import { I18nProvider } from "@/lib/i18n";
import type { Canvas, CanvasElement } from "@/types/ids";

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
