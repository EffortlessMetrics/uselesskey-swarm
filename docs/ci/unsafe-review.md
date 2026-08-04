# Unsafe review lane

`unsafe-review` is advisory unsafe-contract review. It checks whether changed
unsafe seams have reviewable evidence: a safety contract, local guard, test
reach, and witness route.

It does not prove memory safety or undefined-behavior-free status unless a
matching witness receipt, such as Miri or sanitizer evidence, is attached.

## Role split

| Tool | Question |
| --- | --- |
| Source-exception policy | Is this unsafe/source exception allowed and owned? |
| `unsafe-review` | Is this unsafe seam reviewable: contract, guard, test reach, and witness route? |
| Miri / sanitizers | Did a concrete execution expose UB or memory misuse? |
| `xtask` | Which unsafe changes select the lane, and where are receipts summarized? |

## When to run

Run unsafe review when a PR changes unsafe Rust, FFI, C ABI, native or GPU
bindings, raw pointers, layout-sensitive code, parser boundaries with unsafe
assumptions, or witness policy for those surfaces.

This repository currently forbids unsafe code by policy. The lane remains useful
as a doctrine document and as a future route if an explicit, reviewed exception
is introduced.

## Expected artifacts

A future wrapper such as `cargo xtask unsafe-review-pr` should write stable
receipts under `target/unsafe-review/`, such as:

```text
target/unsafe-review/cards.json
target/unsafe-review/pr-summary.md
target/unsafe-review/github-summary.md
target/unsafe-review/cards.sarif
target/unsafe-review/comment-plan.json
target/unsafe-review/witness-plan.md
target/unsafe-review/lsp.json
target/unsafe-review/receipt-audit.json
```

## Claim boundary

A clean unsafe-review receipt means the selected unsafe seams are reviewable
under repo policy. It does not prove memory safety, API correctness, or witness
execution unless linked runtime receipts exist.
