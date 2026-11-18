/*
@cypher schema=1 source_sha256=7a649e87513bbd57ee09eb0d8b7db4e798e4f8aa85191b2196ab9b9c8968fc9b
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
//   - 节点 = Star 真实实体 (worktree/task/agent), 结构化几何体 + 呼吸
//   - 边   = 实体关系 (worktree→task, agent→worktree), 束状连线 + 流动粒子
//   - 物理 = Physis GVPE WASM 驱动节点运动 (力导向图感)
//   - 交互 = 鼠标吸引子 + 点击冲击波 + physis-burst 事件
//   - 视觉 = 层级簇群 + 深度分层 + 能量脉冲传播
// =====================================================================

import { useEffect, useRef, useMemo } from "react";
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
    const shocks: { x: number; y: number; z: number; age: number }[] = [];
    let mouseWorld = { x: 0, y: 0, z: 0 };

    // --- 构建 node-edge 图 (GOS 图原生: 真实实体 + 关系) ---
    type Node = {
      id: string;
      role: keyof typeof ROLE_TOKEN;
      label: string;
      seed: [number, number, number];
      cluster: number;  // worktree cluster index
    };
    const nodes: Node[] = [];
    const edges: { source: number; target: number; type: "tree-task" | "agent-tree" }[] = [];

    // worktree 节点 (hub clusters)
    const wtIndex = new Map<string, number>();
    const wtClusterCenters: THREE.Vector3[] = [];
    worktrees.slice(0, 6).forEach((wt, i) => {
      const angle = (i / Math.max(1, 6 - 1)) * Math.PI * 2;
      const radius = 8 + Math.sin(i * 1.3) * 1.5;
      const center = new THREE.Vector3(
        Math.cos(angle) * radius,
        1 + Math.sin(i * 0.7) * 2,
        Math.sin(angle) * radius * 0.7
      );
      wtClusterCenters.push(center);
      wtIndex.set(wt.id, nodes.length);
      nodes.push({
        id: wt.id,
        role: "worktree",
        label: wt.name,
        seed: [center.x, center.y, center.z],
        cluster: i,
      });
    });

    // task 节点 + worktree→task 边 (clustered around worktrees)
    workItems.slice(0, 20).forEach((wi, i) => {
      const clusterIdx = wi.worktree_id ? (wtIndex.get(wi.worktree_id) ?? 0) : (i % wtClusterCenters.length);
      const center = wtClusterCenters[clusterIdx] || new THREE.Vector3(0, 2, 0);
      const angle = (i / 20) * Math.PI * 2 * 3;
      const r = 2.5 + (i % 5) * 0.6;
      const idx = nodes.length;
      nodes.push({
        id: wi.id,
        role: "task",
        label: wi.key,
        seed: [
          center.x + Math.cos(angle) * r,
          center.y + 1 + Math.sin(i * 0.9) * 1.5,
          center.z + Math.sin(angle) * r,
        ],
        cluster: clusterIdx,
      });
      if (wi.worktree_id) {
        const wt = wtIndex.get(wi.worktree_id);
        if (wt !== undefined && edges.length < 80) {
          edges.push({ source: wt, target: idx, type: "tree-task" });
        }
      }
    });

    // agent 节点 + agent→worktree 边 (orbiting outer ring)
    agentSessions.slice(0, 6).forEach((ag, i) => {
      const angle = (i / Math.max(1, 6 - 1)) * Math.PI * 2;
      const radius = 12 + Math.sin(i * 1.1) * 2;
      const idx = nodes.length;
      nodes.push({
        id: ag.id,
        role: "agent",
        label: `ag-${ag.id.slice(0, 6)}`,
        seed: [
          Math.cos(angle) * radius,
          3 + Math.sin(i * 1.5) * 2,
          Math.sin(angle) * radius * 0.5,
        ],
        cluster: -1, // agents are global
      });
      const wt = wtIndex.get(ag.worktree_id ?? "");
      if (wt !== undefined && edges.length < 80) {
        edges.push({ source: idx, target: wt, type: "agent-tree" });
      }
    });

    // --- three.js 场景 ---
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(55, 1, 0.1, 150);
    camera.position.set(0, 4, 22);

    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(0x000000, 0);
    mount.appendChild(renderer.domElement);
    renderer.domElement.style.width = "100%";
    renderer.domElement.style.height = "100%";

    const rgbToHex = (rgb: readonly number[]) =>
      (Math.round(rgb[0] * 255) << 16) | (Math.round(rgb[1] * 255) << 8) | Math.round(rgb[2] * 255);

    // Lights
    const cursorLight = new THREE.PointLight(
      rgbToHex(readThemeRgb("--color-secondary", [0.545, 0.361, 0.965])), 0, 40
    );
    cursorLight.position.set(0, 0, 5);
    scene.add(cursorLight);

    scene.add(new THREE.AmbientLight(0x8899ff, 0.3));
    const dir = new THREE.DirectionalLight(0xffffff, 0.6);
    dir.position.set(8, 15, 10);
    scene.add(dir);
    const hemi = new THREE.HemisphereLight(0x4488ff, 0x112244, 0.4);
    scene.add(hemi);

    // --- 节点 mesh (结构化几何体, 角色区分) ---
    const nodeMeshes: THREE.Mesh[] = [];
    const nodeMats: THREE.MeshStandardMaterial[] = [];
    const nodeGroups: THREE.Group[] = [];
    const nodePulsePhases: number[] = [];

    nodes.forEach((n, i) => {
      const color = rgbToHex(readThemeRgb(ROLE_TOKEN[n.role], ROLE_FALLBACK[n.role]));
      const roleColor = new THREE.Color(color);

      const group = new THREE.Group();
      group.position.set(n.seed[0], n.seed[1], n.seed[2]);
      scene.add(group);

      // Different geometry per role
      let coreMesh: THREE.Mesh;
      let haloMesh: THREE.Mesh | null = null;
      let ringMesh: THREE.Mesh | null = null;

      if (n.role === "worktree") {
        // Worktree: Octahedron core + torus ring (hub indicator)
        const coreGeom = new THREE.OctahedronGeometry(0.55, 0);
        const coreMat = new THREE.MeshStandardMaterial({
          color,
          emissive: color,
          emissiveIntensity: 0.6,
          roughness: 0.15,
          metalness: 0.4,
        });
        coreMesh = new THREE.Mesh(coreGeom, coreMat);

        // Orbiting ring
        const ringGeom = new THREE.TorusGeometry(0.85, 0.03, 8, 32);
        const ringMat = new THREE.MeshBasicMaterial({
          color,
          transparent: true,
          opacity: 0.4,
          side: THREE.DoubleSide,
        });
        ringMesh = new THREE.Mesh(ringGeom, ringMat);
        ringMesh.rotation.x = Math.PI / 2;
        group.add(ringMesh);

        // Outer pulse ring
        const pulseGeom = new THREE.RingGeometry(0.95, 1.0, 32);
        const pulseMat = new THREE.MeshBasicMaterial({
          color,
          transparent: true,
          opacity: 0.2,
          side: THREE.DoubleSide,
        });
        haloMesh = new THREE.Mesh(pulseGeom, pulseMat);
        haloMesh.rotation.x = -Math.PI / 2;
        group.add(haloMesh);

      } else if (n.role === "agent") {
        // Agent: Icosahedron + wireframe + energy core
        const coreGeom = new THREE.IcosahedronGeometry(0.4, 1);
        const coreMat = new THREE.MeshStandardMaterial({
          color,
          emissive: color,
          emissiveIntensity: 0.7,
          roughness: 0.1,
          metalness: 0.5,
        });
        coreMesh = new THREE.Mesh(coreGeom, coreMat);

        // Wireframe shell
        const wireGeom = new THREE.IcosahedronGeometry(0.52, 1);
        const wireMat = new THREE.MeshBasicMaterial({
          color,
          wireframe: true,
          transparent: true,
          opacity: 0.3,
        });
        haloMesh = new THREE.Mesh(wireGeom, wireMat);
        group.add(haloMesh);

        // Inner energy sphere
        const innerGeom = new THREE.SphereGeometry(0.18, 12, 12);
        const innerMat = new THREE.MeshBasicMaterial({
          color: 0xffffff,
          transparent: true,
          opacity: 0.9,
        });
        const innerMesh = new THREE.Mesh(innerGeom, innerMat);
        group.add(innerMesh);

      } else {
        // Task: Tetrahedron + small orbiting particles
        const coreGeom = new THREE.TetrahedronGeometry(0.3, 0);
        const coreMat = new THREE.MeshStandardMaterial({
          color,
          emissive: color,
          emissiveIntensity: 0.5,
          roughness: 0.2,
          metalness: 0.35,
        });
        coreMesh = new THREE.Mesh(coreGeom, coreMat);

        // 3 micro particles orbiting
        for (let p = 0; p < 3; p++) {
          const microGeom = new THREE.SphereGeometry(0.05, 6, 6);
          const microMat = new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.7 });
          const microMesh = new THREE.Mesh(microGeom, microMat);
          microMesh.userData = { orbitAngle: p * (Math.PI * 2 / 3), orbitRadius: 0.5, orbitSpeed: 0.5 + p * 0.3 };
          group.add(microMesh);
        }
      }

      group.add(coreMesh);
      nodeMeshes.push(coreMesh);
      nodeMats.push(coreMesh.material as THREE.MeshStandardMaterial);
      nodeGroups.push(group);
      nodePulsePhases.push(Math.random() * Math.PI * 2);
    });

    // --- 边 (束状连线 + 流动粒子) ---
    const edgeLines: THREE.Line[] = [];
    const edgeGeometries: THREE.BufferGeometry[] = [];
    const edgeFlowParticles: { geometry: THREE.BufferGeometry; mesh: THREE.Points; progress: number }[] = [];

    edges.forEach((e) => {
      const geo = new THREE.BufferGeometry().setFromPoints([
        new THREE.Vector3(),
        new THREE.Vector3(),
      ]);
      edgeGeometries.push(geo);

      // Main line
      const lineMat = new THREE.LineBasicMaterial({
        color: rgbToHex(readThemeRgb("--cel-cyan", [0.486, 0.549, 1.0])),
        transparent: true,
        opacity: e.type === "agent-tree" ? 0.45 : 0.25,
        linewidth: e.type === "agent-tree" ? 2 : 1,
      });
      const line = new THREE.Line(geo, lineMat);
      scene.add(line);
      edgeLines.push(line);

      // Flow particles along edge
      const flowGeo = new THREE.BufferGeometry();
      const flowCount = 8;
      const flowPositions = new Float32Array(flowCount * 3);
      const flowSizes = new Float32Array(flowCount);
      for (let f = 0; f < flowCount; f++) {
        flowSizes[f] = 0.08 + Math.random() * 0.06;
      }
      flowGeo.setAttribute("position", new THREE.BufferAttribute(flowPositions, 3));
      flowGeo.setAttribute("size", new THREE.BufferAttribute(flowSizes, 1));
      const flowMat = new THREE.PointsMaterial({
        color: rgbToHex(readThemeRgb("--color-secondary", [0.545, 0.361, 0.965])),
        size: 0.1,
        transparent: true,
        opacity: 0.6,
        sizeAttenuation: true,
      });
      const flowMesh = new THREE.Points(flowGeo, flowMat);
      scene.add(flowMesh);
      edgeFlowParticles.push({ geometry: flowGeo, mesh: flowMesh, progress: Math.random() });
    });

    // --- 簇群光环 (worktree cluster glow) ---
    const clusterHalos: THREE.Mesh[] = [];
    wtClusterCenters.forEach((center, ci) => {
      const haloGeom = new THREE.SphereGeometry(3.5, 16, 12);
      const haloMat = new THREE.MeshBasicMaterial({
        color: rgbToHex(readThemeRgb("--ok-DEFAULT", [0.353, 0.824, 1.0])),
        transparent: true,
        opacity: 0.03,
        side: THREE.BackSide,
      });
      const halo = new THREE.Mesh(haloGeom, haloMat);
      halo.position.copy(center);
      scene.add(halo);
      clusterHalos.push(halo);
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
            const group = nodeGroups[i];
            group.position.set(p[0], p[1], p[2]);

            // 呼吸发光
            const node = nodes[i];
            const phase = nodePulsePhases[i] + now * 0.002;
            const pulseIntensity = 0.4 + Math.sin(phase) * 0.25;
            const mat = nodeMats[i];
            mat.emissiveIntensity = pulseIntensity;

            // Halo pulse for worktrees
            if (node.role === "worktree" && nodeGroups[i].children[1]) {
              const halo = nodeGroups[i].children[1] as THREE.Mesh;
              halo.scale.setScalar(0.9 + Math.sin(phase * 1.5) * 0.2);
              if (halo.material instanceof THREE.MeshBasicMaterial) {
                halo.material.opacity = 0.2 + Math.sin(phase * 2) * 0.15;
              }
            }
            // Ring rotation for worktrees
            if (node.role === "worktree" && nodeGroups[i].children[2]) {
              const ring = nodeGroups[i].children[2] as THREE.Mesh;
              ring.rotation.z += dt * 0.3;
            }
            // Wireframe counter-rotation for agents
            if (node.role === "agent" && nodeGroups[i].children[1]) {
              const wire = nodeGroups[i].children[1] as THREE.Mesh;
              wire.rotation.y += dt * 0.5;
              wire.rotation.x += dt * 0.3;
            }
            // Micro particle orbits for tasks
            if (node.role === "task") {
              nodeGroups[i].children.slice(1).forEach((child, pi) => {
                if (child instanceof THREE.Mesh && child.userData.orbitAngle !== undefined) {
                  child.userData.orbitAngle += dt * child.userData.orbitSpeed;
                  const a = child.userData.orbitAngle;
                  const r = child.userData.orbitRadius;
                  child.position.set(Math.cos(a) * r, Math.sin(pi * 1.7 + a * 0.5) * 0.3, Math.sin(a) * r);
                }
              });
            }

            if (p[1] < -8) w.attract(i, p[0], 8.0, p[2], 1.0);
            for (const s of shocks) {
              const dx = p[0] - s.x;
              const dy = p[1] - s.y;
              const dz = p[2] - (s.z || 0);
              const d2 = dx * dx + dy * dy + dz * dz;
              if (d2 < 100) {
                const d = Math.sqrt(d2) + 0.1;
                const f = (1 - s.age / 0.8) * (40 / d);
                w.attract(i, p[0] + dx * f * 2, p[1] + dy * f * 2, p[2] + dz * f * 2, 0.08);
              }
            }
          }
          for (const s of shocks) s.age += dt;
          while (shocks.length > 0 && shocks[0].age > 0.8) shocks.shift();

          // 边端点跟随节点 + 流动粒子
          for (let e = 0; e < edges.length; e++) {
            const { source, target, type } = edges[e];
            const pos = edgeGeometries[e].attributes.position as THREE.BufferAttribute;
            const srcPos = nodeGroups[source].position;
            const tgtPos = nodeGroups[target].position;
            pos.setXYZ(0, srcPos.x, srcPos.y, srcPos.z);
            pos.setXYZ(1, tgtPos.x, tgtPos.y, tgtPos.z);
            pos.needsUpdate = true;
            edgeGeometries[e].computeBoundingSphere();

            // Update flow particles
            const flow = edgeFlowParticles[e];
            if (flow && flow.mesh.visible) {
              const flowPos = flow.geometry.attributes.position as THREE.BufferAttribute;
              const flowSize = flow.geometry.attributes.size as THREE.BufferAttribute;
              flow.progress += dt * (type === "agent-tree" ? 0.8 : 0.5);
              if (flow.progress > 1) flow.progress -= 1;
              const count = flowPos.count;
              for (let f = 0; f < count; f++) {
                const p = (flow.progress + f / count) % 1;
                // Smooth curve with slight arc
                const midX = (srcPos.x + tgtPos.x) / 2 + Math.sin(p * Math.PI) * (tgtPos.z - srcPos.z) * 0.1;
                const midY = (srcPos.y + tgtPos.y) / 2 + Math.sin(p * Math.PI) * 0.5;
                const midZ = (srcPos.z + tgtPos.z) / 2 - Math.sin(p * Math.PI) * (tgtPos.x - srcPos.x) * 0.1;
                const x = srcPos.x * (1 - p) + tgtPos.x * p + Math.sin(p * Math.PI) * (midX - srcPos.x * (1 - p) - tgtPos.x * p);
                const y = srcPos.y * (1 - p) + tgtPos.y * p + Math.sin(p * Math.PI) * 0.5;
                const z = srcPos.z * (1 - p) + tgtPos.z * p + Math.sin(p * Math.PI) * (midZ - srcPos.z * (1 - p) - tgtPos.z * p);
                flowPos.setXYZ(f, x, y, z);
                // Pulse size
                flowSize.setX(f, 0.08 + Math.sin(p * Math.PI * 2 + now * 0.005) * 0.03);
              }
              flowPos.needsUpdate = true;
              flowSize.needsUpdate = true;
            }
          }

          // Cluster halo breathing
          clusterHalos.forEach((halo, ci) => {
            const phase = now * 0.001 + ci * 1.2;
            const scale = 0.9 + Math.sin(phase) * 0.1;
            halo.scale.setScalar(scale);
            if (halo.material instanceof THREE.MeshBasicMaterial) {
              halo.material.opacity = 0.02 + Math.sin(phase * 1.5) * 0.015;
            }
          });

          cursorLight.position.set(mouseWorld.x, mouseWorld.y, 5);
          cursorLight.intensity = 1.5 + Math.sin(now * 0.004) * 0.5;

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
        x: ((e.clientX / w) * 2 - 1) * 16,
        y: -((e.clientY / h) * 2 - 1) * 10,
        z: 0,
      };
    };
    const onPointerMove = (e: PointerEvent) => {
      if (!world) return;
      const t = screenToWorld(e);
      mouseWorld = t;
      for (let i = 0; i < nodeMeshes.length; i += 3) {
        world.attract(i, t.x, t.y, 0, 0.12);
      }
    };
    const onPointerDown = (e: PointerEvent) => {
      if (!world) return;
      const t = screenToWorld(e);
      shocks.push({ x: t.x, y: t.y, z: 0, age: 0 });
      if (shocks.length > 6) shocks.shift();
    };
    const onBurst = (e: Event) => {
      if (!world) return;
      const d = (e as CustomEvent<{ x?: number; y?: number; z?: number }>).detail ?? {};
      const t = d.x !== undefined && d.y !== undefined
        ? { x: d.x, y: d.y, z: d.z ?? 0 }
        : { x: (Math.random() - 0.5) * 16, y: 3, z: (Math.random() - 0.5) * 10 };
      shocks.push({ x: t.x, y: t.y, z: t.z, age: -0.5 });
      if (shocks.length > 6) shocks.shift();
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
        const mat = m.material;
        if (mat) {
          if (Array.isArray(mat)) {
            mat.forEach(m => m.dispose());
          } else {
            mat.dispose();
          }
        }
      });
      edgeGeometries.forEach((g) => g.dispose());
      edgeFlowParticles.forEach(f => f.geometry.dispose());
      clusterHalos.forEach(h => { h.geometry.dispose(); if (h.material && !Array.isArray(h.material)) h.material.dispose(); });
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
