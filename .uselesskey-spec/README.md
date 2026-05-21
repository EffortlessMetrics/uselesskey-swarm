# `.uselesskey-spec/` durable spec namespace

`uselesskey` keeps durable, repo-owned source-of-truth rails under
`.uselesskey-spec/`.

## Scope

This namespace is for long-lived repository knowledge:

- proposals (`why`)
- specs (`what`)
- ADRs (`decision`)
- lane trackers and implementation plans (`how`)
- support/policy references (`what users may claim` and `what enforces it`)
- closeouts (`what happened`)

## External tool state

Tool/session directories are awareness-only and not owned by this system:

- `.codex/`
- `.spec/`
- `.claude/`
- `.jules/`

They may reference or read durable artifacts from `.uselesskey-spec/`, but this
namespace does not manage agent scratch state.
