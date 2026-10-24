#!/usr/bin/env bash
# Times the development steps of this project, to compare development machines. Run it in the
# Nix dev shell (the same tool versions on each machine), on the same commit, with the machine
# on power and other heavy programs closed. No database is necessary.
#
# Usage: scripts/bench.sh [RUNS]
# RUNS is the number of runs of each small step (default 3); the result is the median. The clean
# build runs once: it uses its own empty build directory, so target/ stays as it is.
# The results go to stdout and to benchmarks/<host>-<date>.txt: commit that file.
set -euo pipefail

runs=${1:-3}
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

if ((BASH_VERSINFO[0] < 5)); then
    echo "bench.sh needs bash 5 or later (EPOCHREALTIME); run it in the Nix dev shell" >&2
    exit 1
fi
if [[ ! -d web/node_modules ]]; then
    echo "bench.sh needs the front-end packages: run 'just web-install' first" >&2
    exit 1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
export CARGO_TARGET_DIR=$work/target
# Compile from the query data in .sqlx/, without a database.
export SQLX_OFFLINE=true

host=$(hostname -s)
case $(uname -s) in
    Darwin)
        cpu=$(sysctl -n machdep.cpu.brand_string)
        cores=$(sysctl -n hw.ncpu)
        memory=$(($(sysctl -n hw.memsize) / 1024 / 1024 / 1024))
        ;;
    *)
        cpu=$(sed -n 's/^model name[[:space:]]*: //p' /proc/cpuinfo | head -1)
        cores=$(nproc)
        memory=$(($(sed -n 's/^MemTotal: *\([0-9]*\) kB/\1/p' /proc/meminfo) / 1024 / 1024))
        ;;
esac

# Seconds (with milliseconds) that a command takes; its output goes to the log.
seconds() {
    local start=$EPOCHREALTIME
    "$@" >>"$work/log" 2>&1 || {
        echo "failed: $* (log below)" >&2
        tail -30 "$work/log" >&2
        exit 1
    }
    local end=$EPOCHREALTIME
    awk -v start="$start" -v end="$end" 'BEGIN { printf "%.1f", end - start }'
}

# The median of the times of RUNS runs of a command.
median() {
    local times=()
    for _ in $(seq "$runs"); do
        times+=("$("$@")")
    done
    printf '%s\n' "${times[@]}" | sort -n | awk '{ t[NR] = $1 } END { printf "%.1f", t[int((NR + 1) / 2)] }'
}

core_changed() {
    touch crates/core/src/lib.rs
    seconds cargo build --workspace --all-targets
}

server_changed() {
    touch crates/server/src/lib.rs
    seconds cargo build --workspace --all-targets
}

core_tests() {
    touch crates/core/src/lib.rs
    seconds cargo test -p team_core --locked
}

echo "Downloading the crates (not timed)..." >&2
cargo fetch --locked >>"$work/log" 2>&1

echo "Clean build..." >&2
clean=$(seconds cargo build --workspace --all-targets --locked)
echo "Small rebuilds..." >&2
core=$(median core_changed)
server=$(median server_changed)
echo "Tests..." >&2
tests=$(median core_tests)
echo "Front end..." >&2
web=$(median seconds just check-web)

results=$(
    cat <<EOF
host:        $host
cpu:         $cpu, $cores cores, $memory GB memory
system:      $(uname -sm)
commit:      $(git rev-parse --short HEAD)$(git diff --quiet HEAD -- . ':!benchmarks' ':!scripts/bench.sh' || echo ' (with local changes)')
rustc:       $(rustc --version)
node:        $(node --version)
runs:        $runs (median)

step                                      seconds
clean build (workspace, all targets)      $clean
rebuild after a change in crates/core     $core
rebuild after a change in crates/server   $server
core tests after a change (build and run) $tests
front end (just check-web)                $web
EOF
)

mkdir -p benchmarks
out=benchmarks/$host-$(date +%Y-%m-%d).txt
echo "$results" | tee "$out"
echo "Saved in $out" >&2
