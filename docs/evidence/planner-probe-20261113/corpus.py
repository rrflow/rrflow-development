"""Build one code corpus from a git checkout for the Arango/Surreal/RRFlow probe.

Records: file, chunk (60-line windows), symbol (regex definitions), and edges:
defines (file->symbol), references (enclosing symbol -> referenced symbol),
depends (crate -> crate from Cargo.toml). Vectors are a deterministic 64-d
feature hash of chunk tokens, used only to observe planner behaviour, not quality.
"""
import hashlib, json, math, re, subprocess, sys
from pathlib import Path

root = Path(sys.argv[1]); out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
files = [p for p in subprocess.check_output(["git", "-C", str(root), "ls-files"], text=True).split("\n")
         if p.endswith((".rs", ".py", ".go"))]
DEF = {
    ".rs": re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(fn|struct|enum|trait|type|const|mod)\s+([A-Za-z_][A-Za-z0-9_]*)"),
    ".py": re.compile(r"^\s*(def|class)\s+([A-Za-z_][A-Za-z0-9_]*)"),
    ".go": re.compile(r"^\s*(func|type)\s+(?:\([^)]*\)\s*)?([A-Za-z_][A-Za-z0-9_]*)"),
}
TOKEN = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
DIM = 64

def key(s): return hashlib.sha1(s.encode()).hexdigest()[:16]

def embed(text):
    v = [0.0] * DIM
    for t in TOKEN.findall(text.lower()):
        h = int(hashlib.md5(t.encode()).hexdigest(), 16)
        v[h % DIM] += 1.0 if (h >> 8) & 1 else -1.0
    n = math.sqrt(sum(x * x for x in v)) or 1.0
    return [round(x / n, 6) for x in v]

def crate_of(path):
    parts = path.split("/")
    return parts[2] if len(parts) > 3 and parts[0] == "crates" else parts[0]

F, C, S, defines = [], [], [], []
lines_by_file = {}
for path in files:
    try:
        text = (root / path).read_text(encoding="utf-8")
    except (UnicodeDecodeError, FileNotFoundError):
        continue
    lines = text.split("\n"); lines_by_file[path] = lines
    ext = Path(path).suffix
    F.append({"_key": key(path), "path": path, "lang": ext[1:], "crate": crate_of(path),
              "bytes": len(text.encode()), "lines": len(lines), "test": "/tests/" in path or path.endswith("_test.go") or "/test_" in path})
    for i in range(0, len(lines), 60):
        body = "\n".join(lines[i:i + 60])
        C.append({"_key": key(f"{path}:{i+1}"), "path": path, "crate": crate_of(path), "start": i + 1,
                  "end": min(i + 60, len(lines)), "text": body[:6000], "vec": embed(body)})
    for n, line in enumerate(lines, 1):
        m = DEF[ext].match(line)
        if m:
            sk = key(f"{path}:{n}:{m.group(2)}")
            S.append({"_key": sk, "name": m.group(2), "kind": m.group(1), "path": path, "crate": crate_of(path), "line": n})
            defines.append({"_from": f"file/{key(path)}", "_to": f"symbol/{sk}", "kind": "defines"})

by_name = {}
for s in S:
    by_name.setdefault(s["name"], []).append(s)
refs, seen = [], set()
for path, lines in lines_by_file.items():
    owned = sorted((s for s in S if s["path"] == path), key=lambda s: s["line"])
    if not owned:
        continue
    j = 0
    for n, line in enumerate(lines, 1):
        while j + 1 < len(owned) and owned[j + 1]["line"] <= n:
            j += 1
        if owned[j]["line"] > n:
            continue
        src = owned[j]
        for t in set(TOKEN.findall(line)):
            targets = by_name.get(t)
            if not targets or len(t) < 6 or len(targets) > 3 or t == src["name"]:
                continue
            for tgt in targets:
                k = (src["_key"], tgt["_key"])
                if k not in seen:
                    seen.add(k)
                    refs.append({"_from": f"symbol/{src['_key']}", "_to": f"symbol/{tgt['_key']}", "kind": "references"})

crates, deps = {}, []
for toml in root.glob("crates/*/*/Cargo.toml"):
    name = re.search(r'^name\s*=\s*"([^"]+)"', toml.read_text(), re.M)
    if name:
        crates[name.group(1)] = toml.parent.name
for toml in root.glob("crates/*/*/Cargo.toml"):
    text = toml.read_text(); me = re.search(r'^name\s*=\s*"([^"]+)"', text, re.M).group(1)
    for dep in crates:
        if dep != me and re.search(rf'^{re.escape(dep)}\s*=', text, re.M):
            deps.append({"_from": f"crate/{key(me)}", "_to": f"crate/{key(dep)}", "kind": "depends"})
CR = [{"_key": key(n), "name": n} for n in crates]

for name, rows in [("file", F), ("chunk", C), ("symbol", S), ("crate", CR), ("defines", defines), ("references", refs), ("depends", deps)]:
    (out / f"{name}.jsonl").write_text("\n".join(json.dumps(r) for r in rows) + "\n", encoding="utf-8")
print(json.dumps({"files": len(F), "chunks": len(C), "symbols": len(S), "crates": len(CR),
                  "defines": len(defines), "references": len(refs), "depends": len(deps)}))
