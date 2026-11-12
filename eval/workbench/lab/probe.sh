#!/usr/bin/env bash
# Print one integer byte count for a lab system, for the manifest's probes.
# Usage: probe.sh memory|disk rrflow|CONTAINER
# Exits nonzero when it cannot measure, which the report shows as a missing value.
set -euo pipefail
what=$1
target=$2
root=${RRFLOW_LAB_ROOT:?}

case "$what:$target" in
  memory:rrflow)
    # Resident memory of the engine process that run_system.sh started.
    awk '/^VmRSS:/ {print $2 * 1024}' "/proc/$(cat "$root/rrflow.pid")/status"
    ;;
  disk:rrflow)
    du -sb "$root/project/.rrflow" | cut -f1
    ;;
  memory:shim)
    # Resident memory of a system that runs inside its own shim.
    awk '/^VmRSS:/ {print $2 * 1024}' "/proc/$(cat "$root/shim.pid")/status"
    ;;
  disk:shim)
    du -sb "$root/data" | cut -f1
    ;;
  memory:*)
    # The container's cgroup usage, which excludes inactive page cache. The run names the
    # exact container it created, so a concurrent run's container is never measured.
    usage=$(docker stats --no-stream --format '{{.MemUsage}}' "${RRFLOW_LAB_CONTAINER:-$target}" \
      | cut -d/ -f1 | tr -d ' ')
    python3 - "$usage" <<'EOF'
import re, sys
match = re.fullmatch(r"([0-9.]+)([KMGT]?i?B)", sys.argv[1])
if not match:
    sys.exit(1)
scale = {"B": 1, "KiB": 2**10, "MiB": 2**20, "GiB": 2**30, "TiB": 2**40,
         "KB": 10**3, "MB": 10**6, "GB": 10**9, "TB": 10**12}[match.group(2)]
print(int(float(match.group(1)) * scale))
EOF
    ;;
  disk:lab-*)
    # Measured from outside, because not every image contains a shell. The files can
    # belong to a container user, so a helper container reads them. In a pod they are in
    # the run's named volume.
    docker run --rm -v "${RRFLOW_LAB_VOLUME:-$root/data}:/measured:ro" --entrypoint sh \
      "${RRFLOW_LAB_HELPER_IMAGE:?}" -c 'du -sb /measured | cut -f1'
    ;;
  *) exit 2 ;;
esac
