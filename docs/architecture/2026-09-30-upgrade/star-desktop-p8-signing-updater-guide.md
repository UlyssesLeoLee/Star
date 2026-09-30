# Star Desktop — Tauri P8 Code Signing + 公证 + Auto-Updater 指南

> **date**: 2026-09-30 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: PR #256 follow-up of PR #255 (Multi-OS CI matrix)
> **status**: P8 production 必需 — Code signing + 公证 + auto-updater (per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P6 后续)

## 1. 目标

完成 production 发布必需 3 步骤:
1. **Code signing** (Windows + macOS + Linux)
2. **公证** (Notarization) (macOS 必需, Windows 建议)
3. **Auto-updater** (Tauri built-in updater)

## 2. Code Signing

### 2.1 Windows

| 工具 | 用途 | 来源 |
|---|---|---|
| **EV 代码签名证书** | Trusted by SmartScreen + 排除 Windows Defender 警告 | DigiCert / Sectigo / GlobalSign |
| **Azure Trusted Signing** | Cloud-based signing (推荐 2026+) | Azure account |
| **signtool.exe** | CLI 签名工具 | Windows SDK |
| **CI secret** | CERT + password | GitHub Actions secrets |

```bash
# Windows signing (per docs §7.1)
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 \
    /f cert.pfx /p "$WINDOWS_CERT_PASSWORD" \
    target/release/bundle/msi/star-desktop_0.1.0_x64_en-US.msi

# Verify signature
signtool verify /pa target/release/bundle/msi/star-desktop_0.1.0_x64_en-US.msi
```

### 2.2 macOS

| 工具 | 用途 | 来源 |
|---|---|---|
| **Developer ID Application 证书** | 公证必需 | Apple Developer Program ($99/year) |
| **App-Specific Password** | notarize tool 认证 | appleid.apple.com |
| **codesign** | CLI 签名工具 | Xcode Command Line Tools |
| **notarytool** | Apple 公证服务 (xcrun notarytool) | Xcode 13+ |
| **xcrun stapler** | Staple 公证 ticket | Xcode 13+ |

```bash
# macOS signing (per docs §7.2)
codesign --deep --force --verify --verbose \
    --sign "Developer ID Application: Star Inc. (TEAMID1234)" \
    --options runtime \
    --timestamp \
    target/release/bundle/macos/star-desktop.app

# Notarize
xcrun notarytool submit \
    --apple-id "developer@star.com" \
    --password "$APPLE_ID_PASSWORD" \
    --team-id "TEAMID1234" \
    target/release/bundle/dmg/star-desktop_0.1.0_x64.dmg

# Staple
xcrun stapler staple target/release/bundle/dmg/star-desktop_0.1.0_x64.dmg
```

### 2.3 Linux

| 工具 | 用途 |
|---|---|
| **GPG key** | .deb 签名 |
| **dpkg-sig** | .deb 签名 CLI 工具 |
| **AppImage --sign** | AppImage 内嵌签名 |

```bash
# Linux signing (per docs §7.3)
dpkg-sig --sign builder -k gpg-key-id \
    target/release/bundle/deb/star-desktop_0.1.0_amd64.deb

# AppImage 签名
appimagetool star-desktop.AppImage --sign --sign-key gpg-key-id
```

## 3. CI/CD 集成 (GitHub Actions secrets)

| Secret Name | 用途 | 来源 |
|---|---|---|
| `WINDOWS_CERT_BASE64` | Windows .pfx 证书 (base64 encoded) | DigiCert/Azure Trusted Signing |
| `WINDOWS_CERT_PASSWORD` | .pfx 密码 | 内部保管 |
| `APPLE_ID` | Apple Developer 邮箱 | Apple Developer Program |
| `APPLE_ID_PASSWORD` | App-Specific Password | appleid.apple.com |
| `APPLE_TEAM_ID` | Apple Developer Team ID | Apple Developer Program |
| `GPG_PRIVATE_KEY` | GPG signing key (Linux .deb) | 内部 GPG master key |

## 4. CI/CD Workflow (per PR-255 multi-OS matrix 扩展)

```yaml
# .github/workflows/star-desktop-build.yml (per PR-255) 扩展
linux-build:
  steps:
    - name: Sign .deb
      if: github.event_name == 'push' && github.ref == 'refs/heads/main'
      run: |
        echo "$GPG_PRIVATE_KEY" | gpg --import
        dpkg-sig --sign builder -k "$GPG_KEY_ID" \
            target/release/bundle/deb/star-desktop_0.1.0_amd64.deb

windows-build:
  steps:
    - name: Sign .msi
      if: github.event_name == 'push' && github.ref == 'refs/heads/main'
      run: |
        echo "$WINDOWS_CERT_BASE64" | base64 -d > cert.pfx
        signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 \
            /f cert.pfx /p "$WINDOWS_CERT_PASSWORD" \
            target/release/bundle/msi/star-desktop_0.1.0_x64_en-US.msi

macos-build:
  steps:
    - name: Sign .app
      if: github.event_name == 'push' && github.ref == 'refs/heads/main'
      run: |
        codesign --deep --force --verify --verbose \
            --sign "Developer ID Application: Star Inc. ($APPLE_TEAM_ID)" \
            --options runtime \
            --timestamp \
            target/release/bundle/macos/star-desktop.app

    - name: Notarize .dmg
      if: github.event_name == 'push' && github.ref == 'refs/heads/main'
      run: |
        xcrun notarytool submit \
            --apple-id "$APPLE_ID" \
            --password "$APPLE_ID_PASSWORD" \
            --team-id "$APPLE_TEAM_ID" \
            target/release/bundle/dmg/star-desktop_0.1.0_x64.dmg

    - name: Staple .dmg
      run: |
        xcrun stapler staple target/release/bundle/dmg/star-desktop_0.1.0_x64.dmg
```

## 5. Auto-Updater

### 5.1 Tauri built-in updater (per tauri.conf.json bundle config)

```json
{
  "tauri": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/UlyssesLeoLee/Star/releases/latest/download/{{target}}-{{arch}}.{{ext}}"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_BASE64"
    }
  }
}
```

### 5.2 Rust 端配置 (per crates/star-desktop/src-tauri/src/lib.rs)

```rust
// main.rs / lib.rs — auto-updater initialization
use tauri::updater::Builder;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Auto-updater 启动
            let handle = app.handle();
            tauri::async_runtime::spawn(async move {
                let updater = Builder::new()
                    .endpoint("https://github.com/UlyssesLeoLee/Star/releases/latest/download/{{target}}-{{arch}}.{{ext}}")
                    .pubkey("YOUR_PUBLIC_KEY")
                    .build();
                if let Err(e) = updater.update().await {
                    eprintln!("update failed: {e}");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 5.3 Update 触发流程

```
1. 用户启动 star-desktop v0.1.0
2. Auto-updater 检查 https://github.com/.../releases/latest
3. 比较 tauri.conf.json version (0.1.0) vs latest tag (0.2.0)
4. 若有更新 → 弹 dialog (用户确认) → 下载 + 验证签名 → 安装 + 重启
```

### 5.4 Update server 配置

- **GitHub Releases** (推荐, 集成简单):
  - 创建 release tag (e.g., v0.2.0)
  - 上传 3 平台 binary (.deb / .msi / .dmg) + signature (.sig)
  - tauri.conf.json endpoint 指向 github releases
- **自建 server**:
  - 静态文件 host (e.g., S3 + CloudFront)
  - manifest.json 含 version + signature

## 6. Tauri Updater 签名密钥生成

```bash
# 1. 生成 keypair (一次性, 妥善保管 private key)
openssl genrsa -out private_key.pem 2048
openssl rsa -in private_key.pem -pubout -out public_key.pem

# 2. base64 encode public key (写入 tauri.conf.json)
base64 -i public_key.pem

# 3. 妥善保管 private_key.pem (用于生成 release signature)
#    - GitHub Actions secret TAURI_SIGNING_PRIVATE_KEY
#    - 或者本地 1Password + 受限访问
```

## 7. 守门

| 守门 | 状态 |
|---|---|
| **#5 env 安全 fingerprint 仅 env name 不读 value** | ✅ TAURI_SIGNING_PRIVATE_KEY 仅引用 secret name |
| **#7 unsafe_code = "forbid"** | ✅ Tauri binary 0 unsafe (workspace lint) |
| **#19 0 动 V0.1 业务 logic** | ✅ docs 仅描述, scripts 仅构建 |
| **#11 缺标比错标** | ✅ 6 secret names + 3 platform tools 严格映射 |

## 8. 不在 PR-256 范围 (留后续)

- ❌ 实际申请 + 配置 EV 证书 (需 DigiCert 申请 + $200-500/year)
- ❌ 实际申请 Apple Developer Program ($99/year)
- ❌ GPG master key 生成 + 分布式密钥管理 (HashiCorp Vault)
- ❌ Auto-updater 实际部署 + 第一次 release 测试
- ❌ Distribution channels (GitHub Releases 已设计, 自建 server + auto-update channel 留后续)
- ❌ Crash reporting (Sentry/Bugsnag integration)

## 9. 后续 PR 计划

| PR | 内容 | 估 |
|---|---|---|
| **PR-257** | Code signing 实战: 申请 DigiCert EV + Apple Developer Program + GPG master key | 2 周 (外部依赖) |
| **PR-258** | Auto-updater 实战: 第一次 release v0.1.1 测试 update flow | 1 周 |
| **PR-259** | Crash reporting (Sentry integration via Tauri plugin) | 1 周 |
| **PR-260** | Distribution channel (GitHub Pages + auto-update server) | 1 周 |

## 10. Refs

- [crates/star-desktop/README.md](../crates/star-desktop/README.md) (PR #239 P0)
- [crates/star-desktop/src-tauri/tauri.conf.json](../crates/star-desktop/src-tauri/tauri.conf.json) (PR #239)
- [.github/workflows/star-desktop-build.yml](../.github/workflows/star-desktop-build.yml) (PR #242 + PR #255 multi-OS)
- [docs/architecture/2026-09-30-upgrade/star-desktop-p6-distribution-guide.md §7](../2026-09-30-upgrade/star-desktop-p6-distribution-guide.md) (PR #253)
- [docs/architecture/2026-09-30-upgrade/star-desktop-p7-e2e-performance-guide.md](../2026-09-30-upgrade/star-desktop-p7-e2e-performance-guide.md) (PR #254)
- [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P6 后续](../../architecture/2026-09-29-upgrade/rust-app-end-research.md)
- Tauri updater 文档: https://tauri.app/v1/guides/distribution/updater
- Apple notarization: https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution
- Windows code signing: https://docs.microsoft.com/en-us/windows/win32/seccrypto/cryptography-tools
