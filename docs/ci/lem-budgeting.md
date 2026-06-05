# LEM budgeting

Linux-equivalent minutes (LEM) are the repo's CI fuel gauge.

```text
LEM = wall-clock minutes × runner multiplier
```

The multiplier lets the repo compare Linux, Windows, macOS, container, GPU, and
external review lanes with one budget language. The point is not to make CI
cheap at the expense of proof. The point is to spend verification where it buys
signal.

## Budget model

A mature policy ledger can define:

- preferred default PR budget;
- default PR hard limit;
- elevated label or risk-pack budget;
- absolute hard limit requiring explicit owner acknowledgement;
- runner multipliers;
- labels that opt into expensive lanes or waive advisory lanes.

## Practical rules

- Keep ordinary PRs below the default budget by avoiding duplicated feature-tier
  test work.
- Move coverage, broad mutation, Miri, fuzzing, and release readiness to main,
  nightly, manual, release, or labelled lanes unless a PR risk pack selects
  them.
- Use receipts to show both selected and skipped lanes.
- Treat label changes as proof/spend decisions; avoid cancelling useful runs
  just because a label changed.
- Revisit estimates with actual CI data instead of relying on aspirational
  budgets.

## Example runner multipliers

| Runner class | Multiplier |
| --- | ---: |
| Ubuntu / Linux | 1.0 |
| Windows | 2.0 |
| macOS | 10.0 |
| Docker-heavy | 6.0 |
| GPU | 6.0 |
| External AI review | 1.0 |
