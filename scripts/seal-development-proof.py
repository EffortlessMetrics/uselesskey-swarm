#!/usr/bin/env python3
"""Validate and seal an exact-head Uselesskey development proof receipt."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


class ProofError(ValueError):
    """The development receipt does not establish a mergeable proof."""


def full_sha(value: Any) -> str:
    if not isinstance(value, str):
        raise ProofError("receipt git_sha must be a string")
    if len(value) != 40 or any(ch not in "0123456789abcdef" for ch in value):
        raise ProofError("receipt git_sha must be a lowercase 40-character Git SHA")
    return value


def current_head(workspace: Path) -> str:
    completed = subprocess.run(
        ("git", "rev-parse", "HEAD"),
        cwd=workspace,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise ProofError(
            "unable to identify exact workspace HEAD: " + completed.stderr.strip()
        )
    return full_sha(completed.stdout.strip())


def validate_receipt(receipt: Any, expected_head: str) -> dict[str, Any]:
    if not isinstance(receipt, dict):
        raise ProofError("receipt must be a JSON object")

    if receipt.get("schema_version") != 2:
        raise ProofError("receipt schema_version must be 2")

    receipt_head = full_sha(receipt.get("git_sha"))
    if receipt_head != expected_head:
        raise ProofError(
            f"receipt git_sha {receipt_head} does not match exact HEAD {expected_head}"
        )

    steps = receipt.get("steps")
    if not isinstance(steps, list) or not steps:
        raise ProofError("receipt must contain at least one executed step")

    ok_steps = 0
    for index, step in enumerate(steps):
        if not isinstance(step, dict):
            raise ProofError(f"steps[{index}] must be an object")
        name = step.get("name")
        status = step.get("status")
        if not isinstance(name, str) or not name:
            raise ProofError(f"steps[{index}].name must be a nonempty string")
        if status == "ok":
            ok_steps += 1
        elif status == "skipped":
            continue
        elif status == "failed":
            raise ProofError(f"receipt contains failed step: {name}")
        else:
            raise ProofError(f"receipt contains unknown step status for {name}: {status!r}")

    if ok_steps == 0:
        raise ProofError("receipt contains no successful executed step")

    for field, identity in (
        ("feature_matrix", "features"),
        ("bdd_matrix", "feature_set"),
    ):
        entries = receipt.get(field, [])
        if not isinstance(entries, list):
            raise ProofError(f"{field} must be an array")
        for index, entry in enumerate(entries):
            if not isinstance(entry, dict):
                raise ProofError(f"{field}[{index}] must be an object")
            status = entry.get("status")
            label = entry.get(identity, f"{field}[{index}]")
            if status not in {"ok", "skipped"}:
                raise ProofError(
                    f"{field} contains nonterminal result for {label}: {status!r}"
                )

    return receipt


def digest_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def render_toml(head: str, digest: str) -> str:
    return "\n".join(
        (
            f'head = "{head}"',
            f'proof_head = "{head}"',
            f'proof_receipt_sha256 = "{digest}"',
        )
    )


def self_test() -> None:
    head = "a" * 40
    valid = {
        "schema_version": 2,
        "git_sha": head,
        "steps": [
            {"name": "fmt", "status": "ok"},
            {"name": "optional", "status": "skipped"},
        ],
        "feature_matrix": [{"features": "default", "status": "ok"}],
        "bdd_matrix": [],
    }
    validate_receipt(valid, head)

    cases = [
        ({**valid, "schema_version": 1}, "schema"),
        ({**valid, "git_sha": "b" * 40}, "head drift"),
        ({**valid, "steps": []}, "no steps"),
        (
            {
                **valid,
                "steps": [{"name": "tests", "status": "failed"}],
            },
            "failed step",
        ),
        (
            {
                **valid,
                "feature_matrix": [{"features": "default", "status": "failed"}],
            },
            "failed matrix",
        ),
    ]
    rejected = 0
    for receipt, _label in cases:
        try:
            validate_receipt(receipt, head)
        except ProofError:
            rejected += 1
    if rejected != len(cases):
        raise ProofError(
            f"self-test rejected {rejected} of {len(cases)} invalid receipts"
        )

    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "receipt.json"
        path.write_text(json.dumps(valid, sort_keys=True), encoding="utf-8")
        digest = digest_file(path)
        if len(digest) != 64:
            raise ProofError("self-test produced an invalid SHA-256 digest")

    print(f"development proof self-test: ok ({1 + len(cases)} cases)")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--workspace",
        type=Path,
        default=Path("."),
        help="Repository worktree containing the exact proved HEAD.",
    )
    parser.add_argument(
        "--receipt",
        type=Path,
        default=Path("target/xtask/receipt.json"),
        help="Receipt path, relative to the workspace unless absolute.",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Validate the receipt contract without reading repository state.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.self_test:
        self_test()
        return 0

    workspace = args.workspace.resolve()
    receipt_path = args.receipt
    if not receipt_path.is_absolute():
        receipt_path = workspace / receipt_path
    if not receipt_path.is_file():
        raise ProofError(f"receipt does not exist: {receipt_path}")

    head = current_head(workspace)
    try:
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ProofError(f"receipt JSON is invalid: {exc}") from exc

    validate_receipt(receipt, head)
    digest = digest_file(receipt_path)

    print("# Paste into the canonical promotion packet's [[development]] entry.")
    print(render_toml(head, digest))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ProofError as exc:
        print(f"development proof error: {exc}", file=sys.stderr)
        raise SystemExit(2)
