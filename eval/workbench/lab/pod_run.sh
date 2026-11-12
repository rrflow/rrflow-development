#!/usr/bin/env bash
# Inside the lab runner, which is the pod: build rrflow-lab and rrflow from this checkout, run each
# named system alone in this container's pod, and combine their runs into one report.
# Usage: pod_run.sh TEST_ID RUN_DIRECTORY SYSTEM...   (eval/workbench/lab/pod.sh starts it)
#
# Every system container joins this container's network, so all endpoints are this container's
# own loopback. No value here is a secret.
set -euo pipefail
test_id=$1
run_directory=$(mkdir -p "$2" && cd "$2" && pwd)
shift 2
here=$(cd "$(dirname "$0")" && pwd)
repository=$(cd "$here/../../.." && pwd)
: "${RRFLOW_LAB_POD:?pod_run.sh runs inside the runner that eval/workbench/lab/pod.sh starts}"
: "${CARGO_TARGET_DIR:?the runner image sets CARGO_TARGET_DIR}"
[ "$#" -gt 0 ] || { echo "name at least one system" >&2; exit 2; }

cd "$repository"
# A debug build would misrepresent RRFlow.
cargo build --locked --release -p rrflow-eval --bin rrflow-lab -p rrflow-cli --bin rrflow
export RRFLOW_LAB_BIN=$CARGO_TARGET_DIR/release/rrflow-lab
export RRFLOW_BIN=$CARGO_TARGET_DIR/release/rrflow
export RRFLOW_LAB_MANIFEST=$here/$test_id.json
printf 'revision=%s\nchanged_files=%s\n' "${RRFLOW_LAB_REVISION:-unknown}" \
  "${RRFLOW_LAB_CHANGED_FILES:-unknown}" > "$run_directory/revision.txt"

failed=()
for system in "$@"; do
  echo "== $system"
  # What else this Docker host is running, so a reader can see the neighbours of each run.
  docker stats --no-stream --format '{{.Name}},{{.CPUPerc}},{{.MemUsage}}' \
    > "$run_directory/$system.neighbors.csv" 2>&1 || true
  "$here/run_system.sh" "$system" "$run_directory" || failed+=("$system")
done

"$RRFLOW_LAB_BIN" combine --manifest "$RRFLOW_LAB_MANIFEST" --in "$run_directory" \
  --out "$run_directory/report.json"
echo "Report: $run_directory/report.json"
if [ "${#failed[@]}" -gt 0 ]; then
  echo "These did not finish, and the report says why: ${failed[*]}" >&2
  exit 3
fi
