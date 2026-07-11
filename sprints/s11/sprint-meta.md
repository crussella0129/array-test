# Sprint s11 — Meta

- **Sprint:** 11
- **Title:** Templatization — the repo becomes what it built
- **Phase:** loop
- **Started:** 2026-07-11
- **Exit status:** green
- **Confidence:** 1.0 (109/109 through the fmt pass; fmt + clippy gates verified locally)

## Origin
User insight: a self-verified kernel makes the repo itself a template. Confirmed with
one sharpened word (self-*certifying*, not proven) and recorded as D22 with the
two-layer analysis and the shared-hash-language property.

## Definition of done
- [x] `docs/TEMPLATE.md` — layers A/B, instantiation, the genesis ritual, the
  never-relayout rule.
- [x] `.github/workflows/ci.yml` — fmt/build/test/clippy; the rot guard now runs on
  every push, so the template's promise is live, not decorative.
- [x] `cargo fmt` hygiene pass (mechanical; 19 files) + AC77/AC78 verified locally.
- [x] README template section; D22.
- [ ] Committed & pushed. (GitHub "Template repository" toggle = human step.)

## Next
Post-1.0 backlog unchanged: T14 (user's side-decision), T7b, T8b, T12/T13, T3c.
