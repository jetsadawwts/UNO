#!/bin/sh
# UNO smoke test — run after updating Claude Code, Ollama or UNO.
# Usage:  sh ~/UNO/uno/scripts/smoke_test.sh <onboarded-project-folder> [local]
#   local = also test the Ollama session (same settings as the uno-local shell function)
# Checks: hook logic tests pass; uno-stable is on a tag; a real Claude Code session loads UNO's hooks.
# Sends one short prompt ("approve T99") per backend; no plan is changed, no code is read.
set -u
PROJ="${1:?give the path of an onboarded project}"
UNO="$HOME/UNO/uno"; STABLE="$HOME/UNO/uno-stable"; ok=0; bad=0
pass() { echo "PASS  $1"; ok=$((ok+1)); }
fail() { echo "FAIL  $1"; bad=$((bad+1)); }

echo "claude: $(claude --version 2>/dev/null || echo 'not found')"
python3 "$UNO/tests/test_hooks.py" >/tmp/uno-smoke-tests.txt 2>&1 && pass "hook logic tests ($(tail -1 /tmp/uno-smoke-tests.txt))" || fail "hook logic tests — see /tmp/uno-smoke-tests.txt"
tag=$(git -C "$STABLE" describe --tags --exact-match 2>/dev/null) && pass "uno-stable on tag $tag" || fail "uno-stable is not on a release tag"
[ -L "$PROJ/.uno" ] && [ -f "$PROJ/.uno/zones" ] && pass ".uno link and zones present" || fail ".uno is not a link to ~/UNO/data or zones missing"

run() {  # $1 label, rest: env assignments
  label="$1"; shift
  out=$(cd "$PROJ" && env "$@" claude -p "approve T99" --plugin-dir "$STABLE/plugins/core" 2>&1 | head -c 2000)
  case "$out" in
    *"no plan"*|*"T99"*) pass "$label: session loads UNO hooks";;
    *) fail "$label: hook did not answer — output: $(echo "$out" | head -3)";;
  esac
}
run "claude" UNO_SMOKE=1
if [ "${2:-}" = "local" ]; then
  m="${UNO_LOCAL_MODEL:-qwen3-coder:30b}"
  run "ollama $m" CLAUDE_CONFIG_DIR="$HOME/.claude-local" ANTHROPIC_AUTH_TOKEN=ollama ANTHROPIC_API_KEY= \
      ANTHROPIC_BASE_URL="${UNO_LOCAL_URL:?set UNO_LOCAL_URL to your Ollama address}" ANTHROPIC_MODEL="$m" \
      ANTHROPIC_DEFAULT_OPUS_MODEL="$m" ANTHROPIC_DEFAULT_SONNET_MODEL="$m" ANTHROPIC_DEFAULT_HAIKU_MODEL="$m"
fi
echo "---- $ok passed, $bad failed"
[ "$bad" -eq 0 ]
