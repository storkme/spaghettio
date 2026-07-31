# Scratch file — #504 merge-commit repro

This file exists only to give branch A (`scratch/504-repro-a`) a
>20-changed-line, non-workflow diff so that branch B's merge commit has real
content to merge. It is not meant to land on `main`; the PR that carries it
is a deliberate throwaway, closed immediately after the `claude-review` run
is observed, with both scratch branches deleted afterward.

## Why this file exists

Issue #504 hypothesizes that the review action self-skips when a PR head is
a merge commit, and since the guard's zero-coverage failure is now
unconditional (#501), such a PR can never go green. The issue's own second
comment retracted the only other supporting data point as confounded by a
workflow-file carve-out, so before anyone implements a fix the pattern needs
a clean repro: a merge-commit head, on a PR that touches no workflow files,
with a diff comfortably over the guard's 20-changed-line triviality
threshold.

## Repro shape

- Branch A: this file, added directly on top of `origin/main`.
- Branch B: created from `origin/main`, then `git merge scratch/504-repro-a`
  — a true two-parent merge commit as its head, `git rev-list --parents -n 1
  HEAD` reporting three tokens (own SHA + two parents).
- PR opened from branch B against `main`, title prefixed `SCRATCH` so it is
  unambiguous in the PR list.
- Observation target: does `claude-review` on that PR finish in a few
  seconds with zero review activity (no PR review, no inline comment, no
  bot summary), the same signature recorded for #500's head `3fe964a9`?

## Padding

Lines below exist purely to push the diff size past the guard's 20-line
triviality carve-out; they carry no other meaning.

1. padding line one
2. padding line two
3. padding line three
4. padding line four
5. padding line five
6. padding line six
7. padding line seven
8. padding line eight
9. padding line nine
10. padding line ten
