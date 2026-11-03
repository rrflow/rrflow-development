#!/usr/bin/env python3
"""Turn merged packages of a repository into planning tasks with known answers.

Each merged pull request gives a task: the behavior its brief asked for, at the revision it
started from, with the files it really changed as the answer. Paths are removed from the
task's text, so the answer has to be found by reading. Generated files, journals and indexes
are left out of the answer, because no one plans those by hand.

It reads through the `gh` command and writes a manifest with no models and no spend limit, so
nothing can run from it until someone chooses them. It is deterministic for one list of pull
requests. A task that cannot be stated without its answer is skipped and counted.

Usage: extract_package_tasks.py --repository OWNER/NAME --count N --out FILE
"""
import argparse
import json
import random
import re
import subprocess
import sys
from pathlib import Path

GENERATED = re.compile(
    r"^(docs/evidence/change-journals/|docs/roadmap/rrflow-1\.0-file-plan\.jsonl$|eval/schemas/"
    r"|docs/.*/README\.md$|.*\.lock$|.*/generated/|sdks?/.*/generated)"
)
PATH_LIKE = re.compile(r"`[^`\s]*(?:/[^`\s]*|\.[a-z]{1,5})`")
BARE_PATH = re.compile(r"(?<![\w/.-])(?:[\w.-]+/)+[\w.-]+\.[a-z]{1,5}(?![\w/-])")
RECORD = re.compile(r"\b(?:OBJ-\d{2}|POAM-\d{3}|[A-J]-\d{2}[a-z]?\d*)\b")
BEHAVIOR_HEADING = re.compile(r"current behavior[^\n]*\n", re.IGNORECASE)
# A brief's scope list names where the change goes, which is the answer, so a task stops before it.
END_OF_BEHAVIOR = re.compile(
    r"\n[-*\s]*(?:exact files|exact scope|scope\b|target files|## |for replacement|owner records"
    r"|revised|files and symbols|outside\b|unchanged\b|stop conditions)",
    re.IGNORECASE,
)
MIN_PROMPT_CHARACTERS = 80


def gh(*arguments):
    result = subprocess.run(["gh", *arguments], capture_output=True, text=True, encoding="utf-8", check=True)
    return json.loads(result.stdout)


def behavior_section(body):
    heading = BEHAVIOR_HEADING.search(body)
    if heading is None:
        return None
    rest = body[heading.end() :]
    end = END_OF_BEHAVIOR.search(rest)
    return (rest[: end.start()] if end else rest).strip()


def answer_files(files):
    return sorted(entry["path"] for entry in files if not GENERATED.match(entry["path"]))


def scrub(text, answer):
    text = PATH_LIKE.sub("[a path]", text)
    text = BARE_PATH.sub("[a path]", text)
    for path in sorted(answer, key=len, reverse=True):
        text = text.replace(path, "[a path]")
        name = path.rsplit("/", 1)[-1]
        text = re.sub(re.escape(name), "[a file]", text)
    return text


def records_from(body):
    """The records a brief names ahead of its behavior section, where it says who owns the work."""
    heading = BEHAVIOR_HEADING.search(body)
    return sorted(set(RECORD.findall(body[: heading.start()]))) if heading else []


def task_from(pull_request, repository):
    detail = gh(
        "pr", "view", str(pull_request["number"]), "--repo", repository,
        "--json", "number,title,body,baseRefOid,files",
    )
    answer = answer_files(detail["files"])
    if not 2 <= len(answer) <= 25:
        return None, "answer size"
    behavior = behavior_section(detail["body"] or "")
    if behavior is None:
        return None, "no behavior section"
    prompt = scrub(behavior, answer)
    if len(prompt) < MIN_PROMPT_CHARACTERS:
        return None, "too short"
    if any(path in prompt or path.rsplit("/", 1)[-1] in prompt for path in answer):
        return None, "cannot hide its answer"
    return (
        {
            "id": f"pr-{detail['number']}",
            "title": detail["title"][:200],
            "repository": f"https://github.com/{repository}",
            "revision": detail["baseRefOid"],
            "prompt": prompt,
            "expected_files": answer,
            "expected_records": records_from(detail["body"] or ""),
            "source": f"https://github.com/{repository}/pull/{detail['number']}",
        },
        None,
    )


NOT_YET = "Attunement phases past connect fail with phase-unavailable, inventory stores paths and digests but no text, and POAM-001's knowledge import is not done, so RRFlow cannot return a repository's content through its own path."


def arms():
    read_only = ["Read", "Grep", "Glob"]
    def gap(arm_id, label, state, reason):
        return {
            "id": arm_id,
            "label": label,
            "availability": {"state": state, "reason": reason},
            "repository_access": True,
        }
    return [
        {
            "id": "no-repository",
            "label": "The model alone, with no repository",
            "availability": {"state": "available"},
            "repository_access": False,
            "limits": ["This arm is the floor. If it solves tasks, they test what a model remembers and not what it can find."],
        },
        {
            "id": "model-alone",
            "label": "The model with read-only tools on the checkout",
            "availability": {"state": "available"},
            "repository_access": True,
            "tools": read_only,
        },
        gap(
            "rrflow-harness-import",
            "The model with RRFlow, over text a harness imported",
            "unimplemented",
            "The importer and tool path are not yet proven on a real engine. This arm would measure BM25 and graph retrieval over text the harness chunked, and would not measure attunement, parsing or embeddings.",
        ),
        gap("rrflow-attuned", "The model with RRFlow, over an attuned project", "unimplemented", NOT_YET),
        gap(
            "rrflow-routed",
            "The model with RRFlow choosing its capability for each request",
            "unimplemented",
            "The owner's direction is that capabilities, including Qdrant's through a knowledge path, are chosen per request so different reasoning can be triggered dynamically. No operation does this yet.",
        ),
        gap(
            "rea",
            "The model with REA",
            "not_applicable",
            "REA studies binaries. These tasks are on a repository whose source can be read, so it belongs to a known-answer suite instead.",
        ),
        gap(
            "leanrea",
            "The model with the leanrea skill",
            "not_applicable",
            "leanrea studies binaries. These tasks are on a repository whose source can be read, so it belongs to a known-answer suite instead.",
        ),
    ]


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--repository", required=True)
    parser.add_argument("--count", type=int, default=30)
    parser.add_argument("--seed", type=int, default=20261103)
    parser.add_argument("--list-limit", type=int, default=300)
    parser.add_argument("--out", type=Path, required=True)
    arguments = parser.parse_args()
    merged = gh(
        "pr", "list", "--repo", arguments.repository, "--state", "merged",
        "--limit", str(arguments.list_limit), "--json", "number",
    )
    merged.sort(key=lambda entry: entry["number"])
    random.Random(arguments.seed).shuffle(merged)
    tasks, skipped = [], {}
    for pull_request in merged:
        if len(tasks) >= arguments.count:
            break
        task, reason = task_from(pull_request, arguments.repository)
        if task is None:
            skipped[reason] = skipped.get(reason, 0) + 1
        else:
            tasks.append(task)
    tasks.sort(key=lambda task: int(task["id"].removeprefix("pr-")))
    manifest = {
        "schema_version": 1,
        "test": {
            "id": "rrflow-package-planning",
            "title": "Planning real changes to this repository",
            "family": "agent-tasks",
            "revision": 1,
        },
        "preregistration": {
            "question": "Given the same task at the same revision, does an arm name the files a real change needed more often than the model with read-only tools, and at what cost?",
            "baseline_arm": "model-alone",
            "floor_arm": "no-repository",
            "repetitions": 3,
            "order_seed": arguments.seed,
            "alpha": 0.05,
            "min_file_recall": 0.6,
            "min_file_precision": 0.5,
            "max_floor_solve_rate": 0.1,
        },
        "arms": arms(),
        "tasks": tasks,
    }
    arguments.out.parent.mkdir(parents=True, exist_ok=True)
    with arguments.out.open("w", encoding="utf-8", newline="\n") as output:
        json.dump(manifest, output, indent=2)
        output.write("\n")
    print(f"{len(tasks)} tasks written, skipped: {json.dumps(skipped, sort_keys=True)}")


if __name__ == "__main__":
    main()
