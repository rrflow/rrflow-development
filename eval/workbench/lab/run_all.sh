#!/usr/bin/env bash
# Run one test from the installed definitions: update, plan, run what is on, combine a report.
# Usage: run_all.sh TEST_ID RUN_DIRECTORY
#
# Requires RRFLOW_LAB_BIN (rrflow-lab), and for RRFlow arms RRFLOW_BIN (rrflow). What runs comes
# from `rrflow-lab plan`, so `rrflow-lab settings` is the one place to change it. A system that
# fails stays in the report with its reason, and the others still run. No value here is a secret.
set -euo pipefail
test_id=$1
run_directory=$(mkdir -p "$2" && cd "$2" && pwd)
here=$(cd "$(dirname "$0")" && pwd)
: "${RRFLOW_LAB_BIN:?set RRFLOW_LAB_BIN to the rrflow-lab binary}"

# It never starts anything, and it keeps working from the installed pack if a folder is unreachable.
"$RRFLOW_LAB_BIN" definitions update

definitions=$run_directory/definitions
"$RRFLOW_LAB_BIN" plan --test "$test_id" --out "$definitions"
export RRFLOW_LAB_DEFINITIONS=$definitions
export RRFLOW_LAB_MANIFEST=$definitions/manifests/$test_id.json

failed=()
while IFS= read -r system; do
  echo "== $system"
  "$here/run_system.sh" "$system" "$run_directory" || failed+=("$system")
done < "$definitions/run-list.txt"

"$RRFLOW_LAB_BIN" combine --manifest "$RRFLOW_LAB_MANIFEST" --in "$run_directory" \
  --out "$run_directory/report.json"
echo "Report: $run_directory/report.json"
echo
"$RRFLOW_LAB_BIN" compare --report "$run_directory/report.json" | tee "$run_directory/comparison.txt"
if [ "${#failed[@]}" -gt 0 ]; then
  echo "These did not finish, and the report says why: ${failed[*]}" >&2
  exit 3
fi
