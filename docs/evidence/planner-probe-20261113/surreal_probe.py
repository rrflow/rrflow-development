"""Load the same corpus into SurrealDB and capture EXPLAIN FULL plus timing per question.

Credentials are read from creds.json and never printed.
"""
import base64, json, sys, time, urllib.request, urllib.error
from pathlib import Path

HERE = Path(__file__).parent
PW = json.loads((HERE / "creds.json").read_text())["surreal"]
BASE = "http://127.0.0.1:18000"
HDR = {"Authorization": "Basic " + base64.b64encode(f"root:{PW}".encode()).decode(),
       "Accept": "application/json", "surreal-ns": "lab", "surreal-db": "lab"}

def sql(text):
    req = urllib.request.Request(BASE + "/sql", data=text.encode(), method="POST", headers=HDR)
    try:
        with urllib.request.urlopen(req, timeout=900) as r:
            return json.loads(r.read())
    except urllib.error.HTTPError as e:
        return [{"status": "HTTP_ERR", "result": e.read().decode()[:400]}]

def rows(name):
    return [json.loads(l) for l in (HERE / "corpus" / f"{name}.jsonl").read_text(encoding="utf-8").splitlines() if l]

def ident(k):
    return f"⟨{k}⟩"

def load():
    for _ in range(60):
        try:
            urllib.request.urlopen(BASE + "/health", timeout=2); break
        except Exception:
            time.sleep(1)
    print(sql("DEFINE NAMESPACE IF NOT EXISTS lab; USE NS lab; DEFINE DATABASE IF NOT EXISTS lab;")[-1]["status"])
    for name in ("file", "chunk", "symbol", "crate"):
        data = rows(name); t = time.time(); errs = 0
        for i in range(0, len(data), 100):
            batch = [{**{k: v for k, v in r.items() if k != "_key"}, "id": r["_key"]} for r in data[i:i + 100]]
            res = sql(f"INSERT INTO {name} {json.dumps(batch)} RETURN NONE;")
            errs += sum(1 for x in res if x.get("status") != "OK")
        print(f"load {name}: {len(data)} rows, failed batches={errs}, {time.time()-t:.1f}s")
    for name in ("defines", "references", "depends"):
        data = rows(name); t = time.time(); errs = 0
        for i in range(0, len(data), 2000):
            parts = []
            for r in data[i:i + 2000]:
                ft, fk = r["_from"].split("/"); tt, tk = r["_to"].split("/")
                parts.append(f"{{in: {ft}:{ident(fk)}, out: {tt}:{ident(tk)}}}")
            res = sql(f"INSERT RELATION INTO {name} [{','.join(parts)}] RETURN NONE;")
            errs += sum(1 for x in res if x.get("status") != "OK")
            if errs and i == 0:
                print("  first error:", str(res)[:300])
        print(f"load {name}: {len(data)} edges, failed batches={errs}, {time.time()-t:.1f}s")
    ddl = [
        "DEFINE INDEX symbol_name ON symbol FIELDS name;",
        "DEFINE INDEX symbol_path ON symbol FIELDS path;",
        "DEFINE INDEX chunk_crate_path ON chunk FIELDS crate, path;",
        "DEFINE INDEX file_bytes ON file FIELDS bytes;",
        "DEFINE INDEX file_crate ON file FIELDS crate;",
        "DEFINE ANALYZER code TOKENIZERS class, blank, punct FILTERS lowercase, snowball(english);",
        "DEFINE INDEX chunk_text ON chunk FIELDS text FULLTEXT ANALYZER code BM25 HIGHLIGHTS;",
        "DEFINE INDEX chunk_vec ON chunk FIELDS vec HNSW DIMENSION 64 DIST COSINE;",
    ]
    for d in ddl:
        t = time.time(); r = sql(d)[-1]
        if r.get("status") != "OK" and "FULLTEXT" in d:
            r = sql(d.replace("FULLTEXT", "SEARCH"))[-1]
        print(f"{d[:60]} -> {r.get('status')} {str(r.get('result'))[:160] if r.get('status') != 'OK' else ''} {time.time()-t:.1f}s")

Q = "where is the LSM compaction policy"
QUESTIONS = [
    ("symbol lookup control_attunement", "SELECT name, path, line FROM symbol WHERE name = 'control_attunement'"),
    ("symbol lookup, no index field (kind+line)", "SELECT name FROM symbol WHERE kind = 'trait' AND line < 50"),
    ("BM25: where is the LSM compaction policy", f"SELECT path, start, search::score(1) AS score FROM chunk WHERE text @1@ '{Q}' ORDER BY score DESC LIMIT 5"),
    ("BM25: how does the vector planner choose TurboQuant or the exact scan", "SELECT path, start, search::score(1) AS score FROM chunk WHERE text @1@ 'how does the vector planner choose TurboQuant or the exact scan' ORDER BY score DESC LIMIT 5"),
    ("BM25 + crate filter (hybrid filter)", "SELECT path, start, search::score(1) AS score FROM chunk WHERE text @1@ 'compaction policy' AND crate = 'rrd-lsm' ORDER BY score DESC LIMIT 5"),
    ("graph 1 hop: who references control_attunement", "SELECT <-references<-symbol.{name, path} AS callers FROM symbol WHERE name = 'control_attunement'"),
    ("graph 2 hops inbound", "SELECT array::distinct(<-references<-symbol<-references<-symbol.path) AS p FROM symbol WHERE name = 'control_attunement'"),
    ("vector kNN top 5", "LET $q = (SELECT VALUE vec FROM chunk WHERE string::ends_with(path, 'rrd-lsm/src/database.rs') LIMIT 1)[0]; SELECT path, vector::distance::knn() AS d FROM chunk WHERE vec <|5,40|> $q ORDER BY d"),
    ("vector kNN + filter crate", "LET $q = (SELECT VALUE vec FROM chunk WHERE string::ends_with(path, 'rrd-lsm/src/database.rs') LIMIT 1)[0]; SELECT path, vector::distance::knn() AS d FROM chunk WHERE crate = 'rrd-lsm' AND vec <|5,40|> $q ORDER BY d"),
] + [(f"range selectivity bytes > {b}", f"SELECT path FROM file WHERE bytes > {b}") for b in (500000, 100000, 20000, 5000, 1000, 0)] + [
    (f"limit {n} after equality on crate", f"SELECT path, bytes FROM file WHERE crate = 'rrd-engine' ORDER BY bytes DESC LIMIT {n}") for n in (1, 50)]

def run():
    out = []
    for label, q in QUESTIONS:
        ex = sql(q + " EXPLAIN FULL;")[-1]
        t = time.time(); res = sql(q + ";"); wall = (time.time() - t) * 1000
        last = res[-1]
        out.append({"label": label, "surrealql": q, "explain_status": ex.get("status"), "explain": ex.get("result"),
                    "status": last.get("status"), "server_time": last.get("time"), "wall_ms": round(wall, 1),
                    "rows": len(last.get("result") or []) if isinstance(last.get("result"), list) else None,
                    "first": (last.get("result") or [])[:3] if isinstance(last.get("result"), list) else last.get("result")})
    (HERE / "surreal_results.json").write_text(json.dumps(out, indent=1, ensure_ascii=False))
    for r in out:
        print(f"\n## {r['label']}  status={r['status']} rows={r['rows']} server={r['server_time']} wall={r['wall_ms']}ms")
        print("  explain:", json.dumps(r["explain"], ensure_ascii=False)[:700])
        print("  first:", json.dumps(r["first"], ensure_ascii=False)[:300])

if __name__ == "__main__":
    if "--load" in sys.argv:
        load()
    run()
