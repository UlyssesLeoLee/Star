//! `star submit` — Universal Submit 占位实现
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/flows/05-universal-submit.md` 11/12 步流程,
//! Phase D 骨架只暴露 stub:
//! - 打印 `submit: not implemented yet` 到 stdout
//! - 退出码 0(per 任务完成标准)
//!
//! 完整 11/12 步(检查 completion gate / 跑测试 / 跑 pipeline / 创建 MR / 等 review / 等 merge / 归档)
//! 待 Phase D.1 增量补齐。

#![warn(missing_docs)]

use clap::Args;

use crate::error::StarError;

/// `star submit` 参数(Phase D 全部 stub,留 `--dry-run` 作未来 hook)
#[derive(Debug, Args)]
pub(crate) struct SubmitArgs {
    /// Dry run(Phase D 永远 stub,仅留作未来 hook 标记)
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

/// Stub 入口
pub(crate) fn run(args: SubmitArgs) -> Result<(), StarError> {
    // Phase D:仅打印 + 返回 OK
    println!("submit: not implemented yet (dry_run={})", args.dry_run);
    Ok(())
}
