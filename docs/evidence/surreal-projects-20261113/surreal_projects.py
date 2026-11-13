"""SurrealDB multi-project probe: namespace isolation and noisy-neighbour latency.

Project A = namespace `lab` on the shared server (already loaded by surreal_probe.py).
Project B writes chunks either into namespace `projb` on the SAME server or into a SEPARATE server.
Credentials come from creds.json and are never printed.
"""
import base64, json, statistics, threading, time, urllib.request, urllib.error
from pathlib import Path

HERE = Path(__file__).parent
PW = json.loads((HERE / "creds.json").read_text())["surreal"]
SHARED, SEPARATE = "http://127.0.0.1:18000", "http://127.0.0.1:18001"

def sql(base, text, ns="lab", db="lab", user="root", pw=PW, auth_ns=None):
    hdr = {**({"surreal-auth-ns": auth_ns} if auth_ns else {}), "Authorization": "Basic " + base64.b64encode(f"{user}:{pw}".encode()).decode(),
           "Accept": "application/json", "surreal-ns": ns, "surreal-db": db}
    req = urllib.request.Request(base + "/sql", data=text.encode(), method="POST", headers=hdr)
    try:
        with urllib.request.urlopen(req, timeout=300) as r:
            return json.loads(r.read())
    except urllib.error.HTTPError as e:
        return [{"status": "HTTP_ERR", "code": e.code, "result": e.read().decode()[:300]}]

QUERIES = [
    "SELECT name, path FROM symbol WHERE name = 'control_attunement';",
    "SELECT path, search::score(1) AS s FROM chunk WHERE text @1@ 'compaction policy' AND crate = 'rrd-lsm' ORDER BY s DESC LIMIT 5;",
    "SELECT <-references<-symbol.name AS c FROM symbol WHERE name = 'control_attunement';",
    "SELECT path FROM file WHERE crate = 'rrd-engine' ORDER BY bytes DESC LIMIT 10;",
]
CHUNKS = [json.loads(l) for l in (HERE / "corpus" / "chunk.jsonl").read_text(encoding="utf-8").splitlines() if l]

def server_ms(t):
    for unit, f in (("ns", 1e-6), ("µs", 1e-3), ("us", 1e-3), ("ms", 1.0), ("s", 1000.0)):
        if t.endswith(unit):
            return float(t[: -len(unit)]) * f
    return float("nan")

def measure_a(n=300):
    lat, srv = [], []
    for i in range(n):
        t = time.perf_counter(); r = sql(SHARED, QUERIES[i % len(QUERIES)]); lat.append((time.perf_counter() - t) * 1000)
        srv.append(server_ms(r[-1].get("time", "nan")))
    lat.sort(); srv.sort()
    return {"n": n, "server_p50_ms": round(statistics.median(srv), 3), "server_p95_ms": round(srv[int(n * 0.95)], 3),
            "server_max_ms": round(srv[-1], 3), "client_p50_ms": round(statistics.median(lat), 2)}

def writer(base, ns, stop, counter):
    i = 0
    while not stop.is_set():
        batch = [{k: v for k, v in c.items() if k != "_key"} for c in CHUNKS[(i * 50) % len(CHUNKS):(i * 50) % len(CHUNKS) + 50]]
        sql(base, f"INSERT INTO chunk {json.dumps(batch)} RETURN NONE;", ns=ns, db="lab")
        counter[0] += len(batch); i += 1

def with_noise(base, ns):
    stop, counter = threading.Event(), [0]
    threads = [threading.Thread(target=writer, args=(base, ns, stop, counter)) for _ in range(6)]
    for t in threads: t.start()
    time.sleep(2)
    t0 = time.time(); result = measure_a(); elapsed = time.time() - t0
    stop.set()
    for t in threads: t.join()
    result["noise_rows_written"] = counter[0]; result["noise_rows_per_s"] = round(counter[0] / (elapsed + 2), 1)
    return result

def setup_b(base):
    for _ in range(60):
        try:
            urllib.request.urlopen(base + "/health", timeout=2); break
        except Exception:
            time.sleep(1)
    sql(base, "DEFINE NAMESPACE IF NOT EXISTS projb; USE NS projb; DEFINE DATABASE IF NOT EXISTS lab;", ns="projb")
    sql(base, "DEFINE ANALYZER IF NOT EXISTS code TOKENIZERS class, blank, punct FILTERS lowercase, snowball(english);"
              "DEFINE INDEX IF NOT EXISTS chunk_text ON chunk FIELDS text FULLTEXT ANALYZER code BM25 HIGHLIGHTS;"
              "DEFINE INDEX IF NOT EXISTS chunk_vec ON chunk FIELDS vec HNSW DIMENSION 64 DIST COSINE;", ns="projb")

def isolation():
    sql(SHARED, "DEFINE NAMESPACE IF NOT EXISTS projb; USE NS projb; DEFINE USER IF NOT EXISTS agent_b ON NAMESPACE PASSWORD 'probe-only-b' ROLES VIEWER;", ns="projb")
    own = sql(SHARED, "SELECT count() FROM chunk GROUP ALL;", ns="projb", db="lab", user="agent_b", pw="probe-only-b", auth_ns="projb")[-1]
    other = sql(SHARED, "SELECT count() FROM chunk GROUP ALL;", ns="lab", db="lab", user="agent_b", pw="probe-only-b", auth_ns="projb")[-1]
    return {"agent_b_reads_projb": own.get("status"), "agent_b_reads_lab": other.get("status"), "lab_detail": str(other.get("result") or other.get("code"))[:160]}

if __name__ == "__main__":
    setup_b(SHARED); setup_b(SEPARATE)
    out = {"quiet": measure_a()}
    out["noise_same_server_other_namespace"] = with_noise(SHARED, "projb")
    out["quiet_again"] = measure_a()
    out["noise_separate_server"] = with_noise(SEPARATE, "projb")
    out["isolation"] = isolation()
    (HERE / "surreal_projects_results.json").write_text(json.dumps(out, indent=1))
    print(json.dumps(out, indent=1))
