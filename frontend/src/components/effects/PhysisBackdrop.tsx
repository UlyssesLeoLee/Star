/*
@cypher schema=1 source_sha256=11c0c39fdfa656c7bdb6209ee93567cea441d1f35957e1a264fe52065c321bdb
MERGE (self:File {path:"frontend/src/components/effects/PhysisBackdrop.tsx"})
MERGE (backdrop:Symbol {id:"frontend/src/components/effects/PhysisBackdrop.tsx::PhysisBackdrop",kind:"component"})
MERGE (wasm:Symbol {id:"frontend/src/lib/physis/physis_wasm.js::PhysisWorld",kind:"wasm"})
MERGE (self)-[:DEFINES]->(backdrop)
MERGE (self)-[:IMPORTS]->(wasm)
@endcypher
*/
"use client";

// =====================================================================
// PhysisBackdrop — 3D 主题 node-edge 图背景 (per 2026-11-14 用户发令:
//   3D 主题整体重新设计, node-edge 方式呈现, 参考 GOS 项目做法)
//   GOS 图原生世界观: 一切皆节点(node)与边(edge)
//   - 节点 = Star 真实实体 (worktree/task/agent), 发光球体 + 呼吸
//   - 边   = 实体关系 (worktree→task, agent→worktree), 发光连线
//   - 物理 = Physis GVPE WASM 驱动节点运动 (力导向图感)
//   - 交互 = 鼠标吸引子 + 点击冲击波 + physis-burst 事件
// =====================================================================

import { useEffect, useRef } from "react";
import * as THREE from "three";
import { useTheme } from "next-themes";
import { useStore } from "@/lib/store";
import { readThemeRgb } from "@/lib/theme/themeTokens";

interface PhysisWorldWasm {
  new (gravity: number[]): {
    spawn(x: number, y: number, z: number, radius: number, mass: number): number;
    step(dt: number): void;
    position(idx: number): number[];
    attract(idx: number, tx: number, ty: number, tz: number, k: number): void;
    len(): number;
  };
}

// 节点角色 → 颜色 token (per 2026-11-17 主题统一: 从 html.physis CSS token 取色,
// fallback 为 physis 设计值; 不再硬编码 hex)
const ROLE_TOKEN = {
  worktree: "--ok-DEFAULT",      // #5ad2ff 量子青 (physis)
  task: "--color-secondary",     // #9c6bff 星云紫
  agent: "--cel-cyan",           // #7c8cff 量子蓝
} as const;
const ROLE_FALLBACK: Record<keyof typeof ROLE_TOKEN, [number, number, number]> = {
  worktree: [0.353, 0.824, 1.0],
  task: [0.612, 0.42, 1.0],
  agent: [0.486, 0.549, 1.0],
};

export function PhysisBackdrop() {
  const { resolvedTheme } = useTheme();
  const mountRef = useRef<HTMLDivElement>(null);
  // 实体数据 (节点来源, 每次主题激活时读一次)
  const worktrees = useStore((s) => s.worktrees);
  const workItems = useStore((s) => s.workItems);
  const agentSessions = useStore((s) => s.agentSessions);

  useEffect(() => {
    if (resolvedTheme !== "physis") return;
    const mount = mountRef.current;
    if (!mount) return;

    let raf = 0;
    let disposed = false;
    // a11y (per 2026-11-17 妥善增加): prefers-reduced-motion 时不做物理步进,
    // 只静态渲染一帧 node-edge 图 (节点仍可见, 无动画/冲击波)
    const reducedMotion = window.matchMedia?.("(prefers-reduced-motion: reduce)")?.matches ?? false;
    let world: InstanceType<PhysisWorldWasm> | null = null;
    const shocks: { x: number; y: number; age: number }[] = [];
    let mouseWorld = { x: 0, y: 0 };

    // --- 构建 node-edge 图 (GOS 图原生: 真实实体 + 关系) ---
    type Node = {
      id: string;
      role: keyof typeof ROLE_TOKEN;
      label: string;
      seed: [number, number, number];
    };
    const nodes: Node[] = [];
    const edges: [number, number][] = [];

    // worktree 节点 (hub)
    const wtIndex = new Map<string, number>();
    worktrees.slice(0, 8).forEach((wt, i) => {
      wtIndex.set(wt.id, nodes.length);
      nodes.push({
        id: wt.id,
        role: "worktree",
        label: wt.name,
        seed: [Math.cos((i / 8) * Math.PI * 2) * 6, 2 + Math.sin(i * 1.7) * 2, Math.sin((i / 8) * Math.PI * 2) * 4],
      });
    });
    // task 节点 + worktree→task 边
    workItems.slice(0, 18).forEach((wi, i) => {
      const idx = nodes.length;
      nodes.push({
        id: wi.id,
        role: "task",
        label: wi.key,
        seed: [(Math.random() - 0.5) * 16, 1 + Math.random() * 6, (Math.random() - 0.5) * 8],
      });
      if (wi.worktree_id) {
        const wt = wtIndex.get(wi.worktree_id);
        if (wt !== undefined && edges.length < 60) edges.push([wt, idx]);
      }
    });
    // agent 节点 + agent→worktree 边
    agentSessions.slice(0, 6).forEach((ag, i) => {
      const idx = nodes.length;
      nodes.push({
        id: ag.id,
        role: "agent",
        label: `ag-${ag.id.slice(0, 6)}`,
        seed: [Math.cos((i / 6) * Math.PI * 2) * 9, 4 + Math.sin(i * 2.3) * 3, -2],
      });
      const wt = wtIndex.get(ag.worktree_id ?? "");
      if (wt !== undefined && edges.length < 60) edges.push([idx, wt]);
    });

    // --- three.js 场景 ---
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(55, 1, 0.1, 120);
    camera.position.set(0, 0, 18);

    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(0x000000, 0);
    mount.appendChild(renderer.domElement);
    renderer.domElement.style.width = "100%";
    renderer.domElement.style.height = "100%";

    const rgbToHex = (rgb: readonly number[]) =>
      (Math.round(rgb[0] * 255) << 16) | (Math.round(rgb[1] * 255) << 8) | Math.round(rgb[2] * 255);
    const cursorLight = new THREE.PointLight(
      rgbToHex(readThemeRgb("--color-secondary", [0.545, 0.361, 0.965])), 0, 30);
    cursorLight.position.set(0, 0, 5);
    scene.add(cursorLight);
    scene.add(new THREE.AmbientLight(0x8899ff, 0.5));
    const dir = new THREE.DirectionalLight(0xffffff, 0.9);
    dir.position.set(5, 8, 6);
    scene.add(dir);

    // --- 节点 mesh (发光球体, 角色色) ---
    const nodeMeshes: THREE.Mesh[] = [];
    const nodeMats: THREE.MeshStandardMaterial[] = [];
    const nodeSeeds = nodes.map((n) => n.seed);
    nodes.forEach((n, i) => {
      const color = rgbToHex(readThemeRgb(ROLE_TOKEN[n.role], ROLE_FALLBACK[n.role]));
      const mat = new THREE.MeshStandardMaterial({
        color,
        emissive: color,
        emissiveIntensity: 0.5,
        roughness: 0.25,
        metalness: 0.3,
      });
      const size = n.role === "worktree" ? 0.55 : n.role === "agent" ? 0.4 : 0.3;
      const mesh = new THREE.Mesh(new THREE.SphereGeometry(size, 16, 12), mat);
      mesh.position.set(n.seed[0], n.seed[1], n.seed[2]);
      scene.add(mesh);
      nodeMeshes.push(mesh);
      nodeMats.push(mat);
    });

    // --- 边 (THREE.Line, 随节点实时重算) ---
    const edgeGeos = edges.map(() => new THREE.BufferGeometry().setFromPoints([
      new THREE.Vector3(), new THREE.Vector3(),
    ]));
    const edgeLines = edgeGeos.map((g) => {
      const line = new THREE.Line(
        g,
        new THREE.LineBasicMaterial({
          color: rgbToHex(readThemeRgb("--cel-cyan", [0.486, 0.549, 1.0])),
          transparent: true, opacity: 0.35 }),
      );
      scene.add(line);
      return line;
    });

    // --- Physis WASM ---
    (async () => {
      try {
        const mod = await import("@/lib/physis/physis_wasm.js");
        const World = mod.PhysisWorld as unknown as PhysisWorldWasm;
        const w = new World([0, -1.6, 0]);
        nodeMeshes.forEach((m) => {
          w.spawn(m.position.x, m.position.y, m.position.z, 0.5, 1.0);
        });
        world = w;

        let last = performance.now();
        const tick = () => {
          if (disposed) return;
          if (reducedMotion) { renderer.render(scene, camera); return; }
          raf = requestAnimationFrame(tick);
          const now = performance.now();
          const dt = Math.min((now - last) / 1000, 0.05);
          last = now;

          w.step(dt / 2);
          w.step(dt / 2);

          // 同步节点 + 冲击波 + 底部回推
          for (let i = 0; i < nodeMeshes.length; i++) {
            const p = w.position(i);
            const mesh = nodeMeshes[i];
            mesh.position.set(p[0], p[1], p[2]);
            // 呼吸发光
            nodeMats[i].emissiveIntensity = 0.4 + Math.sin(now * 0.003 + i) * 0.25;

            if (p[1] < -7) w.attract(i, p[0], 7.0, p[2], 1.0);
            for (const s of shocks) {
              const dx = p[0] - s.x;
              const dy = p[1] - s.y;
              const d2 = dx * dx + dy * dy;
              if (d2 < 64) {
                const d = Math.sqrt(d2) + 0.1;
                const f = (1 - s.age / 0.8) * (30 / d);
                w.attract(i, p[0] + dx * f * 2, p[1] + dy * f * 2, p[2], 0.08);
              }
            }
          }
          for (const s of shocks) s.age += dt;
          while (shocks.length > 0 && shocks[0].age > 0.8) shocks.shift();

          // 边端点跟随节点 (GOS edge 实时重连)
          for (let e = 0; e < edges.length; e++) {
            const [a, b] = edges[e];
            const pos = edgeGeos[e].attributes.position as THREE.BufferAttribute;
            pos.setXYZ(0, nodeMeshes[a].position.x, nodeMeshes[a].position.y, nodeMeshes[a].position.z);
            pos.setXYZ(1, nodeMeshes[b].position.x, nodeMeshes[b].position.y, nodeMeshes[b].position.z);
            pos.needsUpdate = true;
            edgeGeos[e].computeBoundingSphere();
          }

          cursorLight.position.set(mouseWorld.x, mouseWorld.y, 5);
          cursorLight.intensity = 1.2 + Math.sin(now * 0.004) * 0.4;

          renderer.render(scene, camera);
        };
        tick();
      } catch (e) {
        console.warn("PhysisBackdrop: wasm load failed, static render", e);
        renderer.render(scene, camera);
      }
    })();

    const screenToWorld = (e: PointerEvent) => {
      const w = window.innerWidth;
      const h = window.innerHeight;
      return {
        x: ((e.clientX / w) * 2 - 1) * 12,
        y: -((e.clientY / h) * 2 - 1) * 8.5,
      };
    };
    const onPointerMove = (e: PointerEvent) => {
      if (!world) return;
      const t = screenToWorld(e);
      mouseWorld = t;
      for (let i = 0; i < nodeMeshes.length; i += 4) {
        world.attract(i, t.x, t.y, 0, 0.1);
      }
    };
    const onPointerDown = (e: PointerEvent) => {
      if (!world) return;
      const t = screenToWorld(e);
      shocks.push({ x: t.x, y: t.y, age: 0 });
      if (shocks.length > 4) shocks.shift();
    };
    const onBurst = (e: Event) => {
      if (!world) return;
      const d = (e as CustomEvent<{ x?: number; y?: number }>).detail ?? {};
      const t = d.x !== undefined && d.y !== undefined
        ? { x: d.x, y: d.y }
        : { x: (Math.random() - 0.5) * 12, y: 2 };
      shocks.push({ x: t.x, y: t.y, age: -0.5 });
      if (shocks.length > 4) shocks.shift();
    };

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerdown", onPointerDown);
    window.addEventListener("physis-burst", onBurst);

    const onResize = () => {
      renderer.setSize(mount.clientWidth, mount.clientHeight);
      camera.aspect = mount.clientWidth / Math.max(1, mount.clientHeight);
      camera.updateProjectionMatrix();
    };
    onResize();
    window.addEventListener("resize", onResize);

    return () => {
      disposed = true;
      cancelAnimationFrame(raf);
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerdown", onPointerDown);
      window.removeEventListener("physis-burst", onBurst);
      window.removeEventListener("resize", onResize);
      scene.traverse((o) => {
        const m = o as THREE.Mesh;
        if (m.geometry) m.geometry.dispose();
        const mat = m.material as THREE.Material | undefined;
        if (mat) mat.dispose();
      });
      edgeGeos.forEach((g) => g.dispose());
      renderer.dispose();
      if (renderer.domElement.parentNode === mount) mount.removeChild(renderer.domElement);
    };
  }, [resolvedTheme, worktrees, workItems, agentSessions]);

  if (resolvedTheme !== "physis") return null;

  return (
    <div
      ref={mountRef}
      data-testid="physis-backdrop"
      aria-hidden="true"
      className="fixed inset-0 pointer-events-none"
      style={{ zIndex: 0 }}
    />
  );
}
