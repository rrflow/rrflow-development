#!/usr/bin/env bash
# Run one lab system in isolation: start it fresh, measure it, always remove it.
# Usage: run_system.sh SYSTEM_ID RUN_DIRECTORY
#
# Requires RRFLOW_LAB_BIN (rrflow-lab), and for RRFlow arms RRFLOW_BIN (rrflow).
# Set RRFLOW_LAB_CPUSET to give every system the same cores, RRFLOW_LAB_CLIENT_CPUSET to
# keep the harness off them, and RRFLOW_LAB_MANIFEST to choose another test. No value
# here is a secret: SurrealDB's password is generated per run and exists only in this
# process's environment.
set -euo pipefail
system=$1
case "$system" in
  milvus)
    echo "Milvus comparisons are unavailable until isolated-service admission is implemented" >&2
    exit 3
    ;;
  custom-reference)
    echo "custom protocol v1 is disabled: per-run resource ownership is not implemented" >&2
    exit 3
    ;;
esac
run_directory=$(mkdir -p "$2" && cd "$2" && pwd)
here=$(cd "$(dirname "$0")" && pwd)
repository=$(cd "$here/../.." && pwd)
manifest=${RRFLOW_LAB_MANIFEST:-$here/vector-small.json}
cpuset=${RRFLOW_LAB_CPUSET:-}
: "${RRFLOW_LAB_BIN:?set RRFLOW_LAB_BIN to the rrflow-lab binary}"

# Keep TMPDIR on the disk under test: a RAM-backed /tmp would flatter whatever runs there.
work=$(mktemp -d "${TMPDIR:-/tmp}/rrflow-lab-XXXXXX")
export RRFLOW_LAB_ROOT=$work
export RRFLOW_LAB_HELPER_IMAGE=qdrant/qdrant@sha256:9bfc89f9914fc14876a88c05270685921a55b8877b254a98bcc1cc2f0a424cd2
mkdir -p "$work/data"
# Some images run as a non-root user, and this directory is private to the run.
chmod 0777 "$work/data"
container=
background=
sampler=
cleanup() {
  if [ -n "$sampler" ]; then
    kill "$sampler" 2> /dev/null || true
  fi
  if [ -n "$container" ]; then
    docker logs --tail 2000 "$container" > "$run_directory/$system.server.log" 2>&1 || true
    docker rm -f -v "$container" > /dev/null 2>&1 || true
  fi
  if [ -n "$background" ]; then
    kill "$background" 2> /dev/null || true
    wait "$background" 2> /dev/null || true
  fi
  # Container users can own these files, so a container removes them.
  docker run --rm -v "$work:/work" --entrypoint sh "$RRFLOW_LAB_HELPER_IMAGE"     -c 'rm -rf /work/data' > /dev/null 2>&1 || true
  rm -rf "$work"
}
trap cleanup EXIT
trap 'exit 143' TERM INT

wait_http() {
  for _ in $(seq 1 "$2"); do
    curl -s -o /dev/null --max-time 2 "$1" && return 0
    sleep 1
  done
  echo "$1 did not answer" >&2
  return 1
}

# An array, not a function: `function &` would background a subshell, whose process is
# not the server, so memory probes and cleanup would both act on the wrong process.
pinned=()
docker_cpus=()
if [ -n "$cpuset" ]; then
  pinned=(taskset -c "$cpuset")
  docker_cpus=(--cpuset-cpus "$cpuset")
fi

# A leftover server would answer in place of the one about to start and be measured.
require_port_free() {
  if curl -s -o /dev/null --max-time 1 "http://127.0.0.1:$1/"; then
    echo "port $1 is already answering; refusing to measure a stale server" >&2
    exit 5
  fi
}

start_container() {
  local created
  if created=$(docker run -d "$@"); then
    if [[ ! $created =~ ^[0-9a-f]{64}$ ]]; then
      echo "Docker did not return a valid container ID; refusing name-based cleanup" >&2
      return 1
    fi
    container=$created
  else
    return $?
  fi
}

start_rrflow() {
  : "${RRFLOW_BIN:?set RRFLOW_BIN to the rrflow binary}"
  mkdir -p "$work/project" "$work/bin"
  # Installation refuses a hard-linked executable, and cargo hard-links its outputs.
  install -m 755 "$RRFLOW_BIN" "$work/bin/rrflow"
  local rrflow=$work/bin/rrflow
  identity=$("$rrflow" install identity --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["installation_identity"])')
  configuration=$(python3 "$here/rrflow_settings.py" "$manifest" "$system"     "$repository/crates/authority/rrd-engine/assets/install/default-estate-configuration-v3.toml" "$work/estate.toml")
  "$rrflow" install plan --project "$work/project" --identity "$identity" --mode fresh     ${configuration:+--configuration "$configuration"} --json > "$work/plan.json"
  digest=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["installation"]["plan_sha256"])' "$work/plan.json")
  "$rrflow" install apply --project "$work/project" --mode fresh --plan "$work/plan.json" --expect "$digest" > /dev/null
  "${pinned[@]}" "$rrflow" serve --project "$work/project" --bind 127.0.0.1:19477     > "$run_directory/$system.engine.log" 2>&1 &
  background=$!
  echo "$background" > "$work/rrflow.pid"
  wait_http http://127.0.0.1:19477/v1/capabilities 120
  # The credential goes straight into this process's environment. It is never printed.
  eval "$(python3 "$here/rrflow_credentials.py" "$work/project")"
}

case "$system" in
  rrflow-*) require_port_free 19477 ;;
  qdrant) require_port_free 16333 ;;
  surrealdb) require_port_free 18000 ;;
esac

case "$system" in
  rrflow-*) start_rrflow ;;
  qdrant)
    # Assign ownership only after Docker reports a successful creation. If the
    # name is occupied, cleanup must never remove the existing container.
    start_container --name lab-qdrant "${docker_cpus[@]}" -p 127.0.0.1:16333:6333       -v "$work/data:/qdrant/storage" \
      qdrant/qdrant@sha256:9bfc89f9914fc14876a88c05270685921a55b8877b254a98bcc1cc2f0a424cd2
    wait_http http://127.0.0.1:16333/ 60
    ;;
  surrealdb)
    SURREAL_LAB_USER=lab
    SURREAL_LAB_SECRET=$(python3 -c 'import secrets; print(secrets.token_hex(24))')
    export SURREAL_LAB_USER SURREAL_LAB_SECRET
    start_container --name lab-surrealdb "${docker_cpus[@]}" -p 127.0.0.1:18000:8000 \
      -e SURREAL_USER="$SURREAL_LAB_USER" -e SURREAL_PASS="$SURREAL_LAB_SECRET" \
      -e SURREAL_CAPS_ALLOW_ALL=true --user root -v "$work/data:/data" \
      surrealdb/surrealdb@sha256:3db515b88e79737aa828c05fbf1515748004ba57495ff89a013851cc3f1ad4b9 \
      start surrealkv:///data/lab
    wait_http http://127.0.0.1:18000/version 60
    ;;
  *)
    echo "unknown system $system; a new custom system adds its own start case here" >&2
    exit 2
    ;;
esac

status=0
client_cpuset=${RRFLOW_LAB_CLIENT_CPUSET:-}
client=()
[ -n "$client_cpuset" ] && client=(taskset -c "$client_cpuset")
host_samples=$run_directory/$system.host.csv
"${client[@]}" "$here/sample_host.sh" > "$host_samples" &
sampler=$!
(cd "$repository" && "${client[@]}" "$RRFLOW_LAB_BIN" run --manifest "$manifest" --system "$system" \
  --out "$run_directory" --note "server_cpuset=${cpuset:-unpinned}" \
  --note "client_cpuset=${client_cpuset:-unpinned}") || status=$?
# Another workload can slow a run without touching its cores, so a busy GPU is reported.
awk -F, 'NR > 1 && $4 != "" {
    samples++
    if ($4 > 5) busy++
    if ($4 > peak) peak = $4
  }
  END {
    if (busy) printf "warning: GPU busy in %d of %d host samples, peak %d%%; other work may have slowed this run\n", busy, samples, peak
  }' "$host_samples" >&2
exit "$status"
