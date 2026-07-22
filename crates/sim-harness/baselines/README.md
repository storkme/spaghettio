# Blessed measured baselines (RFC-050 Phase 3)

Game-measured per-item rates, frozen by `spaghettio-sim bless`, checked by
`spaghettio-sim check --report <fresh> --baselines <this dir>`. Keyed on
(label, game 2.0.76, SA mod set, research_all) — a pin bump demands a
deliberate re-bless. Opt-in like STRESSGOLD, not CI-enforced.

First blessing (2026-07-22, the RFC-050 close-out sweep): gear10 PASS
(+0.0%), ec10 FAIL −50% (#352), automation WARN −4% delivered,
logistic FAIL −40% and military FAIL −48% (both validator-clean — #357).
FAIL baselines are deliberately blessed too: they freeze today's honest
floor so fixes must MOVE the number and regressions can't hide.
