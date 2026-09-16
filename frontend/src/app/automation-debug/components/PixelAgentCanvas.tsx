"use client";

/**
 * PixelAgentCanvas — 页游水准的像素风机器人游戏角色
 *
 * 设计目标 (per 9/6 12:34 JST 用户发令):
 *   - agent 界面需要一个像素风游戏角色（机器人）
 *   - 配备 idle / move / attack 三段动画
 *   - 达到页游水准（造型 + 帧动画 + 反馈）
 *
 * 技术选型:
 *   - Canvas 2D + 像素级 fillRect 手绘 (无外部 sprite sheet, 无版权问题)
 *   - 100% 程序生成, 0 外部资源
 *   - 走 dynamic({ ssr: false }) 引入, 不污染 SSR / main 编译链 (per 守门 #22)
 *
 * 像素艺术规格:
 *   - 内部分辨率 64x64 像素 (page-web 经典角色大小)
 *   - 渲染时 image-smoothing disabled, CSS 放大 4x → 256x256 显示
 *   - 主体 + 阴影 + 装饰层 (天线/LED/炮口闪光) 分层绘制
 *
 * 动画系统:
 *   - idle:  2 帧循环 800ms — 身体上下浮动 + LED 呼吸 + 天线闪光
 *   - move:  4 帧循环 600ms — 双腿交替踏步 + 身体倾斜 + 脚底尘土粒子
 *   - attack: 5 帧一次性 500ms — 右臂炮台前冲 + 炮口闪光 + 冲击波环
 *
 * 交互:
 *   - 点击 → 触发 attack 动画 (一次性)
 *   - hover → 微微放大 + 整体光晕
 *   - autoDemo → idle 4s → move 2.5s → attack 0.5s 循环
 */

import { useEffect, useRef, useState, useCallback } from "react";

const SIZE = 64;             // 内部像素分辨率
const SCALE = 4;             // 渲染放大倍率
const CANVAS = SIZE * SCALE; // 显示尺寸 256x256

// 调色板
type Palette = {
  outline: string;
  body: string;
  bodyLight: string;
  bodyDark: string;
  accent: string;
  led: string;
  eye: string;
  panel: string;
};

const DEFAULT_PALETTE: Palette = {
  outline: "#0a0e1a",
  body: "#3b5bdb",
  bodyLight: "#748ffc",
  bodyDark: "#1c2e6e",
  accent: "#9c36b5",
  led: "#51cf66",
  eye: "#ff4757",
  panel: "#dee2e6",
};

// === 帧状态 (由 computeFrameState 计算) ===
interface FrameState {
  bobY: number;            // 身体上下浮动 (px, 内部坐标系)
  bodyTilt: number;        // 身体倾斜 (-1/0/+1)
  leftLegPhase: number;    // 左腿相位 (0/1/2/3)
  rightLegPhase: number;
  rightArmPunch: number;   // 右臂前冲 (0-10 px)
  eyeBlink: boolean;
  ledAlpha: number;        // LED 呼吸 (0.6-1.0)
  antennaSpark: boolean;
  muzzleFlash: number;     // 炮口闪光强度 (0-3)
  shockwave: number;       // 冲击波环 (0-2)
  dust: Array<{ x: number; y: number; life: number }>;
}

function emptyFrameState(): FrameState {
  return {
    bobY: 0,
    bodyTilt: 0,
    leftLegPhase: 0,
    rightLegPhase: 0,
    rightArmPunch: 0,
    eyeBlink: false,
    ledAlpha: 1.0,
    antennaSpark: false,
    muzzleFlash: 0,
    shockwave: 0,
    dust: [],
  };
}

// 计算当前时间的 FrameState
function computeFrameState(
  state: "idle" | "move" | "attack",
  stateTime: number,
  prevDust: Array<{ x: number; y: number; life: number }>,
  frameIndex: number,
): FrameState {
  const s = emptyFrameState();

  if (state === "idle") {
    const t = (stateTime % 800) / 800;
    s.bobY = Math.sin(t * Math.PI * 2) * 1;
    s.ledAlpha = 0.6 + 0.4 * (0.5 + 0.5 * Math.sin(t * Math.PI * 2));
    s.antennaSpark = Math.floor(stateTime / 400) % 2 === 0;
    s.dust = prevDust.filter(d => d.life > 0).map(d => ({ ...d, life: d.life - 1 }));
  } else if (state === "move") {
    const phase = Math.floor((stateTime % 600) / 150); // 0/1/2/3
    s.bobY = phase === 1 || phase === 3 ? -1 : 0;
    s.bodyTilt = phase === 1 ? 1 : phase === 3 ? -1 : 0;
    s.leftLegPhase = phase;
    s.rightLegPhase = (phase + 2) % 4;
    s.ledAlpha = 0.7 + 0.3 * Math.sin((stateTime / 600) * Math.PI * 2);

    // 偶数相位生成尘土
    if ((phase === 0 || phase === 2) && frameIndex % 8 === 0) {
      s.dust = [
        ...prevDust.filter(d => d.life > 0).map(d => ({ ...d, life: d.life - 1 })),
        { x: 22, y: 62, life: 12 },
        { x: 42, y: 62, life: 12 },
      ].slice(0, 8);
    } else {
      s.dust = prevDust.filter(d => d.life > 0).map(d => ({ ...d, life: d.life - 1 }));
    }
  } else if (state === "attack") {
    const t = Math.min((stateTime % 500) / 500, 1);
    if (t < 0.2) {
      s.rightArmPunch = (t / 0.2) * 6;
      s.muzzleFlash = 0;
      s.shockwave = 0;
    } else if (t < 0.4) {
      s.rightArmPunch = 6 + ((t - 0.2) / 0.2) * 4;
      s.muzzleFlash = 2;
      s.shockwave = 1;
    } else if (t < 0.7) {
      s.rightArmPunch = 10 - ((t - 0.4) / 0.3) * 8;
      s.muzzleFlash = 1;
      s.shockwave = 2;
    } else {
      s.rightArmPunch = 0;
      s.muzzleFlash = 0;
      s.shockwave = 0;
    }
    s.bobY = -1;
    s.ledAlpha = 0.9 + 0.1 * Math.sin(t * Math.PI * 4);
    s.eyeBlink = t > 0.18 && t < 0.22;
    s.dust = prevDust.filter(d => d.life > 0).map(d => ({ ...d, life: d.life - 1 }));
  }

  return s;
}

// === 绘制函数 (内部 64x64 坐标系) ===

function drawRobot(ctx: CanvasRenderingContext2D, s: FrameState, p: Palette) {
  ctx.save();
  ctx.translate(0, s.bobY);

  // 阴影
  ctx.fillStyle = "rgba(0,0,0,0.25)";
  ctx.beginPath();
  ctx.ellipse(32, 63, 14, 2, 0, 0, Math.PI * 2);
  ctx.fill();

  // 身体倾斜
  ctx.save();
  ctx.translate(32, 38);
  ctx.rotate(s.bodyTilt * 0.05);
  ctx.translate(-32, -38);

  // 双腿
  drawLeg(ctx, 26, 48, s.leftLegPhase, p);
  drawLeg(ctx, 38, 48, s.rightLegPhase, p);

  // 躯干
  drawTorso(ctx, p, s.ledAlpha);

  // 左臂 (静止)
  drawArm(ctx, 14, 30, 0, p);

  // 右臂 (可前冲)
  drawArm(ctx, 50, 30, s.rightArmPunch, p);

  ctx.restore(); // 解 tilt

  // 头部 (不参与 tilt, 始终直立)
  drawHead(ctx, p, s.eyeBlink, s.antennaSpark);

  // 炮口闪光
  if (s.muzzleFlash > 0) {
    drawMuzzleFlash(ctx, 52 + s.rightArmPunch, 36, s.muzzleFlash);
  }

  // 冲击波环
  if (s.shockwave > 0) {
    drawShockwave(ctx, 32, 38, s.shockwave);
  }

  // 尘土粒子
  for (const d of s.dust) {
    const alpha = d.life / 12;
    ctx.fillStyle = `rgba(180,180,180,${alpha * 0.5})`;
    const offsetX = (12 - d.life) * 0.5;
    ctx.fillRect(d.x + offsetX, d.y - (12 - d.life) * 0.3, 2, 2);
  }

  ctx.restore();
}

function drawLeg(ctx: CanvasRenderingContext2D, cx: number, cy: number, phase: number, p: Palette) {
  ctx.save();
  const offsetY = phase === 1 ? -1 : phase === 3 ? 1 : 0;
  const offsetX = phase === 1 ? 1 : phase === 3 ? -1 : 0;
  const shinOffset = phase === 1 ? 1 : phase === 3 ? -1 : 0;

  // 大腿
  ctx.fillStyle = p.bodyDark;
  ctx.fillRect(cx - 2 + offsetX, cy + offsetY, 4, 6);
  // 膝关节描边
  ctx.fillStyle = p.outline;
  ctx.fillRect(cx - 2 + offsetX, cy + 5 + offsetY, 4, 1);
  // 小腿
  ctx.fillStyle = p.body;
  ctx.fillRect(cx - 1 + offsetX + shinOffset, cy + 6 + offsetY, 3, 5);
  ctx.fillStyle = p.bodyLight;
  ctx.fillRect(cx - 1 + offsetX + shinOffset, cy + 6 + offsetY, 1, 1);
  // 履带底座
  ctx.fillStyle = p.panel;
  ctx.fillRect(cx - 3, cy + 11, 6, 2);
  ctx.fillStyle = p.outline;
  ctx.fillRect(cx - 3, cy + 11, 6, 1);
  ctx.fillRect(cx - 2, cy + 11, 1, 1);
  ctx.fillRect(cx + 1, cy + 11, 1, 1);
  ctx.restore();
}

function drawTorso(ctx: CanvasRenderingContext2D, p: Palette, ledAlpha: number) {
  // 主体
  ctx.fillStyle = p.outline;
  ctx.fillRect(22, 26, 20, 20);
  ctx.fillStyle = p.body;
  ctx.fillRect(23, 27, 18, 18);
  // 高光
  ctx.fillStyle = p.bodyLight;
  ctx.fillRect(24, 28, 4, 2);
  ctx.fillRect(24, 28, 2, 4);
  // 阴影
  ctx.fillStyle = p.bodyDark;
  ctx.fillRect(38, 42, 3, 3);

  // 肩部护甲
  ctx.fillStyle = p.bodyDark;
  ctx.fillRect(20, 28, 3, 6);
  ctx.fillRect(41, 28, 3, 6);
  ctx.fillStyle = p.accent;
  ctx.fillRect(20, 28, 1, 6);
  ctx.fillRect(43, 28, 1, 6);

  // 胸口反应堆
  ctx.fillStyle = p.outline;
  ctx.fillRect(29, 33, 6, 6);
  ctx.fillStyle = p.panel;
  ctx.fillRect(30, 34, 4, 4);
  const glow = Math.floor(ledAlpha * 3);
  ctx.fillStyle = p.led;
  if (glow >= 2) {
    ctx.fillRect(31, 35, 2, 2);
    ctx.fillStyle = `rgba(81, 207, 102, ${ledAlpha * 0.4})`;
    ctx.fillRect(28, 32, 8, 8);
  } else if (glow >= 1) {
    ctx.fillRect(31, 35, 2, 1);
  }

  // 腹部甲片分隔线
  ctx.fillStyle = p.outline;
  ctx.fillRect(23, 41, 18, 1);
  // 腰部紫色装饰
  ctx.fillStyle = p.accent;
  ctx.fillRect(23, 44, 18, 1);
}

function drawArm(ctx: CanvasRenderingContext2D, hx: number, hy: number, punch: number, p: Palette) {
  const px = hx + punch;
  const py = hy;

  // 肩关节
  ctx.fillStyle = p.outline;
  ctx.fillRect(px - 1, py - 1, 3, 3);
  ctx.fillStyle = p.bodyDark;
  ctx.fillRect(px, py, 1, 1);

  // 上臂
  ctx.fillStyle = p.outline;
  ctx.fillRect(px - 1, py + 2, 4, 6);
  ctx.fillStyle = p.body;
  ctx.fillRect(px, py + 3, 2, 4);

  // 肘关节
  ctx.fillStyle = p.outline;
  ctx.fillRect(px, py + 8, 2, 1);

  // 前臂
  ctx.fillStyle = p.body;
  ctx.fillRect(px, py + 9, 2, 5);
  ctx.fillStyle = p.bodyLight;
  ctx.fillRect(px, py + 9, 1, 1);

  // 拳头 / 炮台
  if (punch > 0) {
    // 攻击状态: 拳头是炮台
    ctx.fillStyle = p.outline;
    ctx.fillRect(px - 1, py + 14, 4, 4);
    ctx.fillStyle = p.bodyDark;
    ctx.fillRect(px, py + 15, 2, 2);
    ctx.fillStyle = p.outline;
    ctx.fillRect(px + 2, py + 15, 2, 2);
  } else {
    // 普通拳头
    ctx.fillStyle = p.outline;
    ctx.fillRect(px - 1, py + 14, 3, 3);
    ctx.fillStyle = p.panel;
    ctx.fillRect(px, py + 15, 1, 1);
  }
}

function drawHead(ctx: CanvasRenderingContext2D, p: Palette, blink: boolean, spark: boolean) {
  // 头部主体
  ctx.fillStyle = p.outline;
  ctx.fillRect(21, 7, 22, 18);
  ctx.fillStyle = p.body;
  ctx.fillRect(22, 8, 20, 16);
  ctx.fillStyle = p.bodyLight;
  ctx.fillRect(23, 9, 4, 2);
  ctx.fillRect(23, 9, 2, 4);
  ctx.fillStyle = p.bodyDark;
  ctx.fillRect(38, 20, 4, 4);

  // 护目镜面板
  ctx.fillStyle = p.outline;
  ctx.fillRect(25, 15, 14, 6);
  ctx.fillStyle = p.panel;
  ctx.fillRect(26, 16, 12, 4);

  // 眼睛 (左右异色)
  if (!blink) {
    ctx.fillStyle = p.eye;
    ctx.fillRect(28, 17, 3, 2);
    ctx.fillStyle = "#22d3ee";
    ctx.fillRect(33, 17, 3, 2);
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(28, 17, 1, 1);
    ctx.fillRect(33, 17, 1, 1);
  } else {
    ctx.fillStyle = p.outline;
    ctx.fillRect(28, 18, 8, 1);
  }

  // 嘴部格栅
  ctx.fillStyle = p.outline;
  ctx.fillRect(29, 22, 6, 2);
  ctx.fillStyle = p.bodyDark;
  ctx.fillRect(30, 22, 1, 2);
  ctx.fillRect(32, 22, 1, 2);
  ctx.fillRect(34, 22, 1, 2);

  // 天线
  ctx.fillStyle = p.outline;
  ctx.fillRect(31, 3, 2, 5);
  if (spark) {
    ctx.fillStyle = p.led;
    ctx.fillRect(31, 2, 2, 2);
    ctx.fillStyle = "rgba(81, 207, 102, 0.5)";
    ctx.fillRect(29, 0, 6, 6);
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(31, 2, 2, 1);
  } else {
    ctx.fillStyle = p.accent;
    ctx.fillRect(31, 2, 2, 2);
  }

  // 散热鳍
  ctx.fillStyle = p.bodyDark;
  ctx.fillRect(24, 7, 1, 2);
  ctx.fillRect(27, 7, 1, 2);
  ctx.fillRect(37, 7, 1, 2);
  ctx.fillRect(40, 7, 1, 2);
}

function drawMuzzleFlash(ctx: CanvasRenderingContext2D, x: number, y: number, intensity: number) {
  if (intensity >= 3) {
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(x + 2, y - 1, 4, 4);
    ctx.fillStyle = "#fbbf24";
    ctx.fillRect(x + 1, y - 2, 6, 6);
  } else if (intensity >= 2) {
    ctx.fillStyle = "#fbbf24";
    ctx.fillRect(x + 2, y - 1, 3, 3);
    ctx.fillStyle = "#f97316";
    ctx.fillRect(x + 1, y, 5, 1);
  } else if (intensity >= 1) {
    ctx.fillStyle = "#fbbf24";
    ctx.fillRect(x + 2, y, 2, 1);
  }
  ctx.fillStyle = "#fbbf24";
  ctx.fillRect(x + 4 + Math.floor(intensity), y - 3, 1, 1);
  ctx.fillRect(x + 5, y + 1, 1, 1);
  ctx.fillStyle = "#ffffff";
  ctx.fillRect(x + 4, y, 1, 1);
}

function drawShockwave(ctx: CanvasRenderingContext2D, cx: number, cy: number, ring: number) {
  const sizes = ring === 1 ? [10, 14] : [18, 22];
  const alphas = ring === 1 ? [0.8, 0.4] : [0.6, 0.2];
  for (let i = 0; i < 2; i++) {
    const r = sizes[i];
    ctx.strokeStyle = `rgba(156, 54, 181, ${alphas[i]})`;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.ellipse(cx, cy, r, r * 0.5, 0, 0, Math.PI * 2);
    ctx.stroke();
  }
}

// === CSS 颜色 → 实际可 fill 颜色 (浏览器 color-mix 解析) ===
function resolveColor(input: string, fallback: string): string {
  if (!input) return fallback;
  try {
    const tmp = document.createElement("canvas");
    tmp.width = 1;
    tmp.height = 1;
    const tctx = tmp.getContext("2d");
    if (!tctx) return fallback;
    tctx.fillStyle = "#000";
    tctx.fillStyle = input;
    return (tctx.fillStyle as string) || fallback;
  } catch {
    return fallback;
  }
}

// === 主组件 ===
interface PixelAgentCanvasProps {
  autoDemo?: boolean;
  className?: string;
}

export function PixelAgentCanvas({ autoDemo = true, className = "" }: PixelAgentCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animRef = useRef<number>(0);
  const stateRef = useRef<"idle" | "move" | "attack">("idle");
  const stateStartRef = useRef<number>(0);
  const frameIndexRef = useRef<number>(0);
  const dustRef = useRef<Array<{ x: number; y: number; life: number }>>([]);
  const paletteRef = useRef<Palette>(DEFAULT_PALETTE);
  const [hover, setHover] = useState(false);

  const triggerAttack = useCallback(() => {
    if (stateRef.current === "attack") return;
    stateRef.current = "attack";
    stateStartRef.current = performance.now();
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.imageSmoothingEnabled = false;

    // 读 CSS 变量 → 调色板
    const styles = getComputedStyle(document.documentElement);
    const primary = styles.getPropertyValue("--color-primary").trim();
    const accent = styles.getPropertyValue("--color-accent-violet").trim();
    const ok = styles.getPropertyValue("--ok-DEFAULT").trim();

    paletteRef.current = {
      ...DEFAULT_PALETTE,
      body: resolveColor(primary, DEFAULT_PALETTE.body),
      bodyLight: resolveColor(
        primary ? `color-mix(in srgb, ${primary} 60%, white)` : "",
        DEFAULT_PALETTE.bodyLight,
      ),
      bodyDark: resolveColor(
        primary ? `color-mix(in srgb, ${primary} 50%, black)` : "",
        DEFAULT_PALETTE.bodyDark,
      ),
      accent: resolveColor(accent, DEFAULT_PALETTE.accent),
      led: resolveColor(ok, DEFAULT_PALETTE.led),
    };

    function drawGround() {
      ctx.fillStyle = "rgba(255,255,255,0.04)";
      ctx.beginPath();
      ctx.ellipse(CANVAS / 2, CANVAS - 16, 88, 14, 0, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = "rgba(255,255,255,0.05)";
      for (let i = 0; i < 7; i++) {
        for (let j = 0; j < 3; j++) {
          const x = (CANVAS / 2 - 60) + i * 20;
          const y = CANVAS - 22 + j * 8;
          ctx.fillRect(x, y, 2, 2);
        }
      }
    }

    function drawGlow() {
      const grad = ctx.createRadialGradient(CANVAS / 2, CANVAS / 2, 40, CANVAS / 2, CANVAS / 2, 120);
      grad.addColorStop(0, "rgba(116, 143, 252, 0.25)");
      grad.addColorStop(1, "rgba(116, 143, 252, 0)");
      ctx.fillStyle = grad;
      ctx.fillRect(0, 0, CANVAS, CANVAS);
    }

    const startTime = performance.now();
    stateStartRef.current = startTime;

    function loop() {
      const now = performance.now();
      const elapsed = now - startTime;
      const stateTime = now - stateStartRef.current;

      // 自动演示: idle 4s → move 2.5s → attack 0.5s → idle 1s
      if (autoDemo) {
        const cycle = elapsed % 8000;
        if (cycle < 4000) {
          if (stateRef.current !== "idle") {
            stateRef.current = "idle";
            stateStartRef.current = now;
            dustRef.current = [];
          }
        } else if (cycle < 6500) {
          if (stateRef.current !== "move") {
            stateRef.current = "move";
            stateStartRef.current = now;
          }
        } else if (cycle < 7000) {
          if (stateRef.current !== "attack") {
            stateRef.current = "attack";
            stateStartRef.current = now;
          }
        } else if (stateRef.current !== "idle") {
          stateRef.current = "idle";
          stateStartRef.current = now;
        }
      }

      // 清屏
      ctx.clearRect(0, 0, CANVAS, CANVAS);

      // 地面
      drawGround();

      // hover 整体光晕
      if (hover) drawGlow();

      // 内部 64x64 画布
      ctx.save();
      const offsetX = (CANVAS - SIZE * SCALE) / 2;
      const offsetY = (CANVAS - SIZE * SCALE) / 2;
      ctx.translate(offsetX, offsetY);
      ctx.scale(SCALE, SCALE);

      const fs = computeFrameState(
        stateRef.current,
        stateTime,
        dustRef.current,
        frameIndexRef.current,
      );
      if (fs.dust) dustRef.current = fs.dust;

      drawRobot(ctx, fs, paletteRef.current);
      ctx.restore();

      // HUD
      ctx.fillStyle = "rgba(255,255,255,0.4)";
      ctx.font = "bold 10px monospace";
      ctx.textAlign = "left";
      ctx.fillText(`[${stateRef.current.toUpperCase()}]`, 6, 12);
      ctx.textAlign = "right";
      ctx.fillText(
        `f${(frameIndexRef.current % 1000).toString().padStart(3, "0")}`,
        CANVAS - 6,
        12,
      );

      frameIndexRef.current++;
      animRef.current = requestAnimationFrame(loop);
    }

    animRef.current = requestAnimationFrame(loop);

    return () => {
      cancelAnimationFrame(animRef.current);
    };
  }, [hover, autoDemo]);

  return (
    <div
      className={`relative inline-block ${className}`}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      onClick={triggerAttack}
      role="button"
      tabIndex={0}
      aria-label="像素机器人 agent — 点击触发攻击动画"
      title="点击触发攻击"
    >
      <canvas
        ref={canvasRef}
        width={CANVAS}
        height={CANVAS}
        className="block cursor-pointer transition-transform duration-200"
        style={{
          imageRendering: "pixelated",
          transform: hover ? "scale(1.05)" : "scale(1)",
          width: `${CANVAS}px`,
          height: `${CANVAS}px`,
        }}
      />
      {hover && (
        <div
          className="absolute -bottom-6 left-1/2 -translate-x-1/2 text-[9px] font-mono uppercase tracking-wider px-2 py-0.5 rounded pointer-events-none whitespace-nowrap"
          style={{
            background: "color-mix(in srgb, var(--color-accent-violet) 20%, transparent)",
            color: "var(--color-accent-violet)",
            border: "1px solid color-mix(in srgb, var(--color-accent-violet) 40%, transparent)",
          }}
        >
          ⚔ CLICK → ATTACK
        </div>
      )}
    </div>
  );
}

export default PixelAgentCanvas;
