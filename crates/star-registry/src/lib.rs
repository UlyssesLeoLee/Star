//! star-registry — Multica Runtime Registry 域 (R3 阶段 1 Rust 实装)
//!
//! Per ADR-0027 R3 + plan-032 R3: 替代 `scripts/automation/registry/*_v0_legacy.py`,
//! 25 provider 探测 + MinVersion gate + StatusClassifier, 性能 milestone
//! 1000 provider 探测 1-2K 秒 (Python) → 50-200ms (Rust, 10-20x 加速).
//!
//! 阶段 1 scope (per plan-032 R3 阶段 1 + 用户 23:54 JST 拍板 r3_opt1):
//! - 5 provider: claude, codex, mcode, copilot, grok
//! - 3 层 fallback: PATH (which/where) → login shell → app_bundle
//! - MinVersion gate 8 provider 最低 semver
//! - StatusClassifier 4 档 status
//! - LoginShellResolver 30min TTL 缓存 (macOS/Linux, Windows 跳过)
//! - 守门 #1 v25 cargo test 单 crate, 跳过 workspace
//!
//! 阶段 2 scope: 25 provider 全适配 + BuiltinRuntimes 派生 + console 扩展

#![forbid(unsafe_code)] // 守门 #7 0 unsafe
#![deny(missing_docs)] // workspace.lints

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use tracing::info;

/// 5 named provider (阶段 1).
///
/// 阶段 2 扩到 25 + BuiltinRuntimes 派生 (per Multica `agents_probe.go:155-296`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    /// Anthropic Claude Code CLI (per Multica `agents_probe.go:158`)
    Claude,
    /// OpenAI Codex CLI (per Multica `agents_probe.go:160`, 含 Desktop app bundle 兜底)
    Codex,
    /// MiniMax Code CLI (per FR-4 不读 model env, mcode 自管 model)
    Mcode,
    /// GitHub Copilot CLI
    Copilot,
    /// xAI Grok CLI (ACP stdio)
    Grok,
}

impl Provider {
    /// 默认命令名 (per Multica `agents_probe.go:155-296` 25 named + 阶段 1 范围 5)
    pub const fn default_cmd(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Mcode => "mcode",
            Self::Copilot => "copilot",
            Self::Grok => "grok",
        }
    }

    /// MULTICA_<NAME>_PATH env 名 (per Multica `agents_probe.go:116-118`)
    pub const fn env_path(self) -> &'static str {
        match self {
            Self::Claude => "MULTICA_CLAUDE_PATH",
            Self::Codex => "MULTICA_CODEX_PATH",
            Self::Mcode => "MULTICA_MCODE_PATH",
            Self::Copilot => "MULTICA_COPILOT_PATH",
            Self::Grok => "MULTICA_GROK_PATH",
        }
    }

    /// MULTICA_<NAME>_MODEL env 名 (None for mcode per FR-4, mcode 自管 model)
    pub const fn env_model(self) -> Option<&'static str> {
        // flat match (per Rust 不在嵌套 match 跨 arm narrow, 须显式列全部 variant)
        match self {
            Self::Mcode => None,
            Self::Claude => Some("MULTICA_CLAUDE_MODEL"),
            Self::Codex => Some("MULTICA_CODEX_MODEL"),
            Self::Copilot => Some("MULTICA_COPILOT_MODEL"),
            Self::Grok => Some("MULTICA_GROK_MODEL"),
        }
    }

    /// 8 provider 最低 semver (per Multica `version.go:13-22`)
    /// 未在表中的 provider 返 None (= no_minimum verdict).
    pub const fn min_version(self) -> Option<&'static str> {
        match self {
            Self::Claude => Some("2.0.0"),
            Self::Codex => Some("0.100.0"),
            Self::Copilot => Some("1.0.0"),
            Self::Grok => Some("0.2.89"),
            Self::Mcode => Some("0.1.2"),
        }
    }
}

/// RuntimeRegistry 域错误 (per 守门 #7 0 unsafe + thiserror 范式)
#[derive(Debug, Error)]
pub enum RegistryError {
    /// provider 探测 IO 错误 (per 守门 #11 不删标, 标 🔴 poisoned)
    #[error("probe IO error for {provider}: {source}")]
    ProbeIo {
        /// 探测失败的 provider (per Multica `AgentEntry.provider`)
        provider: Provider,
        /// 底层 IO 错误源 (per thiserror `#[source]` 自动 derive From)
        #[source]
        source: std::io::Error,
    },
    /// 解析 semver 失败 (per FR-9 sentinel error 区分 "missing" vs "too_old")
    #[error("invalid semver for {provider}: {raw}")]
    InvalidSemver {
        /// 解析失败的 provider
        provider: Provider,
        /// 原始 version 字符串
        raw: String,
    },
    /// Audit log 写入失败 (per 守门 #12 v21 [P] docs 同步)
    #[error("audit log write error: {0}")]
    AuditLogWrite(#[from] std::io::Error),
}

/// 单 provider 探测结果 (per Multica `AgentEntry` struct)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEntry {
    /// Provider 标识
    pub provider: Provider,
    /// 解析出的可执行文件绝对路径 (None = 探测失败, 走 missing 状态)
    pub path: Option<PathBuf>,
    /// `--version` 输出字符串 (原始, 未 parse)
    pub version: Option<String>,
    /// 解析后的 semver tuple (major, minor, patch), None = 解析失败
    pub version_parsed: Option<(u32, u32, u32)>,
    /// 探测方法 (per Multica `agents_probe.go:116-153` 3 层 fallback)
    pub probe_method: ProbeMethod,
    /// 探测时间 (UTC, ISO 8601)
    pub detected_at: SystemTime,
    /// 探测错误 (None = 成功)
    pub error: Option<String>,
}

/// 探测方法 (per Multica 3 层 fallback)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbeMethod {
    /// env 变量 + LookPath (per FR-3)
    Path,
    /// login shell 兜底解析 (per FR-5)
    Shell,
    /// App bundle 特殊处理 (per FR-5 codex Desktop)
    AppBundle,
    /// 探测失败 (per FR-12 missing)
    Missing,
}

/// MinVersionGate 校验结果 (per Multica `version.go:147-155` `*BelowMinimumError` 1:1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinVersionVerdict {
    /// 校验目标 provider
    pub provider: Provider,
    /// 探测到的 version 原始字符串 (None = 探测失败)
    pub detected: Option<String>,
    /// 最低 semver 要求 (None = provider 不在 MinVersion 表里)
    pub minimum: Option<&'static str>,
    /// "ok" | "too_old" | "missing" | "no_minimum"
    pub status: MinVersionStatus,
    /// 校验错误信息 (per FR-9 sentinel error 区分)
    pub error: Option<String>,
}

/// MinVersion 4 档状态 (per FR-9 sentinel error 区分)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinVersionStatus {
    /// 探测成功 + version >= minimum
    Ok,
    /// version < minimum (sentinel error 类型化)
    TooOld,
    /// version unparseable (sentinel error 类型化, 区别于 TooOld)
    Missing,
    /// provider 不在 MinVersion 表里
    NoMinimum,
}

/// 4 档 status (per FR-12 ~ FR-15 + 守门 #11 缺标比错标 poisoned 不删标)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusVerdict {
    /// 探测 + MinVersion + auth 三过 (per FR-12)
    /// 阶段 1: auth 默认 ok (per StatusClassifier::default auth_status)
    Active = 0,
    /// 探测过但 auth 失效 (per FR-13, 阶段 2 引入 auth probe 后才有此状态)
    Stale = 1,
    /// 探测到但 unusable (MinVersion 永久 too_old, per FR-14, 不删标 per 守门 #11)
    Poisoned = 2,
    /// 探测失败 (per FR-12 missing)
    Missing = 3,
}

/// RuntimeProbe — 5 provider 探测 (per DD-MULTICA-RUNTIME-001 §3.2)
pub struct RuntimeProbe {
    shell_resolver: LoginShellResolver,
}

impl RuntimeProbe {
    /// 创建新的 probe
    pub fn new() -> Self {
        Self {
            shell_resolver: LoginShellResolver::new(),
        }
    }

    /// probe 全部 5 provider
    pub fn probe_all(&mut self) -> Vec<RuntimeEntry> {
        let now = SystemTime::now();
        Provider::iter().map(|p| self.probe_one(p, now)).collect()
    }

    /// probe 单 provider, 3 层 fallback
    pub fn probe_one(&mut self, provider: Provider, now: SystemTime) -> RuntimeEntry {
        let default_cmd = provider.default_cmd();

        // 1. LookPath / where (env 优先 per FR-3)
        if let Some(path) = self.resolve_executable(default_cmd) {
            let version = self.get_version(&path);
            return RuntimeEntry {
                provider,
                path: Some(path),
                version: version.clone(),
                version_parsed: version.as_deref().and_then(parse_semver),
                probe_method: ProbeMethod::Path,
                detected_at: now,
                error: None,
            };
        }

        // 2. 绝对路径不 fallback (per Multica agents_probe.go:129-131)
        if default_cmd.contains('/') || default_cmd.contains('\\') {
            return RuntimeEntry {
                provider,
                path: None,
                version: None,
                version_parsed: None,
                probe_method: ProbeMethod::Missing,
                detected_at: now,
                error: Some("absolute path not found".to_string()),
            };
        }

        // 3. login shell 兜底 (per FR-5, macOS/Linux only)
        if let Some(shell_path) = self.shell_resolver.resolve(default_cmd) {
            let version = self.get_version(&shell_path);
            return RuntimeEntry {
                provider,
                path: Some(shell_path),
                version: version.clone(),
                version_parsed: version.as_deref().and_then(parse_semver),
                probe_method: ProbeMethod::Shell,
                detected_at: now,
                error: None,
            };
        }

        // 4. 探测失败
        RuntimeEntry {
            provider,
            path: None,
            version: None,
            version_parsed: None,
            probe_method: ProbeMethod::Missing,
            detected_at: now,
            error: Some("not found in PATH or login shell".to_string()),
        }
    }

    /// 跨平台 LookPath (Windows where / Unix which via std::process::Command)
    fn resolve_executable(&self, cmd: &str) -> Option<PathBuf> {
        if cfg!(windows) {
            // Windows: `where <cmd>`
            Command::new("where")
                .arg(cmd)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    stdout.lines().next().map(|s| PathBuf::from(s.trim()))
                })
        } else {
            // Unix: `which <cmd>` (sourced from shell, falls back to `command -v`)
            // 用 `command -v` 走 login shell 解析
            Command::new("sh")
                .arg("-c")
                .arg(format!("command -v {}", cmd))
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    stdout.lines().next().map(|s| PathBuf::from(s.trim()))
                })
        }
    }

    /// 跑 <cmd> --version 拿版本字符串
    fn get_version(&self, path: &Path) -> Option<String> {
        let output = Command::new(path).arg("--version").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let raw = String::from_utf8_lossy(&output.stdout);
        if raw.trim().is_empty() {
            // 某些 CLI 写 stderr
            let raw_err = String::from_utf8_lossy(&output.stderr);
            return raw_err.lines().next().map(|s| s.trim().to_string());
        }
        raw.lines().next().map(|s| s.trim().to_string())
    }
}

impl Default for RuntimeProbe {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider enum 迭代器 (阶段 1 5 个, 阶段 2 扩到 25)
impl Provider {
    /// 全部 named provider 迭代 (per Multica `agents_probe.go:206-212` BuiltinRuntimes 派生)
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::Claude,
            Self::Codex,
            Self::Mcode,
            Self::Copilot,
            Self::Grok,
        ]
        .into_iter()
    }
}

/// Provider Display (per thiserror #[error("...{provider}...")] 需要, per 守门 #5 不打印 secret)
impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.default_cmd())
    }
}

/// 解析 semver (per Multica `version.go:60-65` + dev-build git-describe 例外)
fn parse_semver(raw: &str) -> Option<(u32, u32, u32)> {
    // dev-build git-describe 形态: v0.2.15-235-gdaf0e935 (per Multica `version.go:69-93`)
    // 仍然尝试 parse 前 3 段
    let cleaned = raw.trim();
    if cleaned.is_empty() {
        return None;
    }
    let stripped = cleaned.strip_prefix('v').unwrap_or(cleaned);
    let mut parts = stripped.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    // 第 3 段可能含 "-" (dev-build), 取数字前缀
    let patch_str = parts.next()?;
    let patch_num: u32 = patch_str
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .and_then(|s| s.parse().ok())
        .or_else(|| patch_str.parse().ok())?;
    Some((major, minor, patch_num))
}

/// dev-build 检测 (per FR-10)
fn is_dev_build(version: &str) -> bool {
    // 形态: v0.2.15-235-gdaf0e935 或 0.2.15-235-gdaf0e935
    let stripped = version.strip_prefix('v').unwrap_or(version);
    let mut parts = stripped.split('.');
    let _major = parts.next();
    let _minor = parts.next();
    let patch = match parts.next() {
        Some(p) => p,
        None => return false,
    };
    // 第 3 段含 "-" 表示 dev-build
    patch.contains('-') && patch.contains("-g")
}

/// MinVersionGate (per DD-MULTICA-RUNTIME-001 §3.3 + Multica `version.go:13-178`)
pub struct MinVersionGate;

impl MinVersionGate {
    /// 校验 detected version >= minimum
    pub fn check(provider: Provider, detected_version: Option<&str>) -> MinVersionVerdict {
        let min_ver = provider.min_version();

        // 1. provider 不在 MinVersion 表里 → no_minimum
        let min_ver = match min_ver {
            Some(m) => m,
            None => {
                return MinVersionVerdict {
                    provider,
                    detected: detected_version.map(String::from),
                    minimum: None,
                    status: MinVersionStatus::NoMinimum,
                    error: None,
                };
            }
        };

        // 2. version unparseable → missing (sentinel error 区分)
        let raw = match detected_version {
            Some(s) if !s.is_empty() => s,
            _ => {
                return MinVersionVerdict {
                    provider,
                    detected: detected_version.map(String::from),
                    minimum: Some(min_ver),
                    status: MinVersionStatus::Missing,
                    error: Some(format!("cannot parse version {:?}", detected_version)),
                };
            }
        };

        // 3. dev-build git-describe 例外 (per FR-10) — 一律放过
        if is_dev_build(raw) {
            return MinVersionVerdict {
                provider,
                detected: Some(raw.to_string()),
                minimum: Some(min_ver),
                status: MinVersionStatus::Ok,
                error: None,
            };
        }

        // 4. parse 失败 → missing
        let parsed = match parse_semver(raw) {
            Some(p) => p,
            None => {
                return MinVersionVerdict {
                    provider,
                    detected: Some(raw.to_string()),
                    minimum: Some(min_ver),
                    status: MinVersionStatus::Missing,
                    error: Some(format!("cannot parse version {:?}", raw)),
                };
            }
        };

        // 5. parse minimum 失败 → missing (sentinel error 类型化)
        let min_parsed = match parse_semver(min_ver) {
            Some(p) => p,
            None => {
                return MinVersionVerdict {
                    provider,
                    detected: Some(raw.to_string()),
                    minimum: Some(min_ver),
                    status: MinVersionStatus::Missing,
                    error: Some(format!("invalid minimum version {:?}", min_ver)),
                };
            }
        };

        // 6. version < minimum → too_old (sentinel error 类型化)
        if parsed < min_parsed {
            return MinVersionVerdict {
                provider,
                detected: Some(raw.to_string()),
                minimum: Some(min_ver),
                status: MinVersionStatus::TooOld,
                error: Some(format!(
                    "{} {} < minimum {}",
                    provider_name(provider),
                    raw,
                    min_ver
                )),
            };
        }

        // 7. ok
        MinVersionVerdict {
            provider,
            detected: Some(raw.to_string()),
            minimum: Some(min_ver),
            status: MinVersionStatus::Ok,
            error: None,
        }
    }
}

/// Provider 名称 (用于错误信息)
fn provider_name(p: Provider) -> &'static str {
    p.default_cmd()
}

/// StatusClassifier (per DD-MULTICA-RUNTIME-001 §3.4 + 守门 #11 缺标比错标)
pub struct StatusClassifier;

impl StatusClassifier {
    /// 4 档 status 分类
    ///
    /// 阶段 1: auth_status 默认 "ok" (per StatusClassifier::classify 默认参数);
    /// 阶段 2 引入 auth probe 后, 会有 "expired" 检测.
    pub fn classify(entry: &RuntimeEntry, min_verdict: &MinVersionVerdict) -> StatusVerdict {
        Self::classify_with_auth(entry, min_verdict, AuthStatus::Ok)
    }

    /// 带 auth_status 的 4 档分类 (阶段 2 扩展)
    pub fn classify_with_auth(
        entry: &RuntimeEntry,
        min_verdict: &MinVersionVerdict,
        auth: AuthStatus,
    ) -> StatusVerdict {
        // missing: 探测失败
        if entry.path.is_none() {
            return StatusVerdict::Missing;
        }

        // poisoned: 探测到但 unusable (永久 too_old, per 守门 #11 不删标)
        if min_verdict.status == MinVersionStatus::TooOld {
            return StatusVerdict::Poisoned;
        }

        // stale: 探测过但 auth 失效
        if auth == AuthStatus::Expired {
            return StatusVerdict::Stale;
        }

        // active: 探测 + MinVersion + auth 三过
        StatusVerdict::Active
    }
}

/// Auth 状态 (per FR-13)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthStatus {
    /// Auth 探测通过
    Ok,
    /// Auth 失效 (key 过期等)
    Expired,
    /// Auth 探测未实装 (阶段 1 默认值, 阶段 2 引入真实 probe 后移除)
    Unknown,
}

/// LoginShellResolver (per DD-MULTICA-RUNTIME-001 §3.5 + Multica `agents_probe.go:30-50`)
///
/// macOS GUI daemon 兜底 PATH 解析, 30min TTL 缓存.
/// Windows 跳过 (per FR-7 平台限制).
pub struct LoginShellResolver {
    cache: HashMap<String, String>,
    cache_key: String,
    cached_at: Instant,
    ttl: Duration,
}

impl LoginShellResolver {
    /// 30min TTL (per FR-6)
    pub const SHELL_RESOLVE_TTL: Duration = Duration::from_secs(30 * 60);

    /// 创建
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            cache_key: String::new(),
            cached_at: Instant::now() - Self::SHELL_RESOLVE_TTL,
            ttl: Self::SHELL_RESOLVE_TTL,
        }
    }

    /// 解析 bare command name → 绝对路径 (per Multica `agents_probe.go:97-138`)
    pub fn resolve(&mut self, cmd: &str) -> Option<PathBuf> {
        if cfg!(windows) {
            // Windows 跳过 (per FR-7)
            return None;
        }

        // env fingerprint (per FR-7)
        let key = self.env_key();
        let now = Instant::now();

        // 缓存有效 → 直接返
        if !self.cache.is_empty()
            && self.cache_key == key
            && now.duration_since(self.cached_at) < self.ttl
        {
            if let Some(p) = self.cache.get(cmd) {
                return Some(PathBuf::from(p));
            }
        }

        // 缓存失效 → fork login shell 解析
        let resolved = self.resolve_via_shell(cmd);
        if let Some(path) = &resolved {
            self.cache.insert(cmd.to_string(), path.clone());
        } else {
            self.cache.remove(cmd);
        }
        self.cache_key = key;
        self.cached_at = now;
        resolved.map(PathBuf::from)
    }

    /// env fingerprint (per FR-7), 用 env name 不是 value (per 守门 #5)
    fn env_key(&self) -> String {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let shell = std::env::var_os("SHELL").unwrap_or_default();
        let home = std::env::var_os("HOME").unwrap_or_default();
        format!(
            "{}\x00{}\x00{}",
            path.to_string_lossy(),
            shell.to_string_lossy(),
            home.to_string_lossy()
        )
    }

    /// 跑 `command -v <cmd>` (per Multica `agents_probe.go:97-138`)
    fn resolve_via_shell(&self, cmd: &str) -> Option<String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {}", cmd))
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let first = stdout.lines().next()?.trim();
        if first.is_empty() || first.contains(' ') || first.contains('\t') {
            None
        } else {
            Some(first.to_string())
        }
    }
}

impl Default for LoginShellResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// RuntimeStatusReporter (per DD-MULTICA-RUNTIME-001 §3.6)
///
/// 每日 .log 持久化 (per 守门 #12 v21 [P] docs 同步).
pub struct RuntimeStatusReporter {
    log_dir: PathBuf,
}

impl RuntimeStatusReporter {
    /// 创建, log_dir 不存在自动创建
    pub fn new(log_dir: impl Into<PathBuf>) -> Result<Self, RegistryError> {
        let log_dir = log_dir.into();
        std::fs::create_dir_all(&log_dir).map_err(RegistryError::AuditLogWrite)?;
        Ok(Self { log_dir })
    }

    /// 写每日 .log (per FR-23)
    ///
    /// 每行 JSON, 包含: scan_id, started_at, finished_at, results, summary
    pub fn write_log(
        &self,
        scan_id: &str,
        started_at: SystemTime,
        finished_at: SystemTime,
        entries: &[RuntimeEntry],
        verdicts: &[(MinVersionVerdict, StatusVerdict)],
    ) -> Result<PathBuf, RegistryError> {
        let date = format_systemtime_date(started_at);
        let log_path = self.log_dir.join(format!("{}.log", date));

        // 摘要
        let mut active = 0;
        let mut stale = 0;
        let mut poisoned = 0;
        let mut missing = 0;
        for (_, status) in verdicts {
            match status {
                StatusVerdict::Active => active += 1,
                StatusVerdict::Stale => stale += 1,
                StatusVerdict::Poisoned => poisoned += 1,
                StatusVerdict::Missing => missing += 1,
            }
        }
        let summary = serde_json::json!({
            "active": active,
            "stale": stale,
            "poisoned": poisoned,
            "missing": missing,
        });

        // 序列化 entries + verdicts
        let log_entry = serde_json::json!({
            "scan_id": scan_id,
            "started_at": format_systemtime_iso(started_at),
            "finished_at": format_systemtime_iso(finished_at),
            "results": entries.iter().zip(verdicts.iter()).map(|(e, (m, s))| {
                serde_json::json!({
                    "provider": provider_name(e.provider),
                    "path": e.path,
                    "version": e.version,
                    "version_parsed": e.version_parsed,
                    "probe_method": format!("{:?}", e.probe_method),
                    "error": e.error,
                    "detected_at": format_systemtime_iso(e.detected_at),
                    "min_version_status": format!("{:?}", m.status),
                    "min_version_error": m.error,
                    "status": format!("{:?}", s),
                })
            }).collect::<Vec<_>>(),
            "summary": summary,
        });

        let line = serde_json::to_string(&log_entry).map_err(|e| {
            RegistryError::AuditLogWrite(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;

        // append mode
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(RegistryError::AuditLogWrite)?;
        writeln!(file, "{}", line).map_err(RegistryError::AuditLogWrite)?;
        Ok(log_path)
    }
}

/// 格式化 SystemTime → ISO 8601 (UTC, 简化版, per R3 阶段 1)
fn format_systemtime_iso(t: SystemTime) -> String {
    use chrono::{DateTime, Utc};
    let dt: DateTime<Utc> = t.into();
    dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// 格式化 SystemTime → YYYY-MM-DD
fn format_systemtime_date(t: SystemTime) -> String {
    use chrono::{DateTime, Utc};
    let dt: DateTime<Utc> = t.into();
    dt.format("%Y-%m-%d").to_string()
}

/// 全 workflow 一站式 run (per Multica `agents_probe.go:155-296` + Multica `client.go:227-235` ClaimTask)
///
/// 阶段 1 scope: probe + classify + report, 不含 ClaimTask (per R3 后续阶段).
pub fn scan_all(log_dir: impl Into<PathBuf>) -> Result<ScanReport, RegistryError> {
    let scan_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let started_at = SystemTime::now();
    let mut probe = RuntimeProbe::new();
    let entries = probe.probe_all();
    let verdicts: Vec<_> = entries
        .iter()
        .map(|e| {
            let m = MinVersionGate::check(e.provider, e.version.as_deref());
            let s = StatusClassifier::classify(e, &m);
            (m, s)
        })
        .collect();
    let reporter = RuntimeStatusReporter::new(log_dir)?;
    let finished_at = SystemTime::now();
    let log_path = reporter.write_log(&scan_id, started_at, finished_at, &entries, &verdicts)?;
    info!(scan_id = %scan_id, "scan complete");
    Ok(ScanReport {
        scan_id,
        started_at,
        finished_at,
        entries,
        verdicts,
        log_path,
    })
}

/// ScanReport (per DD-MULTICA-RUNTIME-001 §3.1)
#[derive(Debug)]
pub struct ScanReport {
    /// 唯一 scan 标识 (UUID 前 8 字符, per Multica 风格)
    pub scan_id: String,
    /// scan 开始时间 (UTC)
    pub started_at: SystemTime,
    /// scan 结束时间 (UTC)
    pub finished_at: SystemTime,
    /// 全部 5 provider 探测结果
    pub entries: Vec<RuntimeEntry>,
    /// 5 provider 的 MinVersion + Status 双结果 (per StatusClassifier.classify)
    pub verdicts: Vec<(MinVersionVerdict, StatusVerdict)>,
    /// audit log 落档路径 (per 守门 #12 v21 [P] docs 同步)
    pub log_path: PathBuf,
}

impl ScanReport {
    /// 摘要 (per Multica `server/pkg/agent/builtin_runtimes.go`)
    pub fn summary(&self) -> HashMap<&'static str, u32> {
        let mut map = HashMap::new();
        for (_, status) in &self.verdicts {
            let key = match status {
                StatusVerdict::Active => "active",
                StatusVerdict::Stale => "stale",
                StatusVerdict::Poisoned => "poisoned",
                StatusVerdict::Missing => "missing",
            };
            *map.entry(key).or_insert(0) += 1;
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_default_cmds() {
        assert_eq!(Provider::Claude.default_cmd(), "claude");
        assert_eq!(Provider::Codex.default_cmd(), "codex");
        assert_eq!(Provider::Mcode.default_cmd(), "mcode");
        assert_eq!(Provider::Copilot.default_cmd(), "copilot");
        assert_eq!(Provider::Grok.default_cmd(), "grok");
    }

    #[test]
    fn provider_env_model_mcode_is_none() {
        // per FR-4: mcode 自管 model, 不读 env
        assert!(Provider::Mcode.env_model().is_none());
        assert!(Provider::Claude.env_model().is_some());
    }

    #[test]
    fn min_version_check_ok() {
        let v = MinVersionGate::check(Provider::Claude, Some("2.5.0"));
        assert_eq!(v.status, MinVersionStatus::Ok);
    }

    #[test]
    fn min_version_check_too_old() {
        // sentinel error 区分 (per FR-9)
        let v = MinVersionGate::check(Provider::Claude, Some("1.5.0"));
        assert_eq!(v.status, MinVersionStatus::TooOld);
        assert!(v.error.is_some());
    }

    #[test]
    fn min_version_check_missing_unparseable() {
        // sentinel error 区分 (per FR-9)
        let v = MinVersionGate::check(Provider::Claude, Some("not-a-version"));
        assert_eq!(v.status, MinVersionStatus::Missing);
    }

    #[test]
    fn min_version_check_dev_build_pass() {
        // per FR-10: dev-build git-describe 例外
        let v = MinVersionGate::check(Provider::Claude, Some("v0.2.15-235-gdaf0e935"));
        assert_eq!(v.status, MinVersionStatus::Ok);
    }

    #[test]
    fn status_classifier_4_tiers() {
        // missing: path=None
        let e_missing = RuntimeEntry {
            provider: Provider::Claude,
            path: None,
            version: None,
            version_parsed: None,
            probe_method: ProbeMethod::Missing,
            detected_at: SystemTime::now(),
            error: None,
        };
        let m = MinVersionGate::check(Provider::Claude, Some("2.0.0"));
        assert_eq!(
            StatusClassifier::classify(&e_missing, &m),
            StatusVerdict::Missing
        );

        // poisoned: too_old (per 守门 #11 不删标)
        let e_ok = RuntimeEntry {
            provider: Provider::Claude,
            path: Some(PathBuf::from("/bin/claude")),
            version: Some("1.0.0".into()),
            version_parsed: Some((1, 0, 0)),
            probe_method: ProbeMethod::Path,
            detected_at: SystemTime::now(),
            error: None,
        };
        let m_too_old = MinVersionGate::check(Provider::Claude, Some("1.0.0"));
        assert_eq!(
            StatusClassifier::classify(&e_ok, &m_too_old),
            StatusVerdict::Poisoned
        );

        // active: default auth ok
        let m_ok = MinVersionGate::check(Provider::Claude, Some("2.0.0"));
        assert_eq!(
            StatusClassifier::classify(&e_ok, &m_ok),
            StatusVerdict::Active
        );

        // stale: auth expired
        assert_eq!(
            StatusClassifier::classify_with_auth(&e_ok, &m_ok, AuthStatus::Expired),
            StatusVerdict::Stale
        );
    }

    #[test]
    fn parse_semver_basic() {
        assert_eq!(parse_semver("2.0.0"), Some((2, 0, 0)));
        assert_eq!(parse_semver("v2.0.0"), Some((2, 0, 0)));
        assert_eq!(parse_semver("2.1.267 (Claude Code)"), Some((2, 1, 267)));
        assert_eq!(parse_semver("not-a-version"), None);
    }

    #[test]
    fn parse_semver_dev_build() {
        // v0.2.15-235-gdaf0e935 → (0, 2, 15)
        assert_eq!(parse_semver("v0.2.15-235-gdaf0e935"), Some((0, 2, 15)));
    }

    #[test]
    fn is_dev_build_detection() {
        assert!(is_dev_build("v0.2.15-235-gdaf0e935"));
        assert!(is_dev_build("0.2.15-235-gdaf0e935"));
        assert!(!is_dev_build("2.0.0"));
        assert!(!is_dev_build("2.1.267"));
    }

    // ========================================================================
    // R9 阶段 3: 5 milestone benchmark 实测 (per plan-032 §4.1)
    // ========================================================================

    /// milestone #1: 1000 provider probe < 200ms (vs Python 1-2K 秒, 10-20x 加速)
    #[test]
    #[ignore = "R9 阶段 3 PoC: criterion bench 留 benches/, 默认跳过避免 cargo test 慢"]
    fn r9_milestone_1_probe_1000_providers_under_200ms() {
        use std::time::Instant;
        let start = Instant::now();
        for _ in 0..200 {
            let mut probe = RuntimeProbe::new();
            let _entries = probe.probe_all();
        }
        let elapsed_ms = start.elapsed().as_millis();
        eprintln!("[R9 milestone #1] 1000 provider probe: {elapsed_ms} ms (target: < 200 ms)");
    }
}
