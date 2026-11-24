#!/bin/sh
# UNO smoke test — run after updating Claude Code, Ollama or UNO.
# Usage:  sh ~/UNO/uno/scripts/smoke_test.sh <onboarded-project-folder> [local]
#   local = also test the Ollama session (same settings as the uno-local shell function)
# Checks: hook logic tests pass; uno-stable is on a tag; a real Claude Code session loads UNO's hooks.
# Sends "uno-smoke <nonce>" per backend; the UNO hook blocks it before any model call and logs the nonce.
set -u
PROJ="${1:?give the path of an onboarded project}"
UNO="$HOME/UNO/uno"; STABLE="$HOME/UNO/uno-stable"; ok=0; bad=0
pass() { echo "PASS  $1"; ok=$((ok+1)); }
fail() { echo "FAIL  $1"; bad=$((bad+1)); }

echo "claude: $(claude --version 2>/dev/null || echo 'not found')"
python3 "$UNO/tests/test_hooks.py" >/tmp/uno-smoke-tests.txt 2>&1 && pass "hook logic tests ($(tail -1 /tmp/uno-smoke-tests.txt))" || fail "hook logic tests — see /tmp/uno-smoke-tests.txt"
tag=$(git -C "$STABLE" describe --tags --exact-match 2>/dev/null) && pass "uno-stable on tag $tag" || fail "uno-stable is not on a release tag"
[ -L "$PROJ/.uno" ] && [ -f "$PROJ/.uno/zones" ] && pass ".uno link and zones present" || fail ".uno is not a link to ~/UNO/data or zones missing"

run() {  # $1 label, rest: env assignments. The hook blocks the prompt, so no model call is made.
  label="$1"; shift
  nonce="smoke-$(date +%s)-$$"
  (cd "$PROJ" && env "$@" perl -e "alarm shift; exec @ARGV" 60 claude -p "uno-smoke $nonce" --plugin-dir "$STABLE/plugins/core" >/dev/null 2>&1)
  if grep -q "\"nonce\": \"$nonce\"" "$PROJ/.uno/audit.log" 2>/dev/null; then
    be=$(grep "\"nonce\": \"$nonce\"" "$PROJ/.uno/audit.log" | sed 's/.*"backend": "\([^"]*\)".*/\1/')
    pass "$label: session runs UNO hooks (backend $be)"
  else
    fail "$label: UNO hook did not run in a real session"
  fi
}
run "claude" UNO_SMOKE=1
if [ "${2:-}" = "local" ]; then
  : "${UNO_LOCAL_URL:?set UNO_LOCAL_URL to your Ollama address}"
  m="${UNO_LOCAL_MODEL:-qwen3-coder:30b}"
  v=$(curl -s -m 10 "$UNO_LOCAL_URL/api/version") && pass "ollama reachable $v" || fail "ollama not reachable at $UNO_LOCAL_URL"
  curl -s -m 10 "$UNO_LOCAL_URL/api/tags" | grep -q "\"$m\"" && pass "ollama has $m" || fail "ollama has no model $m"
  r=$(curl -s -m 180 "$UNO_LOCAL_URL/v1/messages" -H "content-type: application/json" -H "x-api-key: ollama" \
      -d "{\"model\":\"$m\",\"max_tokens\":300,\"messages\":[{\"role\":\"user\",\"content\":\"reply with the word ok\"}]}")
  case "$r" in *'"type":"message"'*) pass "ollama answers in Anthropic format";; *) fail "ollama /v1/messages: $(echo "$r" | head -c 200)";; esac
  run "ollama $m" CLAUDE_CONFIG_DIR="$HOME/.claude-local" ANTHROPIC_AUTH_TOKEN=ollama ANTHROPIC_API_KEY= \
      ANTHROPIC_BASE_URL="$UNO_LOCAL_URL" ANTHROPIC_MODEL="$m" \
      ANTHROPIC_DEFAULT_OPUS_MODEL="$m" ANTHROPIC_DEFAULT_SONNET_MODEL="$m" ANTHROPIC_DEFAULT_HAIKU_MODEL="$m"
fi
echo "---- $ok passed, $bad failed"
[ "$bad" -eq 0 ]
