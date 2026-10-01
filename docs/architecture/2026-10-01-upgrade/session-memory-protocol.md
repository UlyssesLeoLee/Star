# Star Repository — Session Memory Integration Protocol

> **date**: 2026-10-01 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: PR-260 (session 内 5+ 反复实证 workaround 文档化, 加速未来 session 启动 + reduce浪费)
> **status**: protocol v1.0 (5 sections, 验证状态 per section)

## 背景

本 repo (Star monorepo, 95+ Rust crates + Tauri PoC + Multi-OS CI) 在 Windows + WSL2 + GitHub 协作环境下工作。session 内反复实证 5 类 environment-specific issues, 文档化以备后续 session 启动加速。

---

## 1. HTTPS git push 挂起 → SSH workaround

**问题**: `git push origin HEAD:refs/heads/xxx` 经 HTTPS (https://github.com/UlyssesLeoLee/Star.git) 反复超时 60-300s 挂起, 报 "Write failed: Broken pipe" 或 stalled。

**根因**: Windows machine 上 HTTPS push to GitHub 走 HTTP/2 long-lived connection + chunked transfer + git's smart-http, 部分网络栈 (企业 proxy / GFW) 不友好。证据 session 内 3+ 次。

**Workaround**:
```bash
git remote set-url origin git@github.com:UlyssesLeoLee/Star.git
```
SSH 走 22 端口, GitHub 强制纯 IPv4 + RSA/ED25519 短 payload, 立即 success。

**验证状态**: ✅ session 内 3+ 次实证 (PR #246/#248/#251/#252/#254/#257 等成功 push 0 挂起)。

---

## 2. WSL2 + k3s flannel CNI 死结

**问题**: `wsl -d Ubuntu -- bash -c 'sudo /usr/local/bin/k3s kubectl ...'` 反复超时 (3+ 次 10s+), 报 `kubectl connection refused` 或 `sudo ip link set cni0 up` 后 bash 子命令 hung。

**根因**: WSL2 1.4+ kernel 与 k3s flannel CNI 的 iptables/nftables interaction 在 cni0 state DOWN + linkdown 状态下, 后续 `sudo iptables-save` / `sudo ip link set cni0 up` 等命令触发 WSL2 deadlock (per session memory 反复实证)。

**Workaround** (host 端 PowerShell):
```powershell
wsl --shutdown
taskkill /F /IM wsl.exe /IM wslhost.exe 2>NUL
timeout /t 15
wsl -d Ubuntu
```
之后 session 内立即 verify `kubectl get nodes` Ready + 6 pods 1/1 Running。

**验证状态**: ✅ session 内 6+ 次实证, 唯一可靠路径, 部分恢复 (systemctl restart k3s) 偶有 success 但 port-forward race condition 持续, **host 端 wsl --shutdown 才是 100% fix**。

---

## 3. Tauri 2.0 Windows build STATUS_STACK_BUFFER_OVERRUN

**问题**: `cargo check -p star-desktop` 或 `cargo build -p star-desktop` 在 Windows machine 反复触发 STATUS_STACK_BUFFER_OVERRUN (rustc exit 0xc0000409), 报 "could not compile `cssparser-macros` due to previous error; rustc stack overflow"。

**根因**: Tauri 2.0 引入 100+ Windows crates (webview2-com 0.39 + windows 0.62 全套 + windows-sys + serde_with_macros 等), rustc compilation memory peak > 4GB 超过 Windows process stack 默认限制 (1 MB user-mode)。Macros like phf_codegen / cssparser-macros / wasm-bindgen-macro-support 在 deep expansion 时也触到类似 overflow。

**Workaround**:
1. **首选**: Linux build host (GitHub Actions ubuntu-22.04, 见 `.github/workflows/star-desktop-build.yml`, per PR #242 + PR #255 multi-OS matrix), Linux 默认 8 MB stack + flat memory model。
2. **临时**: `cargo metadata --no-deps` 轻量验证 workspace 完整性 (无需 compilation)。
3. **避免**: Windows machine 直接跑 `cargo check -p star-desktop` 或 `cargo build -p star-desktop`。

**验证状态**: ⚠ session 内实证, PR #239 build report。Linux CI 触发后验证 26 → 8 Rust tests pass (per PR #242 + #255)。

---

## 4. git worktree index.lock 残留 → 多次重试 workaround

**问题**: `git worktree remove --force` 失败报 `fatal: not a working tree`, 但 worktree 目录仍存在; 后续 `git branch -D` 报 `used by worktree`, 必须 `git worktree prune` + 用 Windows 正斜杠 path 重试。

**根因**: Windows path translation + git worktree registry 在 MINGW bash 下偶尔 stale; `git worktree remove` 的 path 解析错误 + registry removal 不彻底。

**Workaround** (3-step pattern):
```bash
# 1. 用 Windows 正斜杠 path (git worktree registry style)
git worktree remove --force "C:/Users/.../wt-ulys-xxx"

# 2. Prune stale registry
git worktree prune

# 3. Delete branch (squash-merged evidence required)
git branch -D agent/minimax/ulys-xxx
```

**Locked worktree** (git index.lock 残留 + "initializing" reason) 需要 user 授权:
```bash
# host 端:
rm /d/Star/.git/worktrees/wt-ulys-xxx/index.lock
# 之后:
git worktree unlock ../wt-ulys-xxx
git worktree remove --force "C:/Users/.../wt-ulys-xxx"
git branch -D agent/minimax/ulys-xxx
```

**验证状态**: ✅ session 内 26/28 wt-ulys-* cleanup 成功 (per PR #260-cleanup), 2 个 LOCKED wt-ulys-tauri-impl-docs 留待 user 授权。

---

## 5. squash merge 后 origin/dev stale + auto-delete

**问题**: PR 通过 GitHub squash merge 后, `origin/dev` 仍指向旧 dev HEAD (Squash 之前), 而新 commit 在 `origin/main` (Squash 目标)。`git push origin dev` 报 `Everything up-to-date`, 但 `git status` 报 `ahead of main +N`。同时 GitHub auto-delete `origin/dev` (per PR-merge auto-delete-branch workflow), 后续 `git fetch` 显示 `origin/dev: [gone]`。

**根因**: GitHub PR squash merge 不修改 `dev` branch ref, 只在 `main` 上创建 squash commit + 后 dev branch 自删除。local dev 仍 cache 旧 dev HEAD, 导致与 origin/main 出现"ahead"假象。

**Workaround** (3-step pattern):
```bash
# 1. Fetch 最新 (含 [gone] 清理)
git fetch origin --prune --force

# 2. Restore dev to current main state
git checkout -B dev origin/dev
git reset --hard origin/main
# 现在 dev = main, 0 ahead/behind

# 3. Force push (sync back to GitHub)
git push -f origin dev
```

**Squash 破坏 ancestry → branch 删除用 -D** (守门 #6):
- `git branch -d <branch>` 只在 fast-merged (squash 也算 not-merged) 时成功
- `git branch -D <branch>` 强制 delete, 但需**先 verify squash-merge 证据** (PR number + commit message on main)

**验证状态**: ✅ session 内 20+ PR squash merge 实证 (PR #239-#259 Tauri PoC P0→P10 + 11 WASM PR + 7 Codex PR)。

---

## Refs

- PR #239-#259 — Tauri PoC P0→P10 (21 PRs 全合 main)
- PR #242 — Linux build CI workflow (ubuntu-22.04)
- PR #255 — Multi-OS CI matrix (ubuntu + windows + macos)
- PR #246-#252 — Tauri frontend 集成 (React 19 + Vite + WASM)
- `.github/workflows/star-desktop-build.yml` — Linux CI 主入口
- `crates/star-desktop/README.md` — Tauri 2.0 架构
- `docs/architecture/2026-09-29-upgrade/rust-app-end-research.md` — 4 路径对比 research v0.1
- `docs/architecture/2026-09-30-upgrade/star-desktop-p0p10-final-impl-summary.md` — P0→P10 final summary
- `docs/architecture/2026-09-30-upgrade/star-desktop-p7-e2e-performance-guide.md` — 5 services + performance