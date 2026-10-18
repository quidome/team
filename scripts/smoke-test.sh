#!/usr/bin/env bash
# Smoke test of the built images (decision 0010): migrations, health, the front end, and that the
# login bypass and the development clock are off in the release build. Used by `just smoke` and by
# the Build job in CI.
#
# Usage: scripts/smoke-test.sh RUNTIME_IMAGE MIGRATION_IMAGE
# Needs a PostgreSQL server on localhost:5432 with role team/team (`just db-up`, or the CI service).
# The script creates and drops its own database, team_smoke.
set -euo pipefail

runtime=${1:?usage: smoke-test.sh RUNTIME_IMAGE MIGRATION_IMAGE}
migration=${2:?usage: smoke-test.sh RUNTIME_IMAGE MIGRATION_IMAGE}
admin_url=postgresql://team:team@localhost:5432/postgres
database_url=postgresql://team:team@localhost:5432/team_smoke
# 32 bytes of 7 in base64; only for the smoke test.
player_name_key=BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=
containers=()

cleanup() {
    for container in "${containers[@]}"; do
        docker rm -f "$container" >/dev/null 2>&1 || true
    done
    psql "$admin_url" -qc 'DROP DATABASE IF EXISTS team_smoke' >/dev/null 2>&1 || true
}
trap cleanup EXIT

fail() {
    echo "smoke test failed: $1" >&2
    for container in "${containers[@]}"; do
        echo "--- log of $container" >&2
        docker logs "$container" >&2 || true
    done
    exit 1
}

# Starts the runtime image in the background as the cluster runs it. Extra arguments are env vars.
start() {
    local name=$1 port=$2
    shift 2
    local env_args=()
    for setting in "$@"; do env_args+=(-e "$setting"); done
    docker run -d --name "$name" --network host --read-only --user 1000:1000 \
        -e DATABASE_URL="$database_url" \
        -e OIDC_ISSUER_URL=https://id.example.test \
        -e OIDC_CLIENT_ID=team-coordinator \
        -e OIDC_CLIENT_SECRET=smoke-test \
        -e SESSION_SECRET=smoke-test-secret-of-at-least-32-chars \
        -e PLAYER_NAME_KEY="$player_name_key" \
        -e ORIGIN="http://localhost:$port" \
        -e PORT="$port" \
        "${env_args[@]}" \
        "$runtime" >/dev/null || fail "cannot start $runtime"
    containers+=("$name")
}

# Waits until a URL answers with the expected HTTP status; prints the status it got.
expect_status() {
    local url=$1 expected=$2 status=000
    for _ in $(seq 1 30); do
        status=$(curl -s -o /dev/null -w '%{http_code}' "$url" || true)
        [ "$status" = "$expected" ] && { echo "ok   $url $status"; return 0; }
        sleep 1
    done
    fail "$url gave $status, expected $expected"
}

# Containers left by an interrupted earlier run would block the fixed names below.
docker rm -f team-smoke team-smoke-bypass >/dev/null 2>&1 || true

psql "$admin_url" -qc 'DROP DATABASE IF EXISTS team_smoke' -c 'CREATE DATABASE team_smoke' >/dev/null \
    || fail "cannot create the database team_smoke"

for run in 1 2; do
    docker run --rm --network host --read-only --user 1000:1000 \
        -e DATABASE_URL="$database_url" -e PLAYER_NAME_KEY="$player_name_key" "$migration" >/dev/null \
        || fail "migration run $run failed"
    echo "ok   migration run $run"
done

start team-smoke 3000
expect_status http://localhost:3000/api/health/liveness 200
expect_status http://localhost:3000/api/health/readiness 200
expect_status http://localhost:3000/ 200
content_type=$(curl -s -o /dev/null -w '%{content_type}' http://localhost:3000/)
[[ "$content_type" == text/html* ]] || fail "/ gave content type $content_type"
echo "ok   / is the front end"

start team-smoke-bypass 3001 DEV_AUTH_BYPASS=true DEV_NOW=2020-01-01T00:00:00Z
expect_status http://localhost:3001/api/health/liveness 200
expect_status http://localhost:3001/api/me 401
echo "ok   the release build ignores DEV_AUTH_BYPASS"
# The log is read whole first: `grep -q` stops at the first match, which with pipefail could
# fail the pipe while `docker logs` still writes.
bypass_log=$(docker logs team-smoke-bypass 2>&1)
grep -q 'DEV_NOW is set but ignored' <<<"$bypass_log" \
    || fail "the release build does not say that it ignores DEV_NOW"
echo "ok   the release build ignores DEV_NOW"

echo "smoke test passed"
