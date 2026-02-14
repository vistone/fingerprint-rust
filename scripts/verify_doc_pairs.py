#!/usr/bin/env python3
"""
Documentation policy checks for bilingual doc pairs and root placement.
"""

from __future__ import annotations

import os
import subprocess
import sys
from typing import Dict, List, Set, Tuple


def run_git(args: List[str], repo_root: str) -> str:
    result = subprocess.run(
        args,
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        sys.stderr.write("Git command failed: {}\n".format(" ".join(args)))
        if result.stderr:
            sys.stderr.write(result.stderr)
        sys.exit(1)
    return result.stdout


def parse_name_status_z(data: str) -> List[Tuple[str, str]]:
    tokens = data.split("\0")
    items: List[Tuple[str, str]] = []
    i = 0
    while i < len(tokens):
        status = tokens[i]
        if not status:
            break
        if status.startswith("R") or status.startswith("C"):
            if i + 2 >= len(tokens):
                break
            new_path = tokens[i + 2]
            items.append((status[0], new_path))
            i += 3
        else:
            if i + 1 >= len(tokens):
                break
            path = tokens[i + 1]
            items.append((status[0], path))
            i += 2
    return items


def gather_changes(repo_root: str) -> Dict[str, Set[str]]:
    items: List[Tuple[str, str]] = []
    items += parse_name_status_z(
        run_git(["git", "diff", "--name-status", "-z", "--cached"], repo_root)
    )
    items += parse_name_status_z(
        run_git(["git", "diff", "--name-status", "-z"], repo_root)
    )

    untracked = run_git(
        ["git", "ls-files", "--others", "-z", "--exclude-standard"], repo_root
    )
    if untracked:
        for path in untracked.split("\0"):
            if path:
                items.append(("?", path))

    changes: Dict[str, Set[str]] = {}
    for status, path in items:
        changes.setdefault(path, set()).add(status)
    return changes


def is_doc_md(path: str) -> bool:
    return path.endswith(".md")


def is_root_file(path: str) -> bool:
    return "/" not in path


def main() -> int:
    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    changes = gather_changes(repo_root)
    errors: List[str] = []

    for path, statuses in changes.items():
        if is_root_file(path) and ("A" in statuses or "?" in statuses):
            errors.append("New root-level file not allowed: {}".format(path))

    for path, statuses in changes.items():
        if ("A" in statuses or "?" in statuses) and path.startswith("docs/"):
            if is_doc_md(path) and not (
                path.startswith("docs/en/") or path.startswith("docs/zh/")
            ):
                errors.append(
                    "New doc must live under docs/en and docs/zh, not {}".format(path)
                )

    for path, statuses in changes.items():
        if not is_doc_md(path):
            continue
        if path.startswith("docs/en/"):
            counterpart = "docs/zh/" + path[len("docs/en/") :]
        elif path.startswith("docs/zh/"):
            counterpart = "docs/en/" + path[len("docs/zh/") :]
        else:
            continue

        if "D" in statuses:
            if counterpart not in changes or "D" not in changes.get(counterpart, set()):
                errors.append(
                    "Delete both language docs together: {} -> {}".format(
                        path, counterpart
                    )
                )
        else:
            if counterpart not in changes:
                errors.append(
                    "Update both language docs together: {} -> {}".format(
                        path, counterpart
                    )
                )

    if errors:
        print("Documentation policy checks failed:")
        for err in errors:
            print("- " + err)
        print("\nSee docs/en/PROJECT_POLICIES.md for details.")
        return 1

    print("Documentation policy checks passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
