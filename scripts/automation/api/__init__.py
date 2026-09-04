"""scripts/automation/api — FastAPI routes for TMO + agent automation

约束 (per 守门 #22 + 守门 #24):
  - 子包独立, 不污染主仓编译链
  - 浏览器 → Next.js → FastAPI 8080 console_server.py → subprocess 路径
  - routes_tmo 8 端点 (per 02 §4 API endpoint 表):
      /api/tmo/merge /api/tmo/split /api/tmo/dependencies /api/tmo/bulk
      /api/tmo/summarize /api/tmo/reassign /api/tmo/metadata
      /api/tmo/relationships GET

注: 跨 TMO-01..TMO-07 worktree, 各 owner 各自 mount 自己端点。
本包 (wt-tmo-04) 仅含 /api/tmo/bulk (per 守门分工); 其他端点由其他 wt mount。
merge 跨 worktree 时 namespace 不冲突 (FastAPI mount), per G-TMO-07。
"""

__version__ = "0.4.0-tmo04"
__all__ = ["routes_tmo"]
