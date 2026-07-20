# Inserter-throughput followups

Status: **all items open** (created 2026-07-20 at the trim-rider close-out).
Deferred-work backlog with pick-up notes, per the followups-doc convention.

Origin: the trim rider (`acd147e`) extended the last-in-row far-belt
extension pattern (`0d7132c`) to `triple_input_row` and proved
`quad_input_row` / `dual_input_row_horizontal` structurally immune (hardcoded
single LHIs on trimmed belts / no per-machine trim at all). In doing so it
**falsified** the standing hypothesis that production-science@1's 8 residual
inserter-item-throughput warnings were this trim. The real decomposition,
adversarially verified (each warning classified by belt role, interior-vs-last
signature checked):

## 1. input3 contest-losses (6 of the 8)

electric-furnace stone-brick ×2, production-science rail ×2, rail
steel-plate ×2 — all fire on INTERIOR machines (the prod-sci pair only on
interior; last machines clean — the inverse of a trim signature). These are
losses in the input3/output **contest-resolution** mechanism of
`triple_input_row`, not belt-span starvation. Pick-up: study the
`near_far_shared` / `input3_wins` contest in templates.rs; a smarter
resolution (or a second column for the loser) is the lever. Different
mechanism from the landed extension — do not reach for belt extension here.

## 2. far-side rate wall (2 of the 8)

rail's stone side needs 2.5/s; the LHI ceiling is 2 × 1.2 = 2.4/s, so even
the fully-extended interior machines warn. The extension guard correctly
DECLINES the last machine (partial cover ≠ cover). Levers, in rough order:
the ingredient-to-belt reassignment lever (see `rfp-inserter-sizing.md`), a
third slot (needs face space — interacts with the power arc's Phase 3
band-widening machinery), or belt-tier change (user-controlled, never
automatic). A partial-cover extension variant (extend even when shortfall
remains, reducing but not clearing the warning) is a possible cheap
improvement — landed semantics currently decline it; revisit deliberately.

## 3. quad_input_row explainability gap

quad's LHI-A/LHI-B input sides (hardcoded single inserters on trimmed
belts, "dead budget" by design) emit no `InserterSideCapped` events. If
their demand ever exceeds 1.2/s the warning would be unattributed in the
explainability tooling. Pick-up: add the emit call with a
`hardcoded-single` limit tag (a Phase-2-style call-site addition per
`rfp-validation-explainability.md`).
