"""Load the corpus into ArangoDB and capture how each question is actually processed.

Credentials are read from creds.json and never printed.
"""
import base64, json, sys, time, urllib.request, urllib.error
from pathlib import Path

HERE = Path(__file__).parent
PW = json.loads((HERE / "creds.json").read_text())["arango"]
BASE = "http://127.0.0.1:18529"
AUTH = "Basic " + base64.b64encode(f"root:{PW}".encode()).decode()

def call(method, path, body=None, raw=None, ok=(200, 201, 202, 409)):
    data = raw if raw is not None else (json.dumps(body).encode() if body is not None else None)
    req = urllib.request.Request(BASE + path, data=data, method=method, headers={"Authorization": AUTH, "Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=600) as r:
            return json.loads(r.read() or b"{}")
    except urllib.error.HTTPError as e:
        payload = json.loads(e.read() or b"{}")
        if e.code in ok:
            return payload
        raise RuntimeError(f"{method} {path}: {e.code} {payload.get('errorMessage')}")

def load():
    for _ in range(60):
        try:
            call("GET", "/_api/version"); break
        except Exception:
            time.sleep(1)
    call("POST", "/_api/database", {"name": "lab"})
    db = "/_db/lab"
    for name, edge in [("file", 0), ("chunk", 0), ("symbol", 0), ("crate", 0), ("defines", 1), ("references", 1), ("depends", 1)]:
        call("POST", f"{db}/_api/collection", {"name": name, "type": 3 if edge else 2})
        t = time.time()
        r = call("POST", f"{db}/_api/import?collection={name}&type=documents&overwrite=true", raw=(HERE / "corpus" / f"{name}.jsonl").read_bytes())
        print(f"load {name}: created={r.get('created')} errors={r.get('errors')} {time.time()-t:.1f}s")
    idx = [("symbol", {"type": "persistent", "fields": ["name"], "name": "symbol_name"}),
           ("symbol", {"type": "persistent", "fields": ["path"], "name": "symbol_path"}),
           ("chunk", {"type": "persistent", "fields": ["crate", "path"], "name": "chunk_crate_path"}),
           ("file", {"type": "persistent", "fields": ["bytes"], "name": "file_bytes"}),
           ("file", {"type": "persistent", "fields": ["crate"], "name": "file_crate"}),
           ("chunk", {"type": "inverted", "name": "chunk_text", "fields": [{"name": "text", "analyzer": "text_en"}, {"name": "crate"}]}),
           ("chunk", {"type": "vector", "name": "chunk_vec", "fields": ["vec"], "params": {"metric": "cosine", "dimension": 64, "nLists": 85}})]
    for coll, spec in idx:
        t = time.time()
        r = call("POST", f"{db}/_api/index?collection={coll}", spec)
        print(f"index {spec['name']}: {r.get('errorMessage') or 'ok'} {time.time()-t:.1f}s")
    call("POST", f"{db}/_api/view", {"name": "chunk_search", "type": "search-alias", "indexes": [{"collection": "chunk", "index": "chunk_text"}]})

def explain_and_profile(label, aql, bind=None):
    db = "/_db/lab"
    ex = call("POST", f"{db}/_api/explain", {"query": aql, "bindVars": bind or {}, "options": {"allPlans": False}})
    plan = ex.get("plan", {})
    t = time.time()
    cur = call("POST", f"{db}/_api/cursor", {"query": aql, "bindVars": bind or {}, "batchSize": 1000, "options": {"profile": 2}})
    wall = (time.time() - t) * 1000
    st = cur.get("extra", {}).get("stats", {})
    nodes = [{"type": n["type"], "est_items": n.get("estimatedNrItems"), "est_cost": round(n.get("estimatedCost", 0), 1),
              **({"index": [(i.get("name") or i.get("type")) if isinstance(i, dict) else str(i) for i in n.get("indexes", [])]} if n.get("indexes") else {}),
              **({"strategy": n.get("strategy")} if n.get("strategy") else {})}
             for n in plan.get("nodes", [])]
    prof = {str(n["id"]): n for n in cur.get("extra", {}).get("profile", {}).get("nodes", []) or []} if isinstance(cur.get("extra", {}).get("profile"), dict) else {}
    return {"label": label, "aql": aql, "rules": plan.get("rules"), "estimated_cost": plan.get("estimatedCost"),
            "nodes": nodes, "rows": len(cur.get("result", [])), "first": cur.get("result", [])[:3],
            "stats": {k: st.get(k) for k in ("scannedFull", "scannedIndex", "cursorsCreated", "filtered", "executionTime", "peakMemoryUsage")},
            "wall_ms": round(wall, 1), "profile_phases": cur.get("extra", {}).get("profile", {}) if not prof else None}

QUESTIONS = [
    ("symbol lookup control_attunement", "FOR s IN symbol FILTER s.name == @n RETURN {name: s.name, path: s.path, line: s.line}", {"n": "control_attunement"}),
    ("symbol lookup, no index field (kind+line)", "FOR s IN symbol FILTER s.kind == 'trait' AND s.line < 50 RETURN s.name", None),
    ("BM25: where is the LSM compaction policy", "FOR c IN chunk_search SEARCH ANALYZER(c.text IN TOKENS(@q, 'text_en'), 'text_en') SORT BM25(c) DESC LIMIT 5 RETURN {path: c.path, start: c.start, score: BM25(c)}", {"q": "where is the LSM compaction policy"}),
    ("BM25: how does the vector planner choose TurboQuant or the exact scan", "FOR c IN chunk_search SEARCH ANALYZER(c.text IN TOKENS(@q, 'text_en'), 'text_en') SORT BM25(c) DESC LIMIT 5 RETURN {path: c.path, start: c.start, score: BM25(c)}", {"q": "how does the vector planner choose TurboQuant or the exact scan"}),
    ("BM25 + crate filter (hybrid filter)", "FOR c IN chunk_search SEARCH ANALYZER(c.text IN TOKENS(@q, 'text_en'), 'text_en') AND c.crate == 'rrd-lsm' SORT BM25(c) DESC LIMIT 5 RETURN {path: c.path, start: c.start}", {"q": "compaction policy"}),
    ("graph 1 hop: who references control_attunement", "FOR s IN symbol FILTER s.name == @n FOR v IN 1..1 INBOUND s references RETURN DISTINCT {name: v.name, path: v.path}", {"n": "control_attunement"}),
    ("graph 2 hops inbound", "FOR s IN symbol FILTER s.name == @n FOR v IN 1..2 INBOUND s references OPTIONS {uniqueVertices: 'global', order: 'bfs'} RETURN DISTINCT v.path", {"n": "control_attunement"}),
    ("graph shortest path crate rrflow-cli -> rrd-lsm", "FOR a IN crate FILTER a.name == 'rrflow-cli' FOR b IN crate FILTER b.name == 'rrd-lsm' FOR v IN OUTBOUND SHORTEST_PATH a TO b depends RETURN v.name", None),
    ("vector kNN top 5", "LET q = (FOR c IN chunk FILTER c.path LIKE '%rrd-lsm/src/database.rs' LIMIT 1 RETURN c.vec)[0] FOR c IN chunk SORT APPROX_NEAR_COSINE(c.vec, q) DESC LIMIT 5 RETURN c.path", None),
    ("vector kNN + filter crate", "LET q = (FOR c IN chunk FILTER c.path LIKE '%rrd-lsm/src/database.rs' LIMIT 1 RETURN c.vec)[0] FOR c IN chunk FILTER c.crate == 'rrd-lsm' SORT APPROX_NEAR_COSINE(c.vec, q) DESC LIMIT 5 RETURN c.path", None),
] + [(f"range selectivity bytes > {b}", "FOR f IN file FILTER f.bytes > @b RETURN f.path", {"b": b}) for b in (500000, 100000, 20000, 5000, 1000, 0)] + [
    (f"limit {n} after equality on crate", "FOR f IN file FILTER f.crate == 'rrd-engine' SORT f.bytes DESC LIMIT @n RETURN f.path", {"n": n}) for n in (1, 50)]

if __name__ == "__main__":
    if "--load" in sys.argv:
        load()
    rules = call("GET", "/_api/query/rules")
    (HERE / "arango_rules.json").write_text(json.dumps(rules, indent=1))
    results = [explain_and_profile(*q) for q in QUESTIONS]
    (HERE / "arango_results.json").write_text(json.dumps(results, indent=1))
    for r in results:
        print(f"\n## {r['label']}  rows={r['rows']} wall={r['wall_ms']}ms est_cost={r['estimated_cost']}")
        print("  rules:", ", ".join(r["rules"] or []))
        print("  nodes:", " -> ".join(f"{n['type']}{'['+','.join(n['index'])+']' if 'index' in n else ''}{'('+n['strategy']+')' if 'strategy' in n else ''}~{n['est_items']}" for n in r["nodes"]))
        print("  stats:", r["stats"])
        print("  first:", json.dumps(r["first"])[:300])
