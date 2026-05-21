# Plan Item Template

Use this shape inside an implementation plan when a PR needs a durable
human-readable work item.

````md
### `<work-item-id>` - Short title

Status: `ready`

Linked proposal: `USELESSKEY-PROP-0000`
Linked spec: `USELESSKEY-SPEC-0000`
Linked ADRs: none
Linked plan: `plans/example/implementation-plan.md`

Files:

- `path/to/file`

Support-tier impact: `none`
Policy impact: `none`

Required evidence:

```bash
cargo xtask docs-sync --check
cargo xtask typos
git diff --check
```

Non-goals:

- Do not move release, publish, signing, or source-sync authority.

Claim boundary:

- What this item proves.
- What this item does not prove.

Rollback:

- Revert the PR and leave later work items blocked until the link graph is
  repaired.
````
