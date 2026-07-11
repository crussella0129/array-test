# s11 Research Report — Templatization

## 1. Input
User insight post-freeze: "if we sort of 'self verified' the kernel of the project,
then the repo itself becomes somewhat of a template — like a GitHub template repo."

## 2. Assessment
Correct, with one word sharpened: the kernel is **self-certifying**, not proven —
integrity verified (rot-guarded founding ledger), truthfulness reproducible (§7.4
empty-cache protocol), self-hosting green. Formal proof stays T8b.

With that precision the repo is two templates in one (→ D22):
- **Layer A, the verification kernel**: frozen engine + founding ledger + audit. An
  instance writes units (cells are commands, any language) and inherits provable
  regression from commit one. The freeze gives instances a shared hash language: any
  v1 ledger verifies under any v1 binary.
- **Layer B, the method scaffold**: the sprint-loops records that *were* the working
  memory which produced the project.

The key template mechanic is the **genesis ritual**: an instance deletes
`selfhost/state`, re-pins its toolchain, runs two founding rounds, commits — its own
D21 moment, after which the rot guard protects *its* history.

## 3. What makes it real (this sprint)
- `docs/TEMPLATE.md`: layers, instantiation, genesis ritual, the never-relayout rule.
- CI: the rot guard must actually run on every fork or the promise is decorative —
  `.github/workflows/ci.yml` (fmt check, build, test, clippy -D warnings).
- Formatting hygiene: one `cargo fmt` pass (mechanical, 19 files) so `--check` can
  gate CI without churn later.
- Flipping "Template repository" in GitHub settings is a human step (documented).

## 4. Recommendation
Ship the above; D22. Remaining post-1.0 backlog unchanged (T14 pending user's
side-decision, T7b, T8b, T12/T13, T3c).
