# Star Desktop — Tauri 2.0 打包分发指南 (P6)

> **date**: 2026-09-30 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: PR #253 follow-up of PR #252 (P5-5 BoardView 集成)
> **status**: P6 打包分发骨架 (per PR #244 §3.4 + P6 Tauri 全栈实施)
> **target**: 实现 cargo tauri build → 3 平台分发包 (Linux/Windows/macOS)

## 1. 目标

把 crates/star-desktop Tauri 2.0 应用打包成 3 平台分发包, 让用户下载安装即用。

## 2. 3 平台 targets

| 平台 | Target | 文件 | 大小 (估) | 备注 |
|---|---|---|---|---|
| **Linux** | `.deb` | star-desktop_0.1.0_amd64.deb | ~10 MB | Ubuntu/Debian (.deb + apt) |
| **Linux** | `.AppImage` | star-desktop_0.1.0_amd64.AppImage | ~12 MB | 跨 Linux 发行版 (universal) |
| **Windows** | `.msi` | star-desktop_0.1.0_x64_en-US.msi | ~10 MB | Windows 10/11 (WiX 3.x) |
| **macOS** | `.app` | star-desktop.app | ~15 MB | macOS 11+ (Universal binary: x86_64 + arm64) |
| **macOS** | `.dmg` | star-desktop_0.1.0_x64.dmg | ~16 MB | 磁盘映像 (drag-to-Applications) |

## 3. 构建脚本

### 3.1 scripts/build-desktop.sh (NEW per PR #253)

```bash
# Current platform default build
bash scripts/build-desktop.sh

# Linux .deb only
bash scripts/build-desktop.sh --target deb

# Linux AppImage only
bash scripts/build-desktop.sh --target appimage

# Windows .msi (requires WiX + cross-compile toolchain)
bash scripts/build-desktop.sh --target msi

# macOS .dmg (requires macOS host or cross-compile)
bash scripts/build-desktop.sh --target dmg

# All targets (cross-platform, may fail if some toolchains missing)
bash scripts/build-desktop.sh --all
```

### 3.2 Build steps (3 步骤, per build-desktop.sh)

1. **`npm install`** (frontend deps, per PR #246 package.json)
2. **`npm run build`** (Vite bundle to frontend/dist, per PR #246)
3. **`cargo tauri build`** (Rust + Tauri bundle, per tauri.conf.json bundle config)

### 3.3 Build time

| 阶段 | 时间 |
|---|---|
| 冷启动 (cold) | ~30-45 min (Tauri 2.0 Linux deps compile + Vite + Rust) |
| 增量 (incremental) | ~5-10 min (cache reuse) |

## 4. 系统依赖 (per platform)

### 4.1 Linux (Ubuntu 22.04+ / Debian 12+)

```bash
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    patchelf \
    build-essential
```

(GitHub Action `star-desktop-build.yml` per PR #242 自动安装)

### 4.2 Windows (10 / 11)

| 工具 | 来源 |
|---|---|
| **WiX Toolset 3.x** | https://wixtoolset.org/ (PATH: `C:\wix\`) |
| **cargo-xwin** | `cargo install cargo-xwin --locked` |
| **MSVC Build Tools** | Visual Studio 2019/2022 with C++ workload |

### 4.3 macOS (11 Big Sur+)

| 工具 | 来源 |
|---|---|
| **Xcode Command Line Tools** | `xcode-select --install` |
| **cargo-bundle** | `cargo install cargo-bundle` (Apple Silicon native) |

## 5. 构建产物 (output paths)

| Platform | Path |
|---|---|
| Linux .deb | `crates/star-desktop/src-tauri/target/release/bundle/deb/star-desktop_0.1.0_amd64.deb` |
| Linux AppImage | `crates/star-desktop/src-tauri/target/release/bundle/appimage/star-desktop_0.1.0_amd64.AppImage` |
| Windows .msi | `crates/star-desktop/src-tauri/target/release/bundle/msi/star-desktop_0.1.0_x64_en-US.msi` |
| macOS .app | `crates/star-desktop/src-tauri/target/release/bundle/macos/star-desktop.app` |
| macOS .dmg | `crates/star-desktop/src-tauri/target/release/bundle/dmg/star-desktop_0.1.0_x64.dmg` |

## 6. CI/CD (per PR #242)

```yaml
# .github/workflows/star-desktop-build.yml (per PR #242)
# ubuntu-22.04 + timeout 45min + cargo test -p star-desktop --lib
# 触发: PR base=dev + push dev/main
# 当前: 仅 cargo check + cargo test (待 PR-253+ 加 cargo tauri build)
```

P6+ 升级: 加 multi-OS matrix:

```yaml
strategy:
  matrix:
    os: [ubuntu-22.04, windows-2022, macos-13]
steps:
  - name: Build Linux .deb
    if: matrix.os == 'ubuntu-22.04'
    run: bash scripts/build-desktop.sh --target deb
  - name: Build Windows .msi
    if: matrix.os == 'windows-2022'
    run: bash scripts/build-desktop.sh --target msi
  - name: Build macOS .dmg
    if: matrix.os == 'macos-13'
    run: bash scripts/build-desktop.sh --target dmg
```

## 7. 签名 + 公证 (production 必需)

### 7.1 Windows

- `signtool sign /f cert.pfx /p password target/release/bundle/msi/*.msi`
- 用 EV 代码签名证书 (Azure Trusted Signing 或 DigiCert)

### 7.2 macOS

- `codesign --deep --force --verify --verbose --sign "Developer ID Application: ..." target/release/bundle/macos/star-desktop.app`
- `notarytool submit --apple-id ... --password ... --team-id ... star-desktop.dmg`
- 公证后 `xcrun stapler staple star-desktop.dmg`

### 7.3 Linux

- GPG 签名 .deb: `dpkg-sig --sign builder target/release/bundle/deb/*.deb`
- AppImage: `--sign` 选项

## 8. 守门

| 守门 | 状态 |
|---|---|
| **#7 unsafe_code = "forbid"** | ✅ 继承 workspace lint |
| **#19 0 动 V0.1 业务 logic** | ✅ scripts/build-desktop.sh 仅构建, 不改源码 |
| **#11 缺标比错标** | ✅ cargo / npm / node / tauri-cli 必需 dep 文档完整 |

## 9. 不在 PR-253 范围 (留后续)

- ❌ 实际跑 cargo tauri build (需 Linux build host per PR #242)
- ❌ Code signing (需 EV cert + Apple Developer ID)
- ❌ Auto-update (Tauri built-in updater)
- ❌ Distribution channels (GitHub Releases / website / package managers)
- ❌ Performance baseline (PR #254 P7)

## 10. Refs

- [scripts/build-desktop.sh](../scripts/build-desktop.sh) (NEW PR #253)
- [crates/star-desktop/README.md](../crates/star-desktop/README.md) (PR #239 P0 skeleton)
- [crates/star-desktop/src-tauri/tauri.conf.json](../crates/star-desktop/src-tauri/tauri.conf.json) (PR #239 bundle config)
- [.github/workflows/star-desktop-build.yml](../.github/workflows/star-desktop-build.yml) (PR #242 Linux CI)
- [docs/architecture/2026-09-30-upgrade/star-desktop-p5-frontend-impl-plan.md §3.4](../architecture/2026-09-30-upgrade/star-desktop-p5-frontend-impl-plan.md)
- [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P6](../../architecture/2026-09-29-upgrade/rust-app-end-research.md)
- Tauri 2.0 build 文档: https://tauri.app/v1/guides/distribution/
