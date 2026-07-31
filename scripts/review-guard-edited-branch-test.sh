#!/usr/bin/env bash
set -uo pipefail
REPO=o/r; PR=521
CTR=$(mktemp)   # file-based counter: the previous stub incremented inside a
                # command substitution, so the parent never saw it and case 4
                # silently tested case 3 again.
run_loop() {
  any=0
  for q in "pulls/$PR/reviews|length" \
           "pulls/$PR/comments|length" \
           "issues/$PR/comments|[.[] | select(.user.login == \"claude[bot]\")] | length"; do
    path="${q%%|*}"; expr="${q#*|}"
    if ! n=$(gh api "repos/$REPO/$path" --paginate --jq "$expr" | awk '
              /^[0-9]+$/ { s += $1; seen = 1; next }
              { bad = 1; exit 3 }
              END { if (bad || !seen) exit 3; print s + 0 }'); then
      echo "  -> FAIL-OPEN ($path)"; return 9
    fi
    any=$((any + n))
  done
  echo "  -> any=$any"
  [ "$any" -gt 0 ] && return 0 || return 1
}
bump() { c=$(( $(cat "$CTR") + 1 )); echo "$c" > "$CTR"; echo "$c"; }

echo "1. all three return paginated counts   (expect rc=0, pass)"
gh() { printf '2\n1\n'; }; run_loop; echo "   rc=$?"

echo "2. all three return zero               (expect rc=1, FAIL the check)"
gh() { echo 0; }; run_loop; echo "   rc=$?"

echo "3. FIRST query exits non-zero          (expect rc=9, fail open)"
echo 0 > "$CTR"; gh() { [ "$(bump)" = 1 ] && return 1; echo 5; }; run_loop; echo "   rc=$?"

echo "4. SECOND query exits non-zero         (expect rc=9, fail open)"
echo 0 > "$CTR"; gh() { if [ "$(bump)" = 2 ]; then return 1; fi; echo 5; }; run_loop; echo "   rc=$?"

echo "5. THIRD query exits non-zero          (expect rc=9, fail open)"
echo 0 > "$CTR"; gh() { if [ "$(bump)" = 3 ]; then return 1; fi; echo 5; }; run_loop; echo "   rc=$?"

echo "6. non-numeric body, exit 0            (expect rc=9, fail open)"
gh() { echo "not json"; }; run_loop; echo "   rc=$?"

echo "7. empty output, exit 0                (expect rc=9, fail open)"
gh() { true; }; run_loop; echo "   rc=$?"

echo "8. mixed: a number then garbage        (expect rc=9, fail open)"
gh() { printf '3\noops\n'; }; run_loop; echo "   rc=$?"
