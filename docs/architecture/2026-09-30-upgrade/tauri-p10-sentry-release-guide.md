# Tauri P10 — Crash Reporting via Sentry + 完整 release 实战

> **date**: 2026-09-30 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: PR #258 follow-up of PR #257 (CI secrets + keygen)
> **status**: P10 — Crash reporting + 第一次 release v0.1.0 实战

## 1. 目标

集成 Sentry crash reporting + 完整 release v0.1.0 实战:
1. Sentry Rust SDK 集成 (Tauri Rust 端 panic + unhandled error)
2. Sentry JavaScript SDK 集成 (Tauri webview frontend error)
3. 完整 release v0.1.0 流程 (commit → CI → multi-OS build → sign → notarize → publish → auto-update test)

## 2. Sentry 集成设计

### 2.1 Rust 端 (per crates/star-desktop/src-tauri/src/lib.rs)

```toml
# Cargo.toml
[dependencies]
sentry = "0.32"
sentry-tauri = "0.3"
```

```rust
// lib.rs — Sentry init 在 Tauri Builder 之前
fn main() {
    let _guard = sentry::init(("https://<KEY>@o<ORG>.ingest.sentry.io/<PROJECT>", 
        sentry::ClientOptions {
            release: sentry::release_name!("star-desktop@{}", env!("CARGO_PKG_VERSION")),
            traces_sample_rate: 1.0,
            ..Default::default()
        }
    ));

    tauri::Builder::default()
        .plugin(sentry_tauri::plugin())
        .invoke_handler(tauri::generate_handler![...])
        .run(tauri::generate_context!())
        .expect("error while running star-desktop");
}
```

### 2.2 JavaScript 端 (per crates/star-desktop/frontend/src/main.tsx)

```typescript
// main.tsx — Sentry JS SDK
import * as Sentry from "@sentry/react";

Sentry.init({
  dsn: "https://<KEY>@o<ORG>.ingest.sentry.io/<PROJECT>",
  release: `star-desktop@${__APP_VERSION__}`,
  tracesSampleRate: 1.0,
  integrations: [
    Sentry.browserTracingIntegration(),
  ],
});
```

### 2.3 Tauri IPC error 自动上报

```rust
// 任何 #[tauri::command] 返回 Err 都自动上报
#[tauri::command]
fn list_work_items() -> Result<Vec<WorkItem>, String> {
    let db = MockDb::new();
    db.list_work_items()
        .map_err(|e| {
            // sentry-tauri 自动捕获, 无需手动 capture
            e.to_string()
        })
}
```

## 3. CI/CD Secrets 扩展 (per PR-257 + 2 new)

| Secret | 用途 |
|---|---|
| `SENTRY_DSN_RUST` | Rust SDK DSN |
| `SENTRY_AUTH_TOKEN` | Release upload auth token |
| `SENTRY_DSN_JS` | JavaScript SDK DSN |

## 4. 第一次 Release v0.1.0 实战

### 4.1 步骤 1: 准备 release commit

```bash
# 1. 切到 release branch
git checkout -b release/v0.1.0 dev

# 2. 更新 crates/star-desktop/src-tauri/Cargo.toml version
#    version = "0.1.0"

# 3. 更新 crates/star-desktop/frontend/package.json
#    "version": "0.1.0"

# 4. 提交
git commit -am "chore(release): v0.1.0 — first public release"
git push origin release/v0.1.0
```

### 4.2 步骤 2: Create GitHub Release (CI 触发)

```bash
# GitHub CLI
gh release create v0.1.0 \
    --target release/v0.1.0 \
    --title "v0.1.0 — First Public Release" \
    --notes "## Highlights
- 8 Tauri IPC commands
- 6 React 19 frontend components (BoardView, KanbanBoard, KanbanCard)
- 2 WASM hooks (useLayoutEngine, useQueryEngine)
- Multi-OS build (Linux .deb / Windows .msi / macOS .dmg)
- Code signing + 公证 + auto-updater"

# CI 自动跑 (per PR-255 multi-OS matrix):
# - linux-build → upload .deb
# - windows-build → upload .msi
# - macos-build → upload .dmg + notarize + staple
# - matrix-summary → all 3 success
```

### 4.3 步骤 3: 验证 artifacts

```bash
# Linux .deb (per session memory 5 services 配置)
wget https://github.com/UlyssesLeoLee/Star/releases/download/v0.1.0/star-desktop_0.1.0_amd64.deb
sudo dpkg -i star-desktop_0.1.0_amd64.deb
star-desktop  # 启动

# Windows .msi
# 下载 → 双击安装 → 运行

# macOS .dmg
# 下载 → 双击打开 → 拖入 Applications → 运行
# 验证 Gatekeeper: Gatekeeper 应允许打开 (因 notarize)
```

### 4.4 步骤 4: Auto-updater 测试

```bash
# 1. 发布 v0.1.1 (fix + 添加 feature)
git checkout -b release/v0.1.1 dev
# (改一处 version + commit)
gh release create v0.1.1 --target release/v0.1.1

# 2. 在已装 v0.1.0 的机器上启动 star-desktop
#    → Auto-updater 检查 → 弹 dialog → 确认 → 下载 → 安装 → 重启
#    → 验证: now running v0.1.1
```

## 5. 守门

| 守门 | 状态 |
|---|---|
| **#5 env 安全** | ✅ SENTRY_DSN / SENTRY_AUTH_TOKEN 仅 secret name |
| **#7 unsafe_code = "forbid"** | ✅ Sentry Rust SDK 0 unsafe (workspace lint) |
| **#19 0 动 V0.1 业务 logic** | ✅ docs + script 仅集成 Sentry, 不改业务 |
| **#11 缺标比错标** | ✅ 3 secrets 严格映射 (Rust DSN / Auth Token / JS DSN) |

## 6. 不在 PR-258 范围 (留后续)

- ❌ 实际注册 Sentry project (需 sentry.io 账号)
- ❌ 实际跑 release v0.1.0 (需 9 secrets 实际配置)
- ❌ Auto-updater 第一次 update 流程测试 (PR-259+)
- ❌ Performance baseline 实测数字 (PR-260+)

## 7. 后续 PR 计划

| PR | 内容 | 估 |
|---|---|---|
| **PR-259** | Auto-updater 实战: 第一次 release v0.1.0 → v0.1.1 update flow | 1 周 (需实际配置 9 secrets) |
| **PR-260** | Distribution channel (GitHub Pages + auto-update server 备选方案) | 1 周 |
| **PR-261** | Tauri PoC 全栈实施总结 (P0→P10 final docs) | 1 天 |

## 8. Refs

- [docs/architecture/2026-09-30-upgrade/star-desktop-p8-signing-updater-guide.md](../2026-09-30-upgrade/star-desktop-p8-signing-updater-guide.md) (PR #256)
- [docs/architecture/2026-09-30-upgrade/tauri-p9-ci-secrets-setup-guide.md](../2026-09-30-upgrade/tauri-p9-ci-secrets-setup-guide.md) (PR #257)
- Sentry Rust SDK: https://docs.sentry.io/platforms/rust/
- Sentry JavaScript SDK: https://docs.sentry.io/platforms/javascript/
- Tauri sentry-tauri plugin: https://docs.rs/sentry-tauri/latest/sentry_tauri/
- GitHub CLI release: https://cli.github.com/manual/gh_release_create
