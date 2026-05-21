# Contributing: repo-native spec rails

When adding or updating source-of-truth artifacts, keep durable material in
`.uselesskey-spec/`.

## Rules

1. Keep the full artifact chain (why/what/decision/how/proof/closeout).
2. Do not put durable rails under `.codex/`, `.spec/`, `.claude/`, or `.jules/`.
3. Link artifacts through `.uselesskey-spec/index.toml`.
4. Use `policy/*.toml` as live ledgers; reference them from spec rails instead
   of duplicating policy state.

## Minimal external-state wording

If docs need to mention external tool directories, keep it minimal:

- they may exist;
- they are not durable source-of-truth;
- agents may read `.uselesskey-spec/` for durable lane state.
