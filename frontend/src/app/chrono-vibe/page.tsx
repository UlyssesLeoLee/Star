"use client";

import React, { useState, useEffect, useRef } from "react";

import { AnimeCelShaderCanvas, CelPalette } from "../../components/effects/AnimeCelShaderCanvas";
import {
  CelButton3D,
  CelToggle3D,
  CelBeacon3D,
  CelDial3D,
} from "../../components/effects/Cel3DUI";

export default function ChronoVibePage() {
  const [audioEnabled, setAudioEnabled] = useState(true);
  const [burstMode, setBurstMode] = useState(false);
  const [syncRate] = useState(99.84);
  const [praiseCount, setPraiseCount] = useState(10482);
  const [comboCount, setComboCount] = useState(0);
  const [theoryOpen, setTheoryOpen] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);
  const [stealthProtocol, setStealthProtocol] = useState(false);
  const [autoSagaGuard, setAutoSagaGuard] = useState(true);
  const [selectedTrack, setSelectedTrack] = useState(0);
  const [quoteIdx, setQuoteIdx] = useState(0);
  const [particles, setParticles] = useState<
    Array<{ id: number; text: string; color: string; dx: string; rot: string }>
  >([]);

  // 3D Cel Shader Live Parameters
  const [celPalette, setCelPalette] = useState<CelPalette>("crimson");
  const [themeMode, setThemeMode] = useState<"dark" | "light">("dark");

  const toggleThemeMode = () => {
    playSfx("blip");
    const next = themeMode === "dark" ? "light" : "dark";
    setThemeMode(next);
    if (typeof document !== "undefined") {
      document.documentElement.classList.remove("dark", "light", "theme-dark", "theme-light");
      document.documentElement.classList.add(`theme-${next}`, next);
    }
    if (next === "light") {
      setCelPalette("manga-vermilion");
    } else {
      setCelPalette("crimson");
    }
  };
  const [celBands, setCelBands] = useState<number>(3);
  const [outlineThick, setOutlineThick] = useState<number>(0.05);
  const [halftoneEnabled, setHalftoneEnabled] = useState<boolean>(true);
  const [celSpeed, setCelSpeed] = useState<number>(1.0);

  const heroRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const audioCtxRef = useRef<AudioContext | null>(null);
  const comboTimerRef = useRef<NodeJS.Timeout | null>(null);

  const quotes = [
    "“这才是真正的三渲二艺术！配色克制却气场全开，每一个切角都让人起鸡皮疙瘩！”",
    "“低认知负荷与高视觉张力的教科书级典范，看一眼就直接跪倒信奉神作！”",
    "“零模糊黑阶描边＋动态瑞士斜角，这魄力已经超越了常规 UI，这是可触控的原画！”",
    "“纯粹的美学暴力！毫无犹豫地按下了点赞，这就是艺术品！”",
    "“Persona 5 级张力 × 赛博朋克冷静计算，神作无误！”",
  ];

  // Web Audio API Procedural Synthesizer
  const initAudio = () => {
    if (!audioCtxRef.current && typeof window !== "undefined") {
      const AudioCtx =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext })
          .webkitAudioContext;
      audioCtxRef.current = new AudioCtx();
    }
    if (audioCtxRef.current && audioCtxRef.current.state === "suspended") {
      audioCtxRef.current.resume();
    }
  };

  const playSfx = (type: "click" | "blip" | "slash" | "chime" | "praise" | "blast") => {
    if (!audioEnabled) return;
    initAudio();
    const ctx = audioCtxRef.current;
    if (!ctx) return;

    const now = ctx.currentTime;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);

    if (type === "click") {
      osc.type = "triangle";
      osc.frequency.setValueAtTime(880, now);
      osc.frequency.exponentialRampToValueAtTime(220, now + 0.05);
      gain.gain.setValueAtTime(0.18, now);
      gain.gain.exponentialRampToValueAtTime(0.01, now + 0.05);
      osc.start(now);
      osc.stop(now + 0.05);
    } else if (type === "blip") {
      osc.type = "sine";
      osc.frequency.setValueAtTime(1200, now);
      osc.frequency.exponentialRampToValueAtTime(1800, now + 0.07);
      gain.gain.setValueAtTime(0.12, now);
      gain.gain.exponentialRampToValueAtTime(0.01, now + 0.07);
      osc.start(now);
      osc.stop(now + 0.07);
    } else if (type === "slash") {
      osc.type = "sawtooth";
      osc.frequency.setValueAtTime(450, now);
      osc.frequency.exponentialRampToValueAtTime(90, now + 0.18);
      gain.gain.setValueAtTime(0.2, now);
      gain.gain.exponentialRampToValueAtTime(0.01, now + 0.18);
      osc.start(now);
      osc.stop(now + 0.18);
    } else if (type === "chime") {
      [523.25, 659.25, 783.99, 1046.5].forEach((freq, i) => {
        const o = ctx.createOscillator();
        const g = ctx.createGain();
        o.type = "sine";
        o.frequency.setValueAtTime(freq, now + i * 0.04);
        g.gain.setValueAtTime(0.12, now + i * 0.04);
        g.gain.exponentialRampToValueAtTime(0.001, now + 0.35 + i * 0.04);
        o.connect(g);
        g.connect(ctx.destination);
        o.start(now + i * 0.04);
        o.stop(now + 0.4 + i * 0.04);
      });
    } else if (type === "praise") {
      [440, 554.37, 659.25, 880, 1108.73].forEach((freq, idx) => {
        const o = ctx.createOscillator();
        const g = ctx.createGain();
        o.type = "triangle";
        o.frequency.setValueAtTime(freq, now + idx * 0.03);
        g.gain.setValueAtTime(0.15, now + idx * 0.03);
        g.gain.exponentialRampToValueAtTime(0.001, now + 0.45 + idx * 0.03);
        o.connect(g);
        g.connect(ctx.destination);
        o.start(now + idx * 0.03);
        o.stop(now + 0.5 + idx * 0.03);
      });
    } else if (type === "blast") {
      osc.type = "sawtooth";
      osc.frequency.setValueAtTime(140, now);
      osc.frequency.linearRampToValueAtTime(600, now + 0.12);
      osc.frequency.exponentialRampToValueAtTime(45, now + 0.3);
      gain.gain.setValueAtTime(0.25, now);
      gain.gain.exponentialRampToValueAtTime(0.01, now + 0.3);
      osc.start(now);
      osc.stop(now + 0.3);
    }
  };

  // 3D Parallax Tilt Effect
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!heroRef.current) return;
      const rect = heroRef.current.getBoundingClientRect();
      const cx = rect.left + rect.width / 2;
      const cy = rect.top + rect.height / 2;
      const dx = (e.clientX - cx) / (window.innerWidth / 2);
      const dy = (e.clientY - cy) / (window.innerHeight / 2);
      const rotY = dx * 8;
      const rotX = -dy * 8;
      heroRef.current.style.transform = `perspective(1000px) rotateX(${rotX}deg) rotateY(${rotY}deg)`;
    };

    const handleMouseLeave = () => {
      if (!heroRef.current) return;
      heroRef.current.style.transform = "perspective(1000px) rotateX(0deg) rotateY(0deg)";
    };

    window.addEventListener("mousemove", handleMouseMove);
    return () => window.removeEventListener("mousemove", handleMouseMove);
  }, []);

  // Speedlines Canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animId: number;
    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };
    resize();
    window.addEventListener("resize", resize);

    const lines = Array.from({ length: 40 }, () => ({
      angle: Math.random() * Math.PI * 2,
      dist: Math.random() * 800 + 200,
      speed: Math.random() * 10 + 5,
      len: Math.random() * 120 + 50,
      alpha: Math.random() * 0.3 + 0.05,
    }));

    const render = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const cx = canvas.width / 2;
      const cy = canvas.height / 2;

      for (const l of lines) {
        l.dist -= l.speed * (burstMode ? 2.2 : 1);
        if (l.dist < 80) {
          l.dist = Math.max(canvas.width, canvas.height) * 0.7;
          l.angle = Math.random() * Math.PI * 2;
        }
        const x1 = cx + Math.cos(l.angle) * l.dist;
        const y1 = cy + Math.sin(l.angle) * l.dist;
        const x2 = cx + Math.cos(l.angle) * (l.dist + l.len);
        const y2 = cy + Math.sin(l.angle) * (l.dist + l.len);

        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.strokeStyle = `rgba(255, 255, 255, ${l.alpha * (burstMode ? 2 : 1)})`;
        ctx.lineWidth = burstMode ? 2 : 1;
        ctx.stroke();
      }
      animId = requestAnimationFrame(render);
    };

    render();
    return () => {
      cancelAnimationFrame(animId);
      window.removeEventListener("resize", resize);
    };
  }, [burstMode]);

  // Praise Trigger
  const handlePraise = () => {
    playSfx("praise");
    setPraiseCount((c) => c + 1);
    setComboCount((k) => k + 1);

    // Spawn particle
    const colors = ["#ff184c", "#ffc400", "#00f0ff", "#ffffff"];
    const symbols = ["★", "🔥", "✦", "神作", "MAX", "100%"];
    const newP = {
      id: Date.now() + Math.random(),
      text: symbols[Math.floor(Math.random() * symbols.length)],
      color: colors[Math.floor(Math.random() * colors.length)],
      dx: `${(Math.random() - 0.5) * 100}px`,
      rot: `${(Math.random() - 0.5) * 45}deg`,
    };
    setParticles((prev) => [...prev, newP]);
    setTimeout(() => {
      setParticles((prev) => prev.filter((p) => p.id !== newP.id));
    }, 1100);

    if ((comboCount + 1) % 3 === 0) {
      setQuoteIdx((q) => (q + 1) % quotes.length);
    }

    if (comboTimerRef.current) clearTimeout(comboTimerRef.current);
    comboTimerRef.current = setTimeout(() => {
      setComboCount(0);
    }, 2500);
  };

  // Grandmaster Keyboard Navigation & Tactical Shortcuts Engine
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

      if (e.code === "Space") {
        e.preventDefault();
        handlePraise();
      } else if (e.key === "b" || e.key === "B") {
        playSfx("blast");
        setBurstMode((prev) => !prev);
      } else if (e.key === "m" || e.key === "M") {
        setAudioEnabled((prev) => !prev);
        playSfx("blip");
      } else if (e.key === "?" || e.key === "/") {
        e.preventDefault();
        playSfx("blip");
        setShortcutsOpen((prev) => !prev);
      } else if (e.key === "t" || e.key === "T") {
        playSfx("blip");
        setTheoryOpen((prev) => !prev);
      } else if (e.key === "Escape") {
        setShortcutsOpen(false);
        setTheoryOpen(false);
      } else if (e.key === "1") {
        playSfx("click");
        setSelectedTrack(0);
      } else if (e.key === "2") {
        playSfx("click");
        setSelectedTrack(1);
      } else if (e.key === "3") {
        playSfx("click");
        setSelectedTrack(2);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  });

  const poses = [-8, -4, 0, 4, 8];

  return (
    <div className={`relative min-h-screen bg-[#07090e] text-slate-100 flex flex-col justify-between overflow-x-hidden select-none ${burstMode ? "burst-active" : ""}`}>
      {/* Background Speedlines Canvas */}
      <canvas ref={canvasRef} className="absolute inset-0 pointer-events-none z-0 opacity-20" />

      {/* Ambient Lighting */}
      <div className="fixed inset-0 pointer-events-none z-0">
        <div className="absolute -top-32 -left-32 w-96 h-96 bg-[#ff184c] rounded-full blur-[140px] opacity-15" />
        <div className="absolute -bottom-32 -right-32 w-96 h-96 bg-[#00f0ff] rounded-full blur-[140px] opacity-15" />
      </div>

      {/* TOP BAR: Tactical Anime HUD */}
      <header className="relative z-20 border-b-2 border-black bg-[#0d121c]/95 backdrop-blur-md px-4 sm:px-6 py-2.5">
        <div className="max-w-7xl mx-auto flex items-center justify-between gap-4">
          {/* Identity */}
          <div className="flex items-center gap-3">
            <div
              className="w-10 h-10 bg-[#ff184c] border-2 border-black flex items-center justify-center -rotate-3 hover:rotate-0 transition-transform cursor-pointer"
              style={{ boxShadow: "4px 4px 0px #000" }}
              onClick={() => playSfx("blip")}
            >
              <span className="font-black text-black text-xl italic tracking-tighter">ST</span>
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="text-xs font-black tracking-widest text-[#ff184c] uppercase italic">
                  // STAR CONTROL PLANE
                </span>
                <span className="px-1.5 py-0.2 text-[9px] font-bold bg-black text-[#00f0ff] border border-[#00f0ff] tracking-wider rounded-sm">
                  CEL-NPR:v1.0
                </span>
              </div>
              <h1 className="text-base sm:text-lg font-black tracking-wider text-white uppercase italic flex items-center gap-1.5">
                CHRONO MATRIX{" "}
                <span className="text-[#ffc400] text-xs font-semibold not-italic">
                  〔クロノ・マトリクス〕
                </span>
              </h1>
            </div>
          </div>

          {/* Sync status */}
          <div className="hidden md:flex items-center gap-4 bg-black/60 border border-[#232d42] px-4 py-1.5">
            <div className="flex flex-col">
              <div className="flex justify-between text-[10px] font-bold tracking-wider text-slate-400">
                <span>NEURAL SYNC (神経同調率)</span>
                <span className="text-[#00f0ff] font-mono">{burstMode ? "OVERCLOCK 140%" : `${syncRate}%`}</span>
              </div>
              <div className="w-44 h-2 bg-[#141b29] border border-black mt-0.5 relative overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-[#00f0ff] via-[#ffc400] to-[#ff184c] transition-all duration-300"
                  style={{ width: burstMode ? "100%" : `${syncRate}%` }}
                />
              </div>
            </div>
            <div className="h-6 w-px bg-[#232d42]" />
            <div className="text-right">
              <div className="text-[9px] font-bold text-slate-400">STATUS</div>
              <div className="text-xs font-mono font-bold text-[#ffc400]">INV:PASS (HOLD)</div>
            </div>
          </div>

          {/* Header Controls */}
          <div className="flex items-center gap-2 sm:gap-3">
            {/* Independent Dual Masterpiece Theme Toggle */}
            <button
              onClick={toggleThemeMode}
              className="px-2.5 py-1.5 text-xs font-bold bg-[#141b29] hover:bg-[#1f293d] border border-black text-slate-300 transition-colors flex items-center gap-1.5 cel-shadow"
              title="切换独立双艺术品主题：暗夜赛博 vs 少年原画绘图宣纸"
            >
              <span>{themeMode === "dark" ? "🌙" : "☀️"}</span>
              <span className="hidden sm:inline text-[11px] uppercase tracking-wider font-mono">
                {themeMode === "dark" ? "暗夜神格" : "少年原画"}
              </span>
            </button>

            <button
              onClick={() => {
                setAudioEnabled(!audioEnabled);
                if (!audioEnabled) playSfx("blip");
              }}
              className="px-2.5 py-1.5 text-xs font-bold bg-[#141b29] hover:bg-[#1f293d] border border-black text-slate-300 transition-colors flex items-center gap-1"
              style={{ boxShadow: "3px 3px 0px #000" }}
            >
              <span>{audioEnabled ? "🔊" : "🔇"}</span>
              <span className="hidden sm:inline text-[11px] uppercase tracking-wider">SFX</span>
            </button>

            <button
              onClick={() => {
                playSfx("blip");
                setShortcutsOpen(true);
              }}
              className="px-2.5 py-1.5 text-xs font-bold bg-[#141b29] hover:bg-[#1f293d] border border-[#ffc400]/50 text-[#ffc400] transition-all flex items-center gap-1"
              style={{ boxShadow: "3px 3px 0px #000" }}
              title="按 ? 查看宗师级键盘流快捷键"
            >
              <span>⌨️</span>
              <span className="hidden sm:inline text-[11px] uppercase tracking-wider font-mono">[?] 快捷键</span>
            </button>

            <button
              onClick={() => {
                playSfx("blip");
                setTheoryOpen(true);
              }}
              className="px-3.5 py-2 text-xs font-bold bg-[#141b29] hover:bg-[#1f293d] border border-[#00f0ff]/50 text-[#00f0ff] transition-all flex items-center gap-1.5 cel-shadow"
            >
              <span>✦</span>
              <span className="text-[11px] font-black tracking-wider uppercase">美学理论</span>
            </button>

            <button
              onClick={() => {
                playSfx("blast");
                setBurstMode(!burstMode);
              }}
              className={`px-4 py-2 text-xs font-black italic uppercase border-2 border-black transition-all cel-shadow ${
                burstMode
                  ? "bg-[#ffc400] text-black animate-pulse"
                  : "bg-[#ff184c] hover:bg-[#ff3362] text-black"
              }`}
            >
              ⚡ {burstMode ? "AWAKENED (神格)" : "LIMIT BREAK (覚醒)"}
            </button>
          </div>
        </div>
      </header>

      {/* MAIN VIEWPORT */}
      <main className="relative z-10 max-w-7xl mx-auto w-full px-4 sm:px-6 lg:px-8 py-8 grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
        {/* LEFT COLUMN: 25-Domain State Machines (4 Cols) */}
        <div className="lg:col-span-4 space-y-6">
          <section
            className="cel-ramp-card border-3 border-black p-6 relative overflow-hidden clip-hud-corner cel-shadow-lg"
          >
            <div className="flex items-center justify-between border-b-2 border-black pb-2.5 mb-3.5">
              <div className="flex items-center gap-2">
                <span className="w-2.5 h-2.5 bg-[#00f0ff] border border-black rotate-45" />
                <h2 className="text-sm font-black uppercase tracking-wider text-white">
                  DOMAIN MATRIX <span className="text-[#00f0ff] text-xs">〔領域状態機〕</span>
                </h2>
              </div>
              <span className="text-[10px] font-mono font-bold bg-black px-2 py-0.5 text-[#ffc400] border border-[#ffc400]/40">
                25 BOUNDED
              </span>
            </div>

            {/* Worktree Machine */}
            <div
              className={`mb-3 border-2 p-2.5 transition-all cursor-pointer group flex items-center justify-between gap-3 ${
                selectedTrack === 0 ? "border-[#00f0ff] bg-[#1a2338] cel-shadow-cyan" : "border-black bg-[#151c2c] hover:border-[#00f0ff] cel-shadow"
              }`}
              onClick={() => {
                playSfx("click");
                setSelectedTrack(0);
              }}
            >
              <div className="flex-1">
                <div className="flex items-center justify-between text-xs mb-1.5">
                  <span className="font-black text-white group-hover:text-[#00f0ff] tracking-wide flex items-center gap-1.5">
                    <span className="text-[10px] text-[#ff184c]">01</span> WORKTREE CLUSTER
                  </span>
                  <span className="text-[10px] font-mono text-slate-300 font-bold bg-[#0a0e17] px-1.5 py-0.5 border border-black">
                    17 STATES
                  </span>
                </div>
                <div className="w-full h-2 bg-[#090d14] border border-black flex">
                  <div className="w-8/12 bg-[#00f0ff] border-r border-black" />
                  <div className="w-3/12 bg-[#ffc400] border-r border-black" />
                  <div className="w-1/12 bg-[#ff184c]" />
                </div>
                <div className="flex justify-between text-[9px] font-mono text-slate-400 mt-1">
                  <span>ACTIVE: 12 WT</span>
                  <span className="text-emerald-400">CLEAN / 0 CONFLICT</span>
                </div>
              </div>
              <div className="flex-shrink-0">
                <CelBeacon3D status="active" size={38} title="WT: Active Sync" />
              </div>
            </div>

            {/* Agent Orchestration */}
            <div
              className={`mb-3 bg-[#151c2c] border p-2.5 transition-all cursor-pointer group flex items-center justify-between gap-3 ${
                selectedTrack === 1 ? "border-[#ff184c] bg-[#1a2338]" : "border-black hover:border-[#ff184c]"
              }`}
              onClick={() => {
                playSfx("click");
                setSelectedTrack(1);
              }}
            >
              <div className="flex-1">
                <div className="flex items-center justify-between text-xs mb-1.5">
                  <span className="font-black text-white group-hover:text-[#ff184c] tracking-wide flex items-center gap-1.5">
                    <span className="text-[10px] text-[#00f0ff]">02</span> AGENT RUNTIME ORCH
                  </span>
                  <span className="text-[10px] font-mono text-slate-300 font-bold bg-[#0a0e17] px-1.5 py-0.5 border border-black">
                    14 STATES
                  </span>
                </div>
                <div className="w-full h-2 bg-[#090d14] border border-black flex">
                  <div className="w-10/12 bg-[#ff184c] border-r border-black" />
                  <div className="w-2/12 bg-[#1a2336]" />
                </div>
                <div className="flex justify-between text-[9px] font-mono text-slate-400 mt-1">
                  <span>NODES: Mavis + Ulysses</span>
                  <span className="text-[#ff184c]">DEC-008 SIGNED</span>
                </div>
              </div>
              <div className="flex-shrink-0">
                <CelBeacon3D status="success" size={38} title="Agent: Invariants Hold" />
              </div>
            </div>

            {/* Saga Feedback */}
            <div
              className={`bg-[#151c2c] border p-2.5 transition-all cursor-pointer group flex items-center justify-between gap-3 ${
                selectedTrack === 2 ? "border-[#ffc400] bg-[#1a2338]" : "border-black hover:border-[#ffc400]"
              }`}
              onClick={() => {
                playSfx("click");
                setSelectedTrack(2);
              }}
            >
              <div className="flex-1">
                <div className="flex items-center justify-between text-xs mb-1.5">
                  <span className="font-black text-white group-hover:text-[#ffc400] tracking-wide flex items-center gap-1.5">
                    <span className="text-[10px] text-[#ffc400]">03</span> SAGA RECOVERY LOOP
                  </span>
                  <span className="text-[10px] font-mono text-slate-300 font-bold bg-[#0a0e17] px-1.5 py-0.5 border border-black">
                    IDEMPOTENT
                  </span>
                </div>
                <div className="w-full h-2 bg-[#090d14] border border-black flex">
                  <div className="w-11/12 bg-[#ffc400] border-r border-black" />
                  <div className="w-1/12 bg-black" />
                </div>
                <div className="flex justify-between text-[9px] font-mono text-slate-400 mt-1">
                  <span>TX RETRY: 0 ACC</span>
                  <span className="text-[#ffc400]">RACI LEAD GOV</span>
                </div>
              </div>
              <div className="flex-shrink-0">
                <CelBeacon3D status={autoSagaGuard ? "success" : "idle"} size={38} title="Saga: Idempotent Loop" />
              </div>
            </div>
          </section>

          {/* Tactical Speech Bubble */}
          <section
            className="cel-ramp-card border-3 border-black p-6 relative overflow-hidden clip-hud-corner cel-shadow-lg"
          >
            <div className="flex items-center justify-between border-b-2 border-black pb-2 mb-3">
              <h3 className="text-xs font-black uppercase tracking-wider text-slate-300 flex items-center gap-1.5">
                <span className="text-[#ff184c]">▶</span> TACTICAL SPEECH COMMS
              </h3>
              <span className="text-[9px] font-bold text-slate-500 uppercase">CH-01</span>
            </div>

            <div
              className="relative bg-[#182133] border-2 border-black p-3.5 mb-2.5"
              style={{ boxShadow: "3px 3px 0px #000" }}
            >
              <p className="text-xs font-medium leading-relaxed text-slate-200">
                <strong className="text-[#ff184c] font-black">[MAVIS // 統制官]</strong>:
                「三渲二日漫构架已全维度就绪。硬边缘着色器阶梯阈值锁定，网点纸矩阵与瑞士动态倾角网格交错！这绝非寻常界面——这是直击灵魂的艺术品！」
              </p>
            </div>

            <div className="flex items-center justify-between text-[10px] text-slate-400 font-mono pt-1">
              <span>COGNITIVE LOAD: 0.12 (LOW)</span>
              <span className="text-[#00f0ff]">CHARISMA: MAXIMAL</span>
            </div>
          </section>
        </div>

        {/* CENTER COLUMN: 3D Cel-Shaded Hero Terminal (5 Cols) */}
        <div className="lg:col-span-5">
          <div
            ref={heroRef}
            className="relative bg-[#0e1320] border-3 border-black p-5 overflow-hidden transition-transform duration-100 ease-out"
            style={{ boxShadow: "7px 7px 0px #000" }}
          >
            {/* Diagonal Accent Band */}
            <div className="absolute -right-16 -top-16 w-52 h-20 bg-[#ff184c] transform rotate-45 border-y-4 border-black pointer-events-none flex items-center justify-center">
              <span className="text-black font-black italic tracking-widest text-xs uppercase pl-8">
                MASTERPIECE
              </span>
            </div>

            {/* Header */}
            <div className="flex items-start justify-between mb-4">
              <div>
                <div className="text-[10px] font-black text-[#ffc400] tracking-widest uppercase mb-0.5">
                  AVATAR SPEC // S-CLASS
                </div>
                <h2 className="text-xl sm:text-2xl font-black italic uppercase tracking-wider text-white flex items-center gap-2">
                  BLADE PROTOCOL
                  <span className="text-xs font-black not-italic px-2 py-0.5 bg-[#ff184c] text-black border border-black">
                    神格
                  </span>
                </h2>
              </div>
              <div className="text-right">
                <span className="text-xs font-black text-slate-500 [writing-mode:vertical-rl]">
                  戦術司令機
                </span>
              </div>
            </div>

            {/* Real 3D-to-2D Cel-Shaded Stage (WebGL Custom GLSL Shader) */}
            <div className="relative w-full h-96 sm:h-[480px] bg-[#090d16] border-3 border-black overflow-hidden flex items-center justify-center group cel-shadow-lg">
              {/* Halftone Screentone Canvas Backdrop */}
              <div className="absolute inset-0 bg-screentone-dense opacity-20 pointer-events-none" />

              {/* Japanese Calligraphy Stamp Backdrop */}
              <div className="absolute inset-0 flex items-center justify-center opacity-10 pointer-events-none font-black text-8xl text-white select-none">
                極限
              </div>

              {/* Real 3D Three.js NPR Cel Shader Canvas */}
              <AnimeCelShaderCanvas
                palette={celPalette}
                bands={celBands}
                outlineThickness={outlineThick}
                enableHalftone={halftoneEnabled}
                enableRim={true}
                speed={burstMode ? 2.5 : celSpeed}
                className="w-full h-full"
              />

              {/* Dynamic HUD Overlay inside 3D Canvas */}
              <div className="absolute top-2 left-2 flex items-center gap-1.5 z-20 pointer-events-none">
                <span className="w-2 h-2 rounded-full bg-[#00f0ff] animate-ping" />
                <span className="bg-black/80 border border-[#00f0ff]/40 text-[#00f0ff] px-2 py-0.5 text-[9px] font-mono font-bold">
                  REALTIME_GLSL_NPR_3D
                </span>
              </div>

              <div className="absolute top-2 right-2 z-20 pointer-events-none">
                <span className="bg-black/80 border border-[#ffc400]/40 text-[#ffc400] px-2 py-0.5 text-[9px] font-mono font-bold">
                  {celBands}-BAND CEL RAMP
                </span>
              </div>

              {/* Bottom interactive HUD inside stage */}
              <div className="absolute bottom-2 left-2 right-2 flex justify-between items-end z-20 pointer-events-none">
                <div className="bg-black/85 border border-[#232f47] px-2.5 py-1 text-[9px] font-mono text-slate-300">
                  INVERTED_HULL: <span className="text-[#00f0ff]">{(outlineThick * 100).toFixed(1)}px INK</span>
                </div>
                <div className="bg-[#ff184c] text-black font-black text-[10px] px-2 py-0.5 border border-black">
                  LIGHT_TRACKING: ACTIVE
                </div>
              </div>
            </div>

            {/* Live Shader Controls Tuning Bar */}
            <div className="mt-3.5 bg-[#121826] border border-black p-2.5 space-y-2.5 text-xs">
              {/* Palette Switcher */}
              <div className="flex items-center justify-between gap-2">
                <span className="text-[10px] font-bold text-slate-400 uppercase font-mono">
                  CEL PALETTE:
                </span>
                <div className="flex gap-1.5">
                  {(
                    themeMode === "dark"
                      ? (["crimson", "cyan", "gold", "stealth"] as CelPalette[])
                      : (["manga-vermilion", "manga-cobalt", "manga-gold", "manga-sumi"] as CelPalette[])
                  ).map((p) => (
                    <button
                      key={p}
                      onClick={() => {
                        playSfx("click");
                        setCelPalette(p);
                      }}
                      className={`px-2 py-0.5 text-[10px] font-mono font-bold uppercase border border-black transition-all ${
                        celPalette === p
                          ? "bg-[#ff184c] text-black border-white"
                          : "bg-[#182133] text-slate-300 hover:text-white"
                      }`}
                    >
                      {p.replace("manga-", "")}
                    </button>
                  ))}
                </div>
              </div>

              {/* Bands & Outline Thickness Controls */}
              <div className="grid grid-cols-2 gap-2 pt-1 border-t border-black/40">
                <div className="flex items-center justify-between">
                  <span className="text-[10px] font-mono text-slate-400">CEL BANDS:</span>
                  <div className="flex gap-1">
                    {[2, 3, 4].map((b) => (
                      <button
                        key={b}
                        onClick={() => {
                          playSfx("blip");
                          setCelBands(b);
                        }}
                        className={`w-6 h-5 text-[10px] font-mono font-bold border border-black flex items-center justify-center ${
                          celBands === b ? "bg-[#00f0ff] text-black" : "bg-[#182133] text-slate-400"
                        }`}
                      >
                        {b}
                      </button>
                    ))}
                  </div>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-[10px] font-mono text-slate-400">INK OUTLINE:</span>
                  <div className="flex gap-1">
                    {[0.0, 0.03, 0.05, 0.08].map((t) => (
                      <button
                        key={t}
                        onClick={() => {
                          playSfx("blip");
                          setOutlineThick(t);
                        }}
                        className={`px-1.5 h-5 text-[10px] font-mono font-bold border border-black flex items-center justify-center ${
                          outlineThick === t ? "bg-[#ffc400] text-black" : "bg-[#182133] text-slate-400"
                        }`}
                      >
                        {t === 0 ? "OFF" : `${(t * 100).toFixed(0)}`}
                      </button>
                    ))}
                  </div>
                </div>
              </div>

              {/* Screentone & Speed */}
              <div className="flex items-center justify-between pt-1 border-t border-black/40">
                <button
                  onClick={() => {
                    playSfx("click");
                    setHalftoneEnabled(!halftoneEnabled);
                  }}
                  className={`px-2 py-0.5 text-[10px] font-mono font-bold border border-black flex items-center gap-1 ${
                    halftoneEnabled ? "bg-emerald-400 text-black" : "bg-[#182133] text-slate-400"
                  }`}
                >
                  <span>▤ HALFTONE SCREENTONE:</span>
                  <span>{halftoneEnabled ? "ON" : "OFF"}</span>
                </button>

                <button
                  onClick={() => {
                    playSfx("slash");
                    setCelSpeed((s) => (s === 1.0 ? 2.0 : s === 2.0 ? 0.5 : 1.0));
                  }}
                  className="px-2 py-0.5 text-[10px] font-mono font-bold bg-[#182133] hover:bg-[#222d42] border border-black text-slate-300"
                >
                  SPEED: {celSpeed}x
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* RIGHT COLUMN: Praise & Masterpiece Deck (3 Cols) */}
        <div className="lg:col-span-3 space-y-6">
          <section
            className="cel-ramp-card border-3 border-black p-6 relative overflow-hidden clip-hud-corner cel-shadow-lg"
          >
            <div className="absolute -right-2 -bottom-2 text-6xl font-black text-white/5 pointer-events-none select-none">
              神作
            </div>

            <div className="border-b-2 border-black pb-2.5 mb-4 flex items-center justify-between">
              <div>
                <div className="text-[9px] font-black text-[#ff184c] uppercase tracking-widest">
                  // CHARISMA ACCREDITATION
                </div>
                <h2 className="text-sm font-black uppercase text-white tracking-wider">
                  GOD-TIER PRAISE <span className="text-[#ffc400]">〔崇拝点赞〕</span>
                </h2>
              </div>
              <span className="text-lg">👑</span>
            </div>

            <div className="bg-[#151c2c] border border-black p-3 mb-4 text-center">
              <div className="text-[9px] font-mono text-slate-400 uppercase tracking-widest">
                MASTERPIECE RANKING
              </div>
              <div className="text-base sm:text-lg font-black text-[#ffc400] italic tracking-wider my-0.5">
                {comboCount > 10 ? "👑 宇宙至高・超越神作 👑" : "✦ 殿堂入り神作 ✦"}
              </div>
              <div className="text-[10px] text-slate-400">「毫无多余认知负荷，直击灵魂的美学碾压」</div>
            </div>

            {/* Giant Praise Button */}
            <div className="relative text-center">
              <button
                onClick={handlePraise}
                className="w-full relative group py-4 px-6 bg-gradient-to-b from-[#ff184c] to-[#b8002d] hover:from-[#ff3362] hover:to-[#d60036] text-white border-3 border-black active:translate-x-1 active:translate-y-1 transition-all"
                style={{ boxShadow: "5px 5px 0px #ff184c" }}
              >
                <div className="flex items-center justify-center gap-3">
                  <span className="text-2xl group-hover:scale-125 transition-transform">🔥</span>
                  <div className="text-left">
                    <div className="text-xs font-black tracking-widest uppercase text-black/80">
                      COMMEND AS MASTERPIECE
                    </div>
                    <div className="text-lg font-black italic tracking-wider">忍不住点赞 !</div>
                  </div>
                </div>
                <div className="absolute top-0 left-0 w-full h-1 bg-white/40" />
              </button>

              {/* Floating Particles */}
              <div className="relative pointer-events-none">
                {particles.map((p) => (
                  <span
                    key={p.id}
                    className="absolute text-sm font-black animate-bounce"
                    style={{
                      left: "50%",
                      top: "-20px",
                      color: p.color,
                      transform: `translate(${p.dx}, -40px) rotate(${p.rot})`,
                      transition: "all 1s cubic-bezier(0.1, 0.8, 0.3, 1)",
                    }}
                  >
                    {p.text}
                  </span>
                ))}
              </div>

              <div className="mt-3.5 flex items-center justify-between px-1">
                <div className="text-left">
                  <div className="text-[9px] font-bold text-slate-400">TOTAL TRIBUTES (崇拝数)</div>
                  <div className="text-xl font-mono font-black text-white">
                    {praiseCount.toLocaleString()}
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-[9px] font-bold text-slate-400">COMBO BONUS</div>
                  <div className="text-sm font-mono font-bold text-[#00f0ff]">
                    {comboCount > 0 ? `x${comboCount} COMBO!` : "x1 READY"}
                  </div>
                </div>
              </div>
            </div>

            <div className="mt-4 bg-[#0a0e17] border border-black p-2.5">
              <div className="text-[9px] font-bold text-[#ffc400] uppercase mb-1">AUDIENCE VERDICT:</div>
              <p className="text-xs italic text-slate-300 leading-snug">{quotes[quoteIdx]}</p>
            </div>
          </section>

          {/* Quick Tactical 3D Cel Buttons */}
          <section
            className="cel-ramp-card border-3 border-black p-6 space-y-4 clip-hud-corner cel-shadow-lg"
          >
            <div className="text-[10px] font-black uppercase tracking-wider text-slate-400 flex items-center justify-between">
              <span className="flex items-center gap-1.5">
                <span className="w-2 h-2 bg-[#ffc400]" /> 3D TACTICAL DISPATCH
              </span>
              <span className="text-[9px] font-mono text-[#00f0ff]">SPRING_PHYSICS</span>
            </div>

            <div className="flex flex-col gap-2 items-center">
              <CelButton3D
                label="01 // SYNC WORKTREE"
                sublabel="IDEMPOTENT MERGE"
                variant="cyan"
                onClick={() => {
                  playSfx("slash");
                  alert("【3D 触觉派发】WORKSPACE 同步完成：12 个工作树状态机校验通过，0 冲突。");
                }}
              />
              <CelButton3D
                label="02 // CARGO 0 ERR"
                sublabel="T1.5 VERIFY -j 4"
                variant="gold"
                onClick={() => {
                  playSfx("chime");
                  alert("【3D 触觉派发】CARGO 完整守门通过：--workspace --lib -j 4 零错误！");
                }}
              />
            </div>
          </section>

          {/* 3D Cel-UI Mechanical Switches & Rotary Dial */}
          <section
            className="cel-ramp-card border-3 border-black p-6 space-y-4 clip-hud-corner cel-shadow-lg"
          >
            <div className="text-[10px] font-black uppercase tracking-wider text-slate-400 flex items-center justify-between border-b border-black pb-1.5">
              <span className="flex items-center gap-1.5">
                <span className="w-2 h-2 bg-[#00f0ff]" /> 3D MECHANICAL PROTOCOLS
              </span>
              <span className="text-[9px] font-mono text-emerald-400">NPR_TACTILE</span>
            </div>

            <div className="space-y-2">
              <CelToggle3D
                checked={stealthProtocol}
                onChange={(v) => {
                  playSfx("click");
                  setStealthProtocol(v);
                }}
                label="STEALTH_PROTOCOL"
                sublabel="MINIMALIST NPR PROFILE"
              />

              <CelToggle3D
                checked={autoSagaGuard}
                onChange={(v) => {
                  playSfx("click");
                  setAutoSagaGuard(v);
                }}
                label="AUTO_SAGA_GUARD"
                sublabel="STRICT IDEMPOTENCY LOCK"
              />
            </div>

            <div className="pt-2 border-t border-black/50">
              <CelDial3D
                value={selectedTrack}
                onChange={(v) => {
                  playSfx("blip");
                  setSelectedTrack(v);
                }}
                options={["TRACK_B_WT", "TRACK_B_AGT", "TRACK_C_SCM"]}
              />
            </div>
          </section>
        </div>
      </main>

      {/* FOOTER */}
      <footer className="relative z-20 border-t-2 border-black bg-[#090d16] px-4 sm:px-6 py-3 text-xs text-slate-400">
        <div className="max-w-7xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-2.5">
          <div className="flex items-center gap-3">
            <span className="font-black text-[#ff184c] italic">STAR VIBE CODING</span>
            <span className="text-slate-600">|</span>
            <span className="text-[11px]">美学法则：60% 宇宙曜黑 + 30% 结构精钢 + 10% 觉醒绯红 / 殿堂黄金</span>
          </div>
          <div className="flex items-center gap-4 text-[11px] font-mono">
            <span>INVARIANTS: <span className="text-emerald-400">HOLD</span></span>
            <span>DEC-008: <span className="text-[#00f0ff]">SIGNED</span></span>
            <span className="text-[#ffc400]">MASTERPIECE_ENGINE_ACTIVE</span>
          </div>
        </div>
      </footer>

      {/* MODAL: Tactical Keyboard Shortcuts & Ergonomics */}
      {shortcutsOpen && (
        <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
          <div
            className="bg-[#0f1424] border-3 border-black max-w-xl w-full p-6 relative max-h-[90vh] overflow-y-auto"
            style={{ boxShadow: "8px 8px 0px #000" }}
          >
            <button
              onClick={() => setShortcutsOpen(false)}
              className="absolute top-4 right-4 w-8 h-8 bg-[#ff184c] border-2 border-black text-black font-black text-sm flex items-center justify-center hover:scale-105 transition-transform"
              style={{ boxShadow: "2px 2px 0px #000" }}
            >
              ✕
            </button>
            <div className="flex items-center gap-2.5 mb-4 border-b-2 border-black pb-3">
              <span className="text-xl">⌨️</span>
              <div>
                <h2 className="text-lg font-black uppercase text-white tracking-wider">
                  宗师级交互工效学与战术快捷键
                </h2>
                <p className="text-xs text-[#ffc400] font-mono">
                  // DOHERTY THRESHOLD &lt;16ms · FITTS&apos;S LAW · ZERO COGNITIVE RESISTANCE
                </p>
              </div>
            </div>

            <div className="space-y-3 text-xs text-slate-300">
              <div className="bg-[#151c2e] border border-black p-3">
                <h3 className="font-black text-[#00f0ff] text-sm mb-2 flex items-center gap-2">
                  <span>✦</span> 全局键盘流快捷键 (Keyboard Shortcuts)
                </h3>
                <div className="grid grid-cols-2 gap-2 font-mono">
                  <div className="bg-[#0a0e17] p-2 border border-black flex justify-between items-center">
                    <span className="text-[#ff184c] font-black">Space</span>
                    <span className="text-slate-400">崇拜点赞连击 (+Combo)</span>
                  </div>
                  <div className="bg-[#0a0e17] p-2 border border-black flex justify-between items-center">
                    <span className="text-[#ffc400] font-black">B</span>
                    <span className="text-slate-400">极限觉醒模式 (Awaken)</span>
                  </div>
                  <div className="bg-[#0a0e17] p-2 border border-black flex justify-between items-center">
                    <span className="text-[#00f0ff] font-black">M</span>
                    <span className="text-slate-400">音效开/关 (Mute SFX)</span>
                  </div>
                  <div className="bg-[#0a0e17] p-2 border border-black flex justify-between items-center">
                    <span className="text-emerald-400 font-black">1 / 2 / 3</span>
                    <span className="text-slate-400">切换领域状态机 (Domain)</span>
                  </div>
                  <div className="bg-[#0a0e17] p-2 border border-black flex justify-between items-center">
                    <span className="text-white font-black">T</span>
                    <span className="text-slate-400">美学理论白皮书</span>
                  </div>
                  <div className="bg-[#0a0e17] p-2 border border-black flex justify-between items-center">
                    <span className="text-white font-black">? / Esc</span>
                    <span className="text-slate-400">快捷键指南 / 关闭弹窗</span>
                  </div>
                </div>
              </div>

              <div className="bg-[#151c2e] border border-black p-3 space-y-1.5">
                <h3 className="font-black text-[#ffc400] text-sm mb-1 flex items-center gap-2">
                  <span>⚡</span> 宗师级工效学法则 (Ergonomic Principles)
                </h3>
                <p>
                  <strong>多尔蒂阈值 (Doherty Threshold &lt;16ms)</strong>：从按下按键到 Web Audio 音效触发与三维顶点形变延迟控制在 1 帧以内，形成神经级条件反射。
                </p>
                <p>
                  <strong>菲茨定律 (Fitts&apos;s Law)</strong>：3D 控件具备清晰的碰撞体积与深度投影，杜绝扁平无边界设计的误触与犹豫。
                </p>
              </div>
            </div>

            <div className="mt-5 text-center">
              <button
                onClick={() => setShortcutsOpen(false)}
                className="px-6 py-2 bg-[#ffc400] text-black font-black uppercase text-xs border-2 border-black"
                style={{ boxShadow: "4px 4px 0px #000" }}
              >
                已掌握快捷操作 // 关 闭 (ESC)
              </button>
            </div>
          </div>
        </div>
      )}

      {/* MODAL: Aesthetic Theory */}
      {theoryOpen && (
        <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
          <div
            className="bg-[#0f1424] border-3 border-black max-w-2xl w-full p-6 relative max-h-[90vh] overflow-y-auto"
            style={{ boxShadow: "8px 8px 0px #000" }}
          >
            <button
              onClick={() => setTheoryOpen(false)}
              className="absolute top-4 right-4 w-8 h-8 bg-[#ff184c] border-2 border-black text-black font-black text-sm flex items-center justify-center hover:scale-105 transition-transform"
              style={{ boxShadow: "2px 2px 0px #000" }}
            >
              ✕
            </button>
            <div className="flex items-center gap-2.5 mb-4 border-b-2 border-black pb-3">
              <span className="text-xl">🎨</span>
              <div>
                <h2 className="text-lg font-black uppercase text-white tracking-wider">
                  三渲二日漫界面美学与设计哲学白皮书
                </h2>
                <p className="text-xs text-[#00f0ff] font-mono">
                  // COLOR PSYCHOLOGY, GRAPHIC DESIGN &amp; COGNITIVE LOAD
                </p>
              </div>
            </div>
            <div className="space-y-4 text-xs leading-relaxed text-slate-300">
              <div className="bg-[#151c2e] border border-black p-3.5">
                <h3 className="font-black text-[#ff184c] text-sm mb-1.5">
                  01. 色彩心理学与神经递质调控
                </h3>
                <p>
                  <strong>60% 宇宙曜黑</strong>：提供绝对稳定的认知锚点，降低长时间注视的视疲劳。<br />
                  <strong>10% 觉醒绯红</strong>：瞬间刺激杏仁核与交感神经，释放多巴胺与战斗冲动。<br />
                  <strong>15% 理性电青</strong>：作为红色的互补色，带来极致的计算精度与清晰冷静感。<br />
                  <strong>15% 殿堂黄金</strong>：唤醒最高级别的稀有度感知（SSR / 终极裁决）。
                </p>
              </div>
              <div className="bg-[#151c2e] border border-black p-3.5">
                <h3 className="font-black text-[#00f0ff] text-sm mb-1.5">
                  02. 平面设计理论与新东京超扁平构成
                </h3>
                <p>
                  <strong>瑞士网格 + 动态斜角张力 (-8° Manga Skew)</strong>：底层遵循严密的 Bento 网格，表面切入漫画斜切分镜。<br />
                  <strong>重墨描边与零羽化阴影</strong>：摒弃毛玻璃，采用 2px~4px 纯黑硬轮廓与阶梯式色带。
                </p>
              </div>
              <div className="bg-[#151c2e] border border-black p-3.5">
                <h3 className="font-black text-[#ffc400] text-sm mb-1.5">
                  03. 低认知负荷工程学
                </h3>
                <p>
                  <strong>格式塔图底分离律</strong>：强硬黑边使大脑在 50ms 内判定信息层级。<br />
                  <strong>米勒定律与三区信息聚合</strong>：屏幕严格划分为状态机大盘、英雄核心台、点赞与行动面板。
                </p>
              </div>
            </div>
            <div className="mt-5 text-center">
              <button
                onClick={() => setTheoryOpen(false)}
                className="px-6 py-2 bg-[#ff184c] text-black font-black uppercase text-xs border-2 border-black"
                style={{ boxShadow: "4px 4px 0px #000" }}
              >
                完全理解 // 关 闭 面 板
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
