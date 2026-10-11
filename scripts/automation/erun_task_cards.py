"""Run bounded type and UI contract checks for the Engineering Run Task Cards slice."""

# CYPHER STRUCTURE MANIFEST
# CREATE (f:File {name:"scripts/automation/erun_task_cards.py",type:"file",language:"python"}),
#  (main:Function {name:"main",type:"function",signature:"main() -> int",visibility:"private",complexity:"moderate"}),
#  (run:Function {name:"run_step",type:"function",signature:"run_step(name: str, command: list[str], cwd: Path) -> dict[str, object]",visibility:"private",complexity:"moderate"}),
#  (parser:Variable {name:"parser",type:"variable"}),(args:Variable {name:"args",type:"variable"}),(results:Variable {name:"results",type:"variable"}),(commands:Variable {name:"commands",type:"variable"}),
#  (root:Variable {name:"ROOT",type:"variable"}),(output:Variable {name:"OUTPUT_DIR",type:"variable"}),(tests:Variable {name:"TEST_FILES",type:"variable"}),(timeout:Variable {name:"TIMEOUT_SECONDS",type:"variable"}),(desktopRoot:Variable {name:"desktop_frontend",type:"variable"}),
#  (subprocess:Function {name:"subprocess.run",type:"function"}),(open:Function {name:"Path.open",type:"function"}),(isDir:Function {name:"Path.is_dir",type:"function"}),(which:Function {name:"shutil.which",type:"function"}),(exit:Function {name:"sys.exit",type:"function"}),
#  (f)-[:CONTAINS]->(main),(f)-[:CONTAINS]->(run),(f)-[:CONTAINS]->(root),(f)-[:CONTAINS]->(output),(f)-[:CONTAINS]->(tests),(f)-[:CONTAINS]->(timeout),
#  (main)-[:USES]->(parser),(main)-[:USES]->(args),(main)-[:USES]->(results),(main)-[:USES]->(commands),(main)-[:USES]->(root),(main)-[:USES]->(output),(main)-[:USES]->(tests),(main)-[:USES]->(desktopRoot),
#  (main)-[:CALLS]->(run),(main)-[:CALLS]->(isDir),(main)-[:CALLS]->(which),(main)-[:CALLS]->(exit),(run)-[:CALLS]->(subprocess),(run)-[:CALLS]->(open),(run)-[:USES]->(output),(run)-[:USES]->(root),(run)-[:USES]->(timeout);

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUTPUT_DIR = ROOT / ".cache" / "engineering-run-task-cards"
TEST_FILES = (
    "src/lib/run/runDirectoryApi.test.ts",
    "src/components/run/RunTaskCardsPanel.test.tsx",
    "src/lib/workitem-guard.test.ts",
    "src/lib/agent-game/leveling.test.ts",
    "src/lib/agent-game/mapgen.test.ts",
    "src/lib/agent-game/movement.test.ts",
    "src/lib/agent-view/layout.test.ts",
    "src/lib/agent-view/selectors.test.ts",
    "src/components/agent-view/AgentCanvasView.test.tsx",
    "src/components/board/KanbanCard.test.tsx",
    "src/components/calendar/MonthView.test.tsx",
    "src/components/calendar/WeekView.test.tsx",
    "src/components/gantt/GanttChart.test.tsx",
)
TIMEOUT_SECONDS = 900


def run_step(name: str, command: list[str], cwd: Path) -> dict[str, object]:
    """Run one gate with file-backed logs so subprocess output stays out of memory."""
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    stdout_path = OUTPUT_DIR / f"{name}.stdout.log"
    stderr_path = OUTPUT_DIR / f"{name}.stderr.log"
    try:
        with stdout_path.open("w", encoding="utf-8") as stdout, stderr_path.open("w", encoding="utf-8") as stderr:
            result = subprocess.run(
                command,
                cwd=cwd,
                stdout=stdout,
                stderr=stderr,
                check=False,
                timeout=TIMEOUT_SECONDS,
            )
        status = "passed" if result.returncode == 0 else "failed"
        return {"name": name, "exit_code": result.returncode, "status": status, "command": command}
    except subprocess.TimeoutExpired:
        return {"name": name, "exit_code": 124, "status": "failed", "reason": f"exceeded {TIMEOUT_SECONDS}s", "command": command}
    except OSError as error:
        return {"name": name, "exit_code": 127, "status": "blocked", "reason": str(error), "command": command}


def main() -> int:
    parser = argparse.ArgumentParser(description="Run bounded Engineering Run Task Cards frontend checks.")
    parser.add_argument("--typecheck", action="store_true", help="Run the frontend TypeScript check; requires existing node_modules.")
    parser.add_argument("--tests", action="store_true", help="Run only the Run Directory and Task Cards Vitest files; requires existing node_modules.")
    parser.add_argument("--desktop-build", action="store_true", help="Build/typecheck the Tauri React frontend; requires its existing node_modules.")
    parser.add_argument("--desktop-tests", action="store_true", help="Run focused Tauri fail-closed hook/card tests; requires its existing node_modules.")
    args = parser.parse_args()
    if not any((args.typecheck, args.tests, args.desktop_build, args.desktop_tests)):
        parser.error("select at least one frontend or desktop gate")

    frontend = ROOT / "frontend"
    desktop_frontend = ROOT / "crates" / "star-desktop" / "frontend"
    npm = shutil.which("npm.cmd" if os.name == "nt" else "npm")
    commands = (
        (args.typecheck, "typecheck", [npm or "npm", "run", "typecheck"], frontend),
        (args.tests, "vitest", [npm or "npm", "run", "test", "--", *TEST_FILES], frontend),
        (args.desktop_build, "desktop-build", [npm or "npm", "run", "build"], desktop_frontend),
        (args.desktop_tests, "desktop-tests", [npm or "npm", "run", "test", "--", "tests/useTauriWorkItems.test.ts", "tests/KanbanCard.test.tsx"], desktop_frontend),
        (args.desktop_tests, "desktop-board-fail-closed", [npm or "npm", "run", "test", "--", "tests/BoardView.test.tsx", "-t", "shows error state on Tauri IPC failure"], desktop_frontend),
    )
    results: list[dict[str, object]] = []
    for enabled, name, command, cwd in commands:
        if not enabled:
            continue
        if not npm:
            results.append({"name": name, "status": "blocked", "reason": "npm unavailable"})
        elif not (cwd / "node_modules").is_dir():
            results.append({"name": name, "status": "blocked", "reason": f"{cwd.relative_to(ROOT)}/node_modules is absent; install dependencies explicitly before rerunning"})
        else:
            results.append(run_step(name, command, cwd))

    statuses = {row["status"] for row in results}
    status = "failed" if "failed" in statuses else "blocked" if "blocked" in statuses else "passed"
    summary = {"status": status, "gates": results}
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUTPUT_DIR / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary, ensure_ascii=False))
    return 0 if status == "passed" else 2 if status == "blocked" else 1


if __name__ == "__main__":
    sys.exit(main())
