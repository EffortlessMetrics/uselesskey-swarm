#!/usr/bin/env python3
"""Run and seal a bounded development-proof invocation; raw xtask receipts are not seals."""

from __future__ import annotations

import argparse
from contextlib import contextmanager
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
from typing import Any
import uuid

from development_proof_contract import (
    KIND, PR_COMMAND, SCHEMA_VERSION, ProofError, classify_paths, encode, expected_commands,
    json_object, parse_plan, require_sha, sha256, validate_envelope,
    validate_profile, validate_xtask,
)

OUTPUT = Path("target/development-proof")
RAW_RECEIPT = Path("target/xtask/receipt.json")


def now_ms() -> int:
    return time.time_ns() // 1_000_000


def git(workspace: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=workspace, check=False, capture_output=True, text=True
    )
    if result.returncode:
        raise ProofError(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    # Preserve whitespace in NUL-delimited changed paths.
    return result.stdout.rstrip("\n")


def clean_subject(workspace: Path, base: str) -> dict[str, str]:
    require_sha(base)
    if Path(git(workspace, "rev-parse", "--show-toplevel")).resolve() != workspace:
        raise ProofError("--workspace must be the Git worktree root")
    indexed = git(workspace, "ls-files", "-v", "-z").split("\0")
    if any(entry and (entry[0] == "S" or entry[0].islower()) for entry in indexed):
        raise ProofError("index flags can hide tracked changes; clear assume-unchanged and skip-worktree first")
    if git(workspace, "status", "--porcelain=v1", "--untracked-files=all"):
        raise ProofError("worktree or index is dirty; commit tracked and untracked source first")
    head = require_sha(git(workspace, "rev-parse", "HEAD"))
    if git(workspace, "cat-file", "-t", base) != "commit":
        raise ProofError("base must identify an existing commit")
    git(workspace, "merge-base", "--is-ancestor", base, head)
    if base == head:
        raise ProofError("base and head are identical; there is no development candidate")
    return {
        "head": head,
        "tree": require_sha(git(workspace, "rev-parse", f"{head}^{{tree}}")),
        "base": base,
    }


def classify_changes(workspace: Path, subject: dict[str, str]) -> dict[str, Any]:
    raw = git(
        workspace, "diff", "--name-only", "--no-renames", "-z",
        subject["base"], subject["head"], "--",
    )
    paths = sorted(path for path in raw.split("\0") if path)
    return classify_paths(paths)


def atomic_write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=path.parent, prefix=".proof-", delete=False) as handle:
        temporary = Path(handle.name)
        handle.write(data)
    try:
        temporary.replace(path)
    finally:
        temporary.unlink(missing_ok=True)


@contextmanager
def invocation_lock(workspace: Path):
    directory = workspace / OUTPUT
    directory.mkdir(parents=True, exist_ok=True)
    lock = directory / "lock"
    try:
        lock.mkdir()
    except FileExistsError as exc:
        raise ProofError("another invocation owns the proof lock; inspect it before clearing a stale lock") from exc
    try:
        (lock / "owner.json").write_bytes(encode({"pid": os.getpid(), "started_at_ms": now_ms()}))
        yield
    finally:
        (lock / "owner.json").unlink(missing_ok=True)
        lock.rmdir()


def execute(command: list[str], workspace: Path, base: str) -> int:
    environment = os.environ.copy()
    environment["XTASK_BASE_REF"] = base
    return subprocess.run(command, cwd=workspace, env=environment, check=False).returncode


def run_invocation(workspace: Path, base: str, plan_path: Path) -> Path:
    latest = workspace / OUTPUT / "receipt.json"
    with invocation_lock(workspace):
        invocation_id = uuid.uuid4().hex
        archive = workspace / OUTPUT / "runs" / invocation_id
        archive.mkdir(parents=True)
        envelope: dict[str, Any] = {
            "schema_version": SCHEMA_VERSION, "kind": KIND,
            "invocation_id": invocation_id, "started_at_ms": now_ms(),
            "finished_at_ms": None, "conclusion": "running", "outcomes": [],
        }
        # Invalidate the previous current seal before any new preflight or command.
        atomic_write(latest, encode(envelope))
        atomic_write(archive / "receipt.json", encode(envelope))
        try:
            subject = clean_subject(workspace, base)
            classification = classify_changes(workspace, subject)
            plan_bytes = plan_path.read_bytes()
            plan = parse_plan(plan_bytes)
            validate_profile(plan, classification)
            envelope.update({
                "subject": subject, "classification": classification,
                "plan": {"sha256": sha256(plan_bytes), "toml": plan_bytes.decode("utf-8")},
            })
            atomic_write(latest, encode(envelope))
            for name, command in expected_commands(plan):
                if clean_subject(workspace, base) != subject:
                    raise ProofError("subject changed during the invocation")
                if name == "xtask-pr":
                    raw_path = workspace / RAW_RECEIPT
                    if raw_path.exists():
                        raw_path.replace(archive / "previous-xtask-receipt.json")
                started = now_ms()
                return_code = execute(command, workspace, base)
                finished = now_ms()
                result = {
                    "name": name, "command": command, "started_at_ms": started,
                    "finished_at_ms": finished, "return_code": return_code,
                }
                envelope["outcomes"].append(result)
                atomic_write(latest, encode(envelope))
                if return_code:
                    raise ProofError(f"{name} failed with exit code {return_code}")
                if name == "xtask-pr":
                    # Read once: validation, embedded evidence and digest share these bytes.
                    raw_bytes = raw_path.read_bytes()
                    validate_xtask(raw_bytes, subject, plan, started, finished)
                    envelope["xtask_receipt"] = {
                        "sha256": sha256(raw_bytes), "utf8": raw_bytes.decode("utf-8"),
                    }
            if clean_subject(workspace, base) != subject:
                raise ProofError("subject changed during the invocation")
            envelope["finished_at_ms"] = now_ms()
            envelope["conclusion"] = "success"
            validate_envelope(encode(envelope))
        except BaseException as exc:
            envelope["conclusion"] = "failure"
            envelope["error"] = {"type": type(exc).__name__, "message": str(exc)}
            raise
        finally:
            envelope["finished_at_ms"] = now_ms()
            data = encode(envelope)
            atomic_write(latest, data)
            atomic_write(archive / "receipt.json", data)
    # Seal this invocation's archive, even if another invocation updates `latest`
    # between execution and the separate sealing check.
    return archive / "receipt.json"


def seal(workspace: Path, receipt_path: Path) -> tuple[dict[str, Any], str]:
    with invocation_lock(workspace):
        data = receipt_path.read_bytes()
        envelope = validate_envelope(data)
        if clean_subject(workspace, envelope["subject"]["base"]) != envelope["subject"]:
            raise ProofError("sealed subject no longer matches the worktree")
        if classify_changes(workspace, envelope["subject"]) != envelope["classification"]:
            raise ProofError("changed-path classification does not match Git")
        return envelope, sha256(data)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, default=Path("."))
    parser.add_argument("--receipt", type=Path, help="Seal this invocation envelope; bare xtask JSON is rejected.")
    parser.add_argument("--run", action="store_true", help="Execute the declared development proof.")
    parser.add_argument("--base", help="Exact existing ancestor commit for proof selection.")
    parser.add_argument("--plan", type=Path, help="Operator-reviewed TOML proof plan.")
    parser.add_argument("--self-test", action="store_true", help="Run the Python regression suite.")
    args = parser.parse_args()
    if args.self_test:
        test_file = Path(__file__).with_name("test_development_proof.py")
        if not test_file.is_file():
            raise ProofError("development-proof regression suite is missing")
        return subprocess.run([
            sys.executable, "-B", str(test_file),
        ], check=False).returncode
    workspace = args.workspace.resolve()
    if args.run:
        if args.receipt or not args.base or not args.plan:
            raise ProofError("--run requires --base and --plan, and cannot use --receipt")
        plan_path = args.plan if args.plan.is_absolute() else workspace / args.plan
        receipt_path = run_invocation(workspace, args.base, plan_path)
    else:
        if args.base or args.plan:
            raise ProofError("--base and --plan apply only to --run")
        receipt_path = args.receipt or OUTPUT / "receipt.json"
        if not receipt_path.is_absolute():
            receipt_path = workspace / receipt_path
    envelope, digest = seal(workspace, receipt_path)
    head = envelope["subject"]["head"]
    print(f"# Invocation receipt: {receipt_path}")
    print(f"# Invocation ID: {envelope['invocation_id']}")
    print(f"# Proof plan SHA-256: {envelope['plan']['sha256']}")
    print(f'head = "{head}"')
    print(f'proof_head = "{head}"')
    print(f'proof_receipt_sha256 = "{digest}"')
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ProofError, OSError, UnicodeError) as exc:
        print(f"development proof error: {exc}", file=sys.stderr)
        raise SystemExit(2)
