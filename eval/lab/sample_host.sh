#!/usr/bin/env bash
# Print one CSV line every ten seconds about the host, to read beside a lab run.
# A GB10's CPU and GPU share memory and power, so GPU work can slow a search without
# showing as CPU load. A host without a GPU leaves those columns empty.
set -uo pipefail
echo "unix_seconds,available_kib,load_1m,gpu_percent,gpu_watts"
while true; do
  gpu=$(nvidia-smi --query-gpu=utilization.gpu,power.draw --format=csv,noheader,nounits 2> /dev/null | head -n 1 | tr -d ' ')
  echo "$(date +%s),$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo),$(cut -d' ' -f1 /proc/loadavg),${gpu:-,}"
  sleep 10
done
