#!/usr/bin/env python3
"""Fixture tests for the trivial-delta gate in second-opinion.yml.

The gate's decision logic lives in jq/bash embedded in workflow YAML —
outside any compiler or test harness — and every hardening round of PR
#633 fixed a misclassification found in production. This script closes
that gap: it extracts the gate's run script and its jq programs VERBATIM
from the workflow (a hand-copied twin already drifted from reality once,
on the +++/--- file-header assumption) and drives them through fixtures.

Run: python3 scripts/test_second_opinion_gate.py   (needs python3-yaml, jq)
Exit 1 on any failure. Not CI-wired; run it when touching the gate.
"""
import json
import re
import subprocess
import sys

import yaml

WF = ".github/workflows/second-opinion.yml"

failures = []


def check(name, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'} {name}" + (f"  {detail}" if detail and not ok else ""))
    if not ok:
        failures.append(name)


def jq(prog, doc, raw_input=None, null_input=False):
    # null_input mirrors the workflow's `jq -rn` timeline invocation: with
    # -n, `inputs` reads ALL page arrays; without it, the first page is
    # consumed as `.` and silently dropped — running the program under the
    # wrong mode is itself a bug this suite once had.
    inp = raw_input if raw_input is not None else json.dumps(doc)
    cmd = ["jq", "-rn" if null_input else "-r", prog]
    p = subprocess.run(cmd, input=inp, capture_output=True, text=True)
    if p.returncode != 0:
        return f"ERR:{p.stderr.strip()}"
    return p.stdout.strip()


wf = yaml.safe_load(open(WF))
steps = wf["jobs"]["second-opinion"]["steps"]
gate = next(s for s in steps if s.get("name") == "Trivial-delta gate")
record = next(s for s in steps if s.get("name") == "Record skipped review")
script = gate["run"]

# --- bash syntax of both run blocks ---
for label, body in [("gate", script), ("record", record["run"])]:
    p = subprocess.run(["bash", "-n"], input=body, capture_output=True, text=True)
    check(f"bash -n ({label} step)", p.returncode == 0, p.stderr.strip())

# --- extract the three jq programs verbatim ---
m = re.search(r"compare/\$\{last\}\.\.\.\$\{HEAD_SHA\}\" --jq '(.*?)' 2>/dev/null", script, re.S)
assert m, "verdict jq not found"
verdict_jq = m.group(1)

m = re.search(r"issues/\$\{PR\}/comments\" --paginate \\\n\s*--jq '(.*?)' \\\n", script, re.S)
assert m, "anchor jq not found"
anchor_jq = m.group(1)

m = re.search(r"retarget=\$\(jq -rn '(.*?)' <<<", script, re.S)
assert m, "timeline jq not found"
timeline_jq = m.group(1)

# --- verdict jq ---
V = [
    ("null-files -> api-failed", {"status": "ahead"}, "review:compare-api-failed"),
    ("empty-list -> skip", {"status": "ahead", "files": []}, "skip"),
    ("identical -> identical", {"status": "identical", "files": []}, "identical"),
    ("behind -> diverged", {"status": "behind", "files": []}, "review:history-diverged"),
    ("md-only -> skip", {"status": "ahead", "files": [{"filename": "docs/a.md", "status": "modified"}]}, "skip"),
    ("comment-only-rs -> skip", {"status": "ahead", "files": [
        {"filename": "x.rs", "status": "modified", "patch": "@@ -1,2 +1,2 @@\n-// a\n+// b\n+\n-"}]}, "skip"),
    ("mixed md+comment-rs -> skip", {"status": "ahead", "files": [
        {"filename": "d.md", "status": "added"},
        {"filename": "x.rs", "status": "modified", "patch": "@@ -1 +1 @@\n-// a\n+// b"}]}, "skip"),
    ("code -> review", {"status": "ahead", "files": [
        {"filename": "x.rs", "status": "modified", "patch": "@@ -1 +1 @@\n-let a=1;\n+let a=2;"}]}, "review:code-in-delta"),
    ("trailing-comment edit -> review", {"status": "ahead", "files": [
        {"filename": "x.rs", "status": "modified", "patch": "@@ -1 +1 @@\n-let a=1; // x\n+let a=1; // y"}]}, "review:code-in-delta"),
    ("++-content line -> review", {"status": "ahead", "files": [
        {"filename": "x.rs", "status": "modified", "patch": "@@ -1 +1,2 @@\n     // ctx\n+++ x;\n+// note"}]}, "review:code-in-delta"),
    ("new rs file -> review", {"status": "ahead", "files": [
        {"filename": "y.rs", "status": "added", "patch": "@@ -0,0 +1 @@\n+// only comments"}]}, "review:code-in-delta"),
    ("rename -> review", {"status": "ahead", "files": [
        {"filename": "b.md", "previous_filename": "a.md", "status": "renamed"}]}, "review:code-in-delta"),
    ("py -> review", {"status": "ahead", "files": [
        {"filename": "s.py", "status": "modified", "patch": "@@ -1 +1 @@\n-# a\n+# b"}]}, "review:code-in-delta"),
    ("rs no patch -> review", {"status": "ahead", "files": [
        {"filename": "x.rs", "status": "modified"}]}, "review:code-in-delta"),
    ("300 files -> truncated", {"status": "ahead", "files": [
        {"filename": f"f{i}.md", "status": "modified"} for i in range(300)]}, "review:file-list-truncated"),
    ("oversized rs patch -> review (truncation insurance)", {"status": "ahead", "files": [
        {"filename": "x.rs", "status": "modified",
         "patch": "@@ -1,3000 +1,3000 @@\n" + "+// c\n" * 3000}]}, "review:code-in-delta"),
]
for name, doc, exp in V:
    got = jq(verdict_jq, doc)
    check(f"verdict: {name}", got == exp, f"got {got!r} want {exp!r}")

# --- anchor jq (sha + created_at pair; foreign markers and skip markers ignored) ---
comments = [
    {"body": "<!-- second-opinion sha=" + "a" * 40 + " -->\n### 🤖 Second opinion — union ×2\n...",
     "created_at": "2026-08-14T10:00:00Z"},
    {"body": "<!-- second-opinion sha=" + "f" * 40 + " -->\n### Claude finished\nforeign bot, marker only",
     "created_at": "2026-08-14T10:30:00Z"},
    {"body": "<!-- second-opinion-skip sha=" + "b" * 40 + " -->\n**Second opinion — skipped**",
     "created_at": "2026-08-14T11:00:00Z"},
    {"body": "<!-- second-opinion sha=" + "c" * 40 + " -->\n### 🤖 Second opinion — union ×3\n...",
     "created_at": "2026-08-14T12:00:00Z"},
]
got = jq(anchor_jq, comments)
check("anchor: newest owned review, with timestamp",
      got == "c" * 40 + " 2026-08-14T12:00:00Z", f"got {got!r}")
check("anchor: no comments -> empty", jq(anchor_jq, []) == "", "")

# --- timeline jq (input is CONCATENATED page arrays, as gh --paginate emits) ---
pages = json.dumps([{"event": "labeled"}, {"event": "base_ref_changed", "created_at": "2026-08-14T11:30:00Z"}]) \
    + json.dumps([{"event": "committed"}])
got = jq(timeline_jq, None, raw_input=pages, null_input=True)
check("timeline: base_ref_changed found across pages", got == "2026-08-14T11:30:00Z", f"got {got!r}")
got = jq(timeline_jq, None, raw_input=json.dumps([{"event": "labeled"}]), null_input=True)
check("timeline: no retarget -> empty", got == "", f"got {got!r}")
got = jq(timeline_jq, None, raw_input=json.dumps([{"event": "automatic_base_change_succeeded"}]), null_input=True)
check("timeline: auto base change, no timestamp -> unknown-time", got == "unknown-time", f"got {got!r}")

# --- end-to-end orchestration harness (round-5 major: the bash between
# the jq programs — label handling, anchor parsing, the retarget-margin
# date arithmetic, verdict dispatch — was untested; a reversed comparison
# would have passed this suite green) ---
import os
import tempfile

ANCHOR_SHA = "a" * 40


def run_gate(labels, comments_out, timeline, compare_out,
             timeline_fail=False):
    with tempfile.TemporaryDirectory() as td:
        fix = os.path.join(td, "fix")
        os.mkdir(fix)
        open(os.path.join(fix, "comments.out"), "w").write(comments_out)
        if timeline_fail:
            open(os.path.join(fix, "timeline.fail"), "w").write("")
        else:
            open(os.path.join(fix, "timeline.json"), "w").write(timeline)
        open(os.path.join(fix, "compare.out"), "w").write(compare_out)
        shim = os.path.join(td, "gh")
        open(shim, "w").write(
            '#!/bin/bash\nargs="$*"\ncase "$args" in\n'
            '  *"/comments"*) cat "$FIXDIR/comments.out";;\n'
            '  *"/timeline"*) [ -e "$FIXDIR/timeline.fail" ] && exit 1; cat "$FIXDIR/timeline.json";;\n'
            '  *"/compare/"*) cat "$FIXDIR/compare.out";;\n'
            '  *) echo "unexpected gh call: $args" >&2; exit 1;;\n'
            'esac\n')
        os.chmod(shim, 0o755)
        event = os.path.join(td, "event.json")
        open(event, "w").write(json.dumps(
            {"pull_request": {"labels": [{"name": n} for n in labels]}}))
        out = os.path.join(td, "output")
        open(out, "w").write("")
        env = dict(os.environ,
                   PATH=td + os.pathsep + os.environ["PATH"],
                   FIXDIR=fix, GITHUB_EVENT_PATH=event, GITHUB_OUTPUT=out,
                   GH_TOKEN="test", PR="633", HEAD_SHA="d" * 40,
                   GITHUB_REPOSITORY="storkme/spaghettio")
        p = subprocess.run(["bash", "-c", script], env=env,
                           capture_output=True, text=True)
        outputs = dict(line.split("=", 1) for line in
                       open(out).read().splitlines() if "=" in line)
        return p, outputs


E2E = [
    # (name, labels, comments.out, timeline pages, compare.out,
    #  timeline_fail, want_trivial, want_notice, want_reason_substr)
    ("force-review label -> review", ["force-review"], "", "[]", "skip",
     False, "false", "false", "force-review label present"),
    ("no marker -> review", [], "", "[]", "skip",
     False, "false", "false", "no prior review marker"),
    ("trivial delta -> skip+notice", [],
     f"{ANCHOR_SHA} 2026-08-14T10:00:00Z\n", "[]", "skip",
     False, "true", "true", "docs/comment-only delta"),
    ("retarget inside 90m margin -> review", [],
     f"{ANCHOR_SHA} 2026-08-14T10:00:00Z\n",
     json.dumps([{"event": "base_ref_changed",
                  "created_at": "2026-08-14T09:30:00Z"}]), "skip",
     False, "false", "false", "base-retargeted"),
    ("ancient retarget -> skip", [],
     f"{ANCHOR_SHA} 2026-08-14T10:00:00Z\n",
     json.dumps([{"event": "base_ref_changed",
                  "created_at": "2026-08-14T07:00:00Z"}]), "skip",
     False, "true", "true", "docs/comment-only delta"),
    ("identical head -> skip, no notice", [],
     f"{ANCHOR_SHA} 2026-08-14T10:00:00Z\n", "[]", "identical",
     False, "true", "false", "same-head re-event"),
    ("timeline API failure -> review", [],
     f"{ANCHOR_SHA} 2026-08-14T10:00:00Z\n", "", "skip",
     True, "false", "false", "timeline-api-failed"),
    ("code delta -> review", [],
     f"{ANCHOR_SHA} 2026-08-14T10:00:00Z\n", "[]", "review:code-in-delta",
     False, "false", "false", "review:code-in-delta"),
]
for name, labels, c, tl, cmp_out, tlf, w_triv, w_not, w_reason in E2E:
    p, outputs = run_gate(labels, c, tl, cmp_out, timeline_fail=tlf)
    ok = (p.returncode == 0
          and outputs.get("trivial") == w_triv
          and outputs.get("notice") == w_not
          and w_reason in p.stdout)
    check(f"e2e: {name}", ok,
          f"rc={p.returncode} outputs={outputs} stdout={p.stdout!r} stderr={p.stderr!r}")

print("ALL PASS" if not failures else f"{len(failures)} FAILURES: {failures}")
sys.exit(1 if failures else 0)
