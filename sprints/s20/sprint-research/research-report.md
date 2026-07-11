# s20 Research — the security cluster (F16–F18, F21)

Re-audited the plan's four security findings against the live code rather than the plan's
prose, because the plan had already proven partly stale elsewhere (F1 count, F7/F8).

## F18 — unit `id` reaches path construction (REAL)
`id` is author-controlled (manifest.toml) and flows to `mutation.rs`
(`id.replace('/', "_")` for the work-dir name) and to unit-dir enumeration. `replace('/')`
handles one separator but not `\`, `..`, leading dot, or control chars. Verdict: validate
at load time — the earliest, cheapest point, and it keeps the error at its cause. This is
the same doctrine already applied to the self-dependency and duplicate-dep checks.

## F16 — judge `critique_ref` joined to state dir (STALE PREMISE)
Traced `critique_ref`: set in `judge_round` to `format!("ledger/critiques/{}/N.md",
cell.cell_key.hex())` — a fixed shape whose only variable is a 64-hex cell key. It is
engine-generated, never read from an attacker-controlled source before the join, so it
cannot traverse. Not a live vulnerability. Kept a `safe_state_path` guard anyway: the
day a `CellJudgment` is deserialized from disk and fed back here, the guard is already in
place. Defense-in-depth, not a fix — recorded as such so the audit trail is honest.

## F17 / F21 — undocumented trust boundaries (DOC)
`run_repair` spawns an operator-authored command; `make_root_readonly` is `unsafe` with a
mount-namespace precondition. Neither was a bug; both were under-documented. The trust
level and the uncheckable precondition belong in doc-comments so a future reader does not
have to re-derive them.

## Doctrine
Validate at the source (F18), contain at the use site as defense-in-depth (F16), document
the boundary the type system can't enforce (F17/F21). Record every place the plan's premise
was wrong so the divergence stays auditable (D29).
