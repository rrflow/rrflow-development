#!/usr/bin/env python3
"""Recheck each published result: is the number it quotes still at its source?

It only reads, and it changes no reference: updating one is a deliberate change to a signed
definition. It reaches the network only when run by hand. A number found at the source shows it is
there, and not that a row attributes it correctly, so read a source before changing its row.

Usage: check_references.py [--folder references] [--all]
"""
import argparse
import json
import re
import urllib.error
import urllib.request
from pathlib import Path


def fetchable(url):
    """Where a source serves its own text, in place of a page that renders it."""
    match = re.match(r"https://github\.com/([^/]+/[^/]+)/blob/([^/]+)/(.*)", url)
    if match:
        return f"https://raw.githubusercontent.com/{match[1]}/{match[2]}/{match[3]}"
    match = re.match(r"https://docs\.google\.com/spreadsheets/d/([^/]+)/edit", url)
    if match:
        return f"https://docs.google.com/spreadsheets/d/{match[1]}/export?format=csv"
    match = re.match(r"https://arxiv\.org/pdf/(.*)", url)
    if match:
        return f"https://arxiv.org/html/{match[1]}"
    return url


def fetch(url, cache):
    target = fetchable(url)
    if target not in cache:
        request = urllib.request.Request(target, headers={"User-Agent": "rrflow-lab-check-references"})
        with urllib.request.urlopen(request, timeout=60) as response:
            cache[target] = response.read().decode("utf-8", "replace").replace(",", "")
    return cache[target]


def numbers(quote):
    """Numbers a reader would check: at least three digits, from the row's own quote."""
    found = re.findall(r"\d[\d,]*\.?\d*", quote)
    return [item.replace(",", "") for item in found if len(re.sub(r"\D", "", item)) >= 3]


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--folder", type=Path, default=Path(__file__).with_name("references"))
    parser.add_argument("--all", action="store_true", help="also print rows that still check")
    arguments = parser.parse_args()
    cache = {}
    checked = moved = unknown = 0
    for path in sorted(arguments.folder.glob("*.json")):
        reference = json.loads(path.read_text(encoding="utf-8"))
        for row in reference.get("seek", []) + reference.get("answers", []):
            name = f"{reference['source']['id']}: {row['system']}"
            url = row["origin"]["url"]
            wanted = numbers(row["origin"]["quote"])
            if url.endswith(".parquet"):
                unknown += 1
                print(f"unknown  {name}: {url} is not text, so read it by hand")
                continue
            try:
                body = fetch(url, cache)
            except (urllib.error.URLError, TimeoutError, UnicodeError) as error:
                unknown += 1
                print(f"unknown  {name}: could not read {url}: {error}")
                continue
            missing = [item for item in wanted if item not in body]
            checked += 1
            if missing:
                moved += 1
                print(f"MOVED    {name}: no longer found: {' '.join(missing[:6])}, at {url}")
            elif arguments.all:
                print(f"current  {name}")
    print(f"{checked} rows read, {moved} moved, {unknown} could not be checked")
    return 1 if moved else 0


if __name__ == "__main__":
    raise SystemExit(main())
