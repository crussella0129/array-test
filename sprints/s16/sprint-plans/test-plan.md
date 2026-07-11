# s16 Test Plan — Finalized - DO NOT EDIT
- Default `cargo test`: 3 ignored (netns/mount/hypothesis no longer falsely pass).
- `cargo test -- --include-ignored`: all green (the three real assertions execute in a
  capable environment).
- +9 parser edge-case tests green; fmt + clippy(-D warnings) clean.
