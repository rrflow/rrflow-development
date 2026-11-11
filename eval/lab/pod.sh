#!/usr/bin/env bash
# Run lab systems on any host whose Docker daemon this shell can reach, Docker Desktop included.
# Usage: eval/lab/pod.sh TEST_ID RUN_DIRECTORY SYSTEM...
#
# It builds the `lab` target of .devcontainer/Containerfile, the same pinned toolchain as
# the development container, and starts it as the pod. Inside, eval/lab/pod_run.sh builds
# rrflow-lab and rrflow from this checkout, which is mounted read-only, and runs each system
# alone as a sibling container that joins the pod's network. Nothing is published on the host.
# The run directory receives every run file and the report. Cargo's cache and target stay in
# named volumes between runs, and the run's scratch volume is removed when it ends.
#
# The runner is given this host's Docker socket, which is control of its Docker daemon. Run it
# only on a host where that is already yours. No value here is a secret.
set -euo pipefail
export MSYS_NO_PATHCONV=1
test_id=$1
run_directory=$(mkdir -p "$2" && cd "$2" && pwd)
shift 2
here=$(cd "$(dirname "$0")" && pwd)
repository=$(cd "$here/../.." && pwd)
[ "$#" -gt 0 ] || { echo "name at least one system" >&2; exit 2; }

# Docker on Windows reads a mount source as a Windows path.
host_path() {
  if command -v cygpath > /dev/null 2>&1; then cygpath -m "$1"; else printf '%s\n' "$1"; fi
}

# Read here, because a worktree's .git points outside the mount: a report names what was built.
revision=$(git -C "$(host_path "$repository")" rev-parse HEAD)
changed=$(git -C "$(host_path "$repository")" status --porcelain | wc -l | tr -d ' ')

image=localhost/rrflow-lab-runner
docker build --quiet --file "$(host_path "$repository/.devcontainer/Containerfile")" \
  --target lab --tag "$image" "$(host_path "$repository/.devcontainer")" > /dev/null

pod=rrflow-lab-pod-$(od -An -N4 -tx1 /dev/urandom | tr -d ' \n')
work=$pod-work
cleanup() {
  docker rm -f "$pod" > /dev/null 2>&1 || true
  docker volume rm "$work" > /dev/null 2>&1 || true
}
trap cleanup EXIT
trap 'exit 143' TERM INT

docker volume create "$work" > /dev/null
# Root inside the runner, because the socket it is given belongs to root.
docker run --rm --name "$pod" --user root \
  -e RRFLOW_LAB_POD="$pod" -e TMPDIR=/work \
  -e RRFLOW_LAB_REVISION="$revision" -e RRFLOW_LAB_CHANGED_FILES="$changed" \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v "$(host_path "$repository"):/workspaces/rrflow:ro" \
  -v "$(host_path "$run_directory"):/runs" \
  -v rrflow-lab-cargo-registry:/cache/cargo-registry \
  -v rrflow-lab-cargo-git:/cache/cargo-git \
  -v rrflow-lab-target:/cache/target \
  -v "$work:/work" \
  "$image" eval/lab/pod_run.sh "$test_id" /runs "$@"
