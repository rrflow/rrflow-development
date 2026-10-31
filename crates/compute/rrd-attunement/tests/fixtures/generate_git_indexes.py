"""Developer-only Git input fixture generation; never a runtime dependency.

Run explicitly from any directory. Scratch repositories are retained for audit.
All inputs below are authored parser correctness cases, not learning records.
"""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    output = Path(__file__).resolve().parent / "git-index"
    output.mkdir(exist_ok=True)
    scratch = Path(tempfile.mkdtemp(prefix="rrflow-git-index-")).resolve()
    env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
    commands = []

    def git(repo, *args, data=None):
        commands.append(list(args))
        return subprocess.run(
            ["git", "-C", str(repo), *args], input=data, env=env,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True,
        ).stdout

    receipts = []

    def capture(repo, name):
        content = (repo / ".git" / "index").read_bytes()
        (output / name).write_bytes(content)
        receipts.append({"file": name, "bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()})

    for algorithm in ("sha1", "sha256"):
        repo = scratch / algorithm
        repo.mkdir()
        git(repo, "init", f"--object-format={algorithm}")
        oid = git(repo, "hash-object", "-w", "--stdin", data=b"fixture content\n").strip().decode("ascii")
        # Deliberately includes ignored parents, hidden config, executable and
        # symbolic-link modes. Index modes do not authorize reading the target.
        entries = [
            (".github/workflows/check.yml", "100644"),
            ("ignored/tracked.txt", "100644"),
            ("link", "120000"),
            ("src/main.rs", "100755"),
        ]
        data = "".join(f"{mode} {oid}\t{path}\n" for path, mode in entries).encode("utf-8")
        git(repo, "update-index", "--index-info", data=data)
        git(repo, "update-index", "--index-version=2")
        capture(repo, f"{algorithm}-v2.index")
        if algorithm == "sha1":
            git(repo, "update-index", "--skip-worktree", "src/main.rs")
            git(repo, "update-index", "--index-version=3")
            capture(repo, "sha1-v3.index")
            git(repo, "update-index", "--index-version=4")
            capture(repo, "sha1-v4.index")
            git(repo, "update-index", "--split-index")
            capture(repo, "sha1-split.index")
    repo = scratch / "ignore"
    repo.mkdir()
    git(repo, "init", "--object-format=sha1")
    sources = [
        {"kind": "global", "patterns": "*.tmp\n*.log\n"},
        {"kind": "repository", "patterns": "!*.log\n"},
        {"kind": "worktree", "directory": "", "patterns": "*.log\n!root.log\nignored/\n!ignored/keep.txt\nout/*\n!out/keep/\nout/keep/*\n!out/keep/file\n\\#literal\n\\!literal\n**/cache/*.tmp\n/root-only\nfolder/\n"},
        {"kind": "worktree", "directory": "src", "patterns": "!keep.log\n*.TMP\n"},
        {"kind": "worktree", "directory": "ignored", "patterns": "!keep.txt\n"},
    ]
    global_ignore = scratch / "global-ignore"
    for source in sources:
        if source["kind"] == "global":
            destination = global_ignore
        elif source["kind"] == "repository":
            destination = repo / ".git" / "info" / "exclude"
        else:
            destination = repo / source["directory"] / ".gitignore"
        destination.parent.mkdir(exist_ok=True, parents=True)
        destination.write_text(source["patterns"], encoding="utf-8", newline="\n")
    paths = [
        ("root.log", False), ("other.log", False), ("src/keep.log", False),
        ("src/other.log", False), ("other.tmp", False), ("src/a.TMP", False),
        ("src/a.tmp", False), ("ignored/keep.txt", False), ("ignored/other", False),
        ("out/keep", True), ("out/keep/file", False), ("out/keep/other", False),
        ("out/drop/file", False), ("#literal", False), ("!literal", False),
        ("a/b/cache/a.tmp", False), ("cache/a.tmp", False), ("cache/a.txt", False),
        ("root-only", False), ("a/root-only", False), ("folder", True),
        ("folder/child", False), (".github/workflows/check.yml", False),
    ]
    for path, is_directory in paths:
        destination = repo / path
        destination.parent.mkdir(exist_ok=True, parents=True)
        if is_directory:
            destination.mkdir(exist_ok=True)
        else:
            destination.write_bytes(b"authored ignore oracle input\n")
    cases = []
    for ignore_case in (False, True):
        for path, is_directory in paths:
            args = ["-c", f"core.excludesFile={global_ignore}", "-c",
                    f"core.ignoreCase={str(ignore_case).lower()}", "check-ignore", "--no-index", "-q", "--", path]
            result = subprocess.run(["git", "-C", str(repo), *args], env=env, capture_output=True)
            if result.returncode not in (0, 1):
                raise RuntimeError(result.stderr.decode("utf-8", errors="replace"))
            cases.append({"path": path, "is_directory": is_directory,
                          "ignore_case": ignore_case, "ignored": result.returncode == 0})
    oracle = {"purpose": "Observed git check-ignore decisions on authored parser cases; not learning records.",
              "sources": sources, "cases": cases,
              "command": "git -c core.excludesFile=<explicit fixture> -c core.ignoreCase=<case> check-ignore --no-index -q -- <path>"}
    oracle_bytes = (json.dumps(oracle, indent=2) + "\n").encode("utf-8")
    (output / "ignore-oracle.json").write_bytes(oracle_bytes)
    receipts.append({"file": "ignore-oracle.json", "bytes": len(oracle_bytes),
                     "sha256": hashlib.sha256(oracle_bytes).hexdigest()})
    repo = scratch / "compressed"
    repo.mkdir()
    git(repo, "init", "--object-format=sha1")
    oid = git(repo, "hash-object", "-w", "--stdin", data=b"compressed path fixture\n").strip().decode("ascii")
    entries = "".join(f"100644 {oid}\t{'p' * 128}/{number:03}.rs\n" for number in range(128))
    git(repo, "update-index", "--index-info", data=entries.encode("ascii"))
    git(repo, "update-index", "--index-version=4")
    capture(repo, "sha1-compressed-v4.index")
    manifest = {
        "purpose": "Authored parser correctness fixtures; not training or routing evaluation data.",
        "generator": "generate_git_indexes.py",
        "git_version": subprocess.check_output(["git", "--version"], env=env, text=True).strip(),
        "commands": commands,
        "files": receipts,
    }
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"scratch": str(scratch), "files": receipts}))


if __name__ == "__main__":
    main()
