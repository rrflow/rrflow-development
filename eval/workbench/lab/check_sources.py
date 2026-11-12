#!/usr/bin/env python3
"""Report which outside sources have moved past what sources.json pinned.

It only reads, and it changes no pin: adopting a newer version is a deliberate change to a
manifest, because it starts a new comparison. It reaches the network only when run by hand.
Set GITHUB_TOKEN for a higher GitHub rate limit.

Usage: check_sources.py [--file sources.json] [--all]
"""
import argparse
import json
import os
import urllib.error
import urllib.request
from pathlib import Path


def fetch(url):
    headers = {"User-Agent": "rrflow-lab-check-sources", "Accept": "application/json"}
    if "api.github.com" in url and os.environ.get("GITHUB_TOKEN"):
        headers["Authorization"] = f"Bearer {os.environ['GITHUB_TOKEN']}"
    request = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def newest(track):
    kind = track["from"]
    if kind == "github_release":
        return fetch(f"https://api.github.com/repos/{track['repository']}/releases/latest")["tag_name"]
    if kind == "github_commit":
        return fetch(f"https://api.github.com/repos/{track['repository']}/commits/HEAD")["sha"]
    if kind == "pypi":
        return fetch(f"https://pypi.org/pypi/{track['package']}/json")["info"]["version"]
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--file", type=Path, default=Path(__file__).with_name("sources.json"))
    parser.add_argument("--all", action="store_true", help="also print sources that have not moved")
    arguments = parser.parse_args()
    sources = json.loads(arguments.file.read_text(encoding="utf-8"))["sources"]
    moved = 0
    for source in sources:
        try:
            latest = newest(source["track"])
        except (urllib.error.URLError, KeyError, TimeoutError) as error:
            print(f"unknown  {source['id']}: could not check: {error}")
            continue
        if latest is None:
            if arguments.all:
                print(f"manual   {source['id']}: checked by hand on {source['checked_on']}")
        elif latest != source["pinned"]:
            moved += 1
            print(f"moved    {source['id']}: pinned {source['pinned']}, newest {latest} ({source['status']})")
        elif arguments.all:
            print(f"current  {source['id']}: {latest}")
    print(f"{moved} of {len(sources)} sources have moved past their pin")


if __name__ == "__main__":
    main()
