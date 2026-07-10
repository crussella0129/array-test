# selfhost — the durable ledger

This workspace is array-test testing itself through its own front door: three units
whose cells drive the freshly built `array-test` binary (reached via a *relative* PATH
entry, so nothing here is machine-specific) through `tap`, `run`, and `verify`.

`state/` is the project's **first durable ledger** — the commitment that formally froze
the `array-test/v1/*` hash contexts (decisions.md D21). It is append-only history: a
rot-guard test (`tests/t15b_durable.rs`) runs the full audit over it on every CI run,
so any accidental edit to the committed past fails the suite.

To run a new round (appends to the committed history):

```sh
cargo build
target/debug/array-test run --units selfhost/units --state selfhost/state
```

Notes:
- Engine changes re-key cells; new rounds simply append with new keys. History never
  rewrites.
- `units/toolchain.lock` pins the producing environment's rustc (D16). Regenerate it
  for your environment (`rustc -vV > selfhost/units/toolchain.lock`) — that re-keys the
  workspace, which is the point.
- Scripts assume a POSIX Linux userland (GNU sed in verify's tamper step).
