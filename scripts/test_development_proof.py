"""Regression tests for proof custody; Git is real and Cargo execution is stubbed."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location(
    "development_proof", Path(__file__).with_name("seal-development-proof.py")
)
assert SPEC is not None and SPEC.loader is not None
proof = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(proof)


class DevelopmentProofTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.git("init", "-q")
        (self.root / ".gitignore").write_text("/target\n", encoding="utf-8")
        (self.root / "README.md").write_text("Initial\n", encoding="utf-8")
        self.commit("base")
        self.base = self.git("rev-parse", "HEAD")
        source = self.root / "crates/demo/src/lib.rs"
        source.parent.mkdir(parents=True)
        source.write_text("pub fn value() -> u8 { 42 }\n", encoding="utf-8")
        self.commit("product candidate")
        self.head = self.git("rev-parse", "HEAD")
        self.plan_path = self.root / "target/plan.toml"
        self.plan_path.parent.mkdir(parents=True)
        self.set_plan()
        self.calls = []

    def git(self, *args):
        result = subprocess.run(
            ["git", "-c", "user.name=EffortlessSteven", "-c",
             "user.email=15812269+EffortlessSteven@users.noreply.github.com", *args],
            cwd=self.root, capture_output=True, text=True, check=True,
        )
        return result.stdout.strip()

    def commit(self, message):
        self.git("add", ".")
        self.git("commit", "-qm", message)

    def set_plan(self, profile="product", required=None, skips=None, focused=None):
        required = required if required is not None else ["test:demo"]
        parts = [
            "schema_version = 1", f"profile = {json.dumps(profile)}",
            "required_steps = " + json.dumps(required),
        ]
        if skips is not None:
            parts.append("[allow_skips]")
            parts.extend(f"{json.dumps(name)} = {json.dumps(reason)}" for name, reason in skips.items())
        for item in focused or []:
            parts += [
                "[[focused]]", f"name = {json.dumps(item['name'])}",
                f"kind = {json.dumps(item['kind'])}",
                "command = " + json.dumps(item["command"]),
            ]
        self.plan_path.write_text("\n".join(parts) + "\n", encoding="utf-8")

    def raw_receipt(self):
        steps = [
            {"name": "detect-changes", "status": "ok", "details": f"base_ref={self.base}, files=1"},
            {"name": "public-surface", "status": "ok", "details": None},
            {"name": "spec-check", "status": "ok", "details": None},
            {"name": "test:demo", "status": "ok", "details": None},
        ]
        return {
            "schema_version": 2, "timestamp": proof.now_ms(), "git_sha": self.head,
            "crate_set": "pr:1", "steps": steps, "feature_matrix": [], "bdd_matrix": [],
            "bdd_counts": {},
        }

    def writer(self, transform=None, return_code=0):
        def execute(command, workspace, base):
            self.calls.append((command, workspace, base))
            if command == proof.PR_COMMAND:
                raw = self.raw_receipt()
                if transform:
                    raw = transform(raw)
                path = self.root / proof.RAW_RECEIPT
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(proof.encode(raw))
            return return_code
        return execute

    def run_proof(self, execute=None):
        with patch.object(proof, "execute", side_effect=execute or self.writer()):
            return proof.run_invocation(self.root, self.base, self.plan_path)

    def success(self):
        self.run_proof()
        path = self.root / proof.OUTPUT / "receipt.json"
        return path, json.loads(path.read_bytes())

    def update_envelope(self, path, envelope):
        path.write_bytes(proof.encode(envelope))

    def make_docs_candidate(self):
        self.git("reset", "--hard", self.base)
        (self.root / "README.md").write_text("Changed documentation\n", encoding="utf-8")
        self.commit("docs candidate")
        self.head = self.git("rev-parse", "HEAD")
        self.set_plan(
            "docs", ["spec-check"], {"tests": "No product source changed."},
            [{"name": "docs-sync", "kind": "docs", "command": ["cargo", "xtask", "docs-sync", "--check"]}],
        )

    def docs_receipt(self, raw):
        raw["steps"] = [item for item in raw["steps"] if item["name"] != "test:demo"]
        raw["steps"].append({"name": "tests", "status": "skipped", "details": "docs-only"})
        return raw

    def test_happy_path_seals_exact_envelope_and_resolved_base(self):
        path, envelope = self.success()
        sealed, digest = proof.seal(self.root, path)
        self.assertEqual(digest, hashlib.sha256(path.read_bytes()).hexdigest())
        self.assertEqual(sealed["subject"], {
            "head": self.head, "tree": self.git("rev-parse", "HEAD^{tree}"), "base": self.base,
        })
        self.assertEqual(envelope["kind"], "uselesskey-development-proof")
        self.assertEqual([call[0] for call in self.calls], [proof.PR_COMMAND])
        self.assertEqual(self.calls[0][2], self.base)
        self.assertEqual(envelope["outcomes"][0]["return_code"], 0)
        self.assertTrue((path.parent / "runs" / envelope["invocation_id"] / "receipt.json").is_file())

    def test_bare_receipt_is_rejected_with_migration_instruction(self):
        path = self.root / proof.RAW_RECEIPT
        path.parent.mkdir(parents=True)
        path.write_bytes(proof.encode(self.raw_receipt()))
        with self.assertRaisesRegex(proof.ProofError, "bare xtask receipts"):
            proof.seal(self.root, path)

    def test_detect_changes_only_is_not_proof(self):
        def only_detect(raw):
            raw["steps"] = raw["steps"][:1]
            return raw
        with self.assertRaisesRegex(proof.ProofError, "required successful obligation"):
            self.run_proof(self.writer(only_detect))

    def test_missing_declared_obligation_is_rejected(self):
        self.set_plan(required=["test:demo", "bdd"])
        with self.assertRaisesRegex(proof.ProofError, "bdd"):
            self.run_proof()

    def test_product_cannot_qualify_with_administrative_steps_only(self):
        self.set_plan(required=["spec-check"])
        def no_tests(raw):
            raw["steps"] = raw["steps"][:3]
            return raw
        with self.assertRaisesRegex(proof.ProofError, "behavioral proof"):
            self.run_proof(self.writer(no_tests))

    def test_explicit_product_command_supplies_focused_behavioral_proof(self):
        self.set_plan(required=["spec-check"], focused=[{
            "name": "regression", "kind": "product",
            "command": ["cargo", "test", "-p", "demo", "regression"],
        }])
        path = self.run_proof()
        envelope, _ = proof.seal(self.root, path)
        self.assertEqual(envelope["outcomes"][0]["name"], "focused:regression")
        self.assertEqual(len(self.calls), 2)

    def test_docs_profile_preserves_reviewed_skip_and_focused_execution(self):
        self.make_docs_candidate()
        path = self.run_proof(self.writer(self.docs_receipt))
        envelope, _ = proof.seal(self.root, path)
        raw = json.loads(envelope["xtask_receipt"]["utf8"])
        self.assertEqual(raw["steps"][-1]["status"], "skipped")
        self.assertEqual([item["name"] for item in envelope["outcomes"]], ["focused:docs-sync", "xtask-pr"])

    def test_unreviewed_skip_is_rejected(self):
        def skip_extra(raw):
            raw["steps"].append({"name": "fmt", "status": "skipped", "details": "not selected"})
            return raw
        with self.assertRaisesRegex(proof.ProofError, "not accepted"):
            self.run_proof(self.writer(skip_extra))

    def test_skip_without_instrument_reason_is_rejected(self):
        self.set_plan(skips={"fmt": "Formatting is covered by the focused command."})
        def blank_reason(raw):
            raw["steps"].append({"name": "fmt", "status": "skipped", "details": ""})
            return raw
        with self.assertRaisesRegex(proof.ProofError, "no recorded reason"):
            self.run_proof(self.writer(blank_reason))

    def test_required_step_cannot_be_skipped_in_the_plan(self):
        self.set_plan(required=["test:demo"], skips={"test:*": "No tests needed."})
        with self.assertRaisesRegex(proof.ProofError, "cannot also be allowed"):
            self.run_proof()
        self.assertFalse(self.calls)

    def test_arbitrary_skip_wildcard_is_rejected(self):
        self.set_plan(skips={"*": "Skip everything."})
        with self.assertRaisesRegex(proof.ProofError, "wildcard"):
            self.run_proof()

    def test_failed_and_unknown_step_results_are_rejected(self):
        for status in ("failed", "unknown", "running"):
            with self.subTest(status=status):
                def change(raw):
                    raw["steps"][-1]["status"] = status
                    return raw
                with self.assertRaisesRegex(proof.ProofError, "failed or unknown"):
                    self.run_proof(self.writer(change))

    def test_duplicate_steps_are_rejected(self):
        def duplicate(raw):
            raw["steps"].append(copy.deepcopy(raw["steps"][-1]))
            return raw
        with self.assertRaisesRegex(proof.ProofError, "duplicate xtask step"):
            self.run_proof(self.writer(duplicate))

    def test_matrix_failure_is_rejected(self):
        def failed_matrix(raw):
            raw["feature_matrix"] = [{"features": "default", "status": "failed"}]
            return raw
        with self.assertRaisesRegex(proof.ProofError, "failed or unknown feature_matrix"):
            self.run_proof(self.writer(failed_matrix))

    def test_unsubstantiated_matrix_success_is_rejected(self):
        def orphan_matrix(raw):
            raw["feature_matrix"] = [{"features": "default", "status": "ok"}]
            return raw
        with self.assertRaisesRegex(proof.ProofError, "no corresponding"):
            self.run_proof(self.writer(orphan_matrix))

    def test_required_matrix_step_without_its_result_is_rejected(self):
        self.set_plan(required=["test:demo", "feature-matrix:default"])
        def absent_matrix_result(raw):
            raw["steps"].append({"name": "feature-matrix:default", "status": "ok", "details": None})
            return raw
        with self.assertRaisesRegex(proof.ProofError, "no corresponding matrix result"):
            self.run_proof(self.writer(absent_matrix_result))

    def test_stale_raw_receipt_is_rejected(self):
        def stale(raw):
            raw["timestamp"] = 1
            return raw
        with self.assertRaisesRegex(proof.ProofError, "stale"):
            self.run_proof(self.writer(stale))

    def test_base_drift_in_raw_receipt_is_rejected(self):
        def other_base(raw):
            raw["steps"][0]["details"] = "base_ref=origin/main, files=1"
            return raw
        with self.assertRaisesRegex(proof.ProofError, "explicit invocation base"):
            self.run_proof(self.writer(other_base))

    def test_previous_receipt_cannot_satisfy_a_new_empty_invocation(self):
        path, previous = self.success()
        with patch.object(proof, "execute", return_value=0):
            with self.assertRaises(FileNotFoundError):
                proof.run_invocation(self.root, self.base, self.plan_path)
        current = json.loads(path.read_bytes())
        self.assertNotEqual(current["invocation_id"], previous["invocation_id"])
        self.assertNotEqual(current["conclusion"], "success")
        with self.assertRaises(proof.ProofError):
            proof.seal(self.root, path)
        saved = path.parent / "runs" / current["invocation_id"] / "previous-xtask-receipt.json"
        self.assertTrue(saved.is_file())

    def test_nonzero_command_exit_invalidates_previous_success(self):
        path, _ = self.success()
        with self.assertRaisesRegex(proof.ProofError, "exit code 7"):
            self.run_proof(self.writer(return_code=7))
        envelope = json.loads(path.read_bytes())
        self.assertEqual(envelope["outcomes"][-1]["return_code"], 7)
        self.assertNotEqual(envelope["conclusion"], "success")

    def test_process_launch_failure_leaves_an_inspectable_terminal_failure(self):
        path, _ = self.success()
        with patch.object(proof, "execute", side_effect=FileNotFoundError("cargo missing")):
            with self.assertRaises(FileNotFoundError):
                proof.run_invocation(self.root, self.base, self.plan_path)
        failed = json.loads(path.read_bytes())
        self.assertEqual(failed["conclusion"], "failure")
        self.assertEqual(failed["error"]["type"], "FileNotFoundError")
        self.assertEqual(failed["outcomes"], [])
        with self.assertRaisesRegex(proof.ProofError, "incomplete or unsuccessful"):
            proof.seal(self.root, path)

    def test_interruption_invalidates_prior_success_and_releases_lock(self):
        path, previous = self.success()
        with patch.object(proof, "execute", side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt):
                proof.run_invocation(self.root, self.base, self.plan_path)
        failed = json.loads(path.read_bytes())
        self.assertNotEqual(failed["invocation_id"], previous["invocation_id"])
        self.assertEqual(failed["error"]["type"], "KeyboardInterrupt")
        self.assertFalse((path.parent / "lock").exists())
        with self.assertRaisesRegex(proof.ProofError, "incomplete or unsuccessful"):
            proof.seal(self.root, path)

    def test_incomplete_or_unsuccessful_envelopes_cannot_seal(self):
        path, envelope = self.success()
        for state in ("running", "failure", "cancelled", None):
            with self.subTest(state=state):
                changed = copy.deepcopy(envelope)
                changed["conclusion"] = state
                self.update_envelope(path, changed)
                with self.assertRaises(proof.ProofError):
                    proof.seal(self.root, path)

    def test_missing_command_outcome_cannot_seal(self):
        path, envelope = self.success()
        envelope["outcomes"] = []
        self.update_envelope(path, envelope)
        with self.assertRaisesRegex(proof.ProofError, "lacks declared command outcomes"):
            proof.seal(self.root, path)

    def test_wrong_committed_tree_cannot_seal(self):
        path, envelope = self.success()
        envelope["subject"]["tree"] = self.git("rev-parse", f"{self.base}^{{tree}}")
        self.update_envelope(path, envelope)
        with self.assertRaisesRegex(proof.ProofError, "no longer matches"):
            proof.seal(self.root, path)

    def test_changed_command_cannot_seal(self):
        path, envelope = self.success()
        envelope["outcomes"][0]["command"] = ["echo", "success"]
        self.update_envelope(path, envelope)
        with self.assertRaisesRegex(proof.ProofError, "differs from"):
            proof.seal(self.root, path)

    def test_dirty_tracked_tree_is_rejected_even_when_diff_check_passes(self):
        path, _ = self.success()
        (self.root / "crates/demo/src/lib.rs").write_text("pub fn value() -> u8 { 0 }\n", encoding="utf-8")
        self.git("diff", "--check")
        with self.assertRaisesRegex(proof.ProofError, "dirty"):
            proof.seal(self.root, path)

    def test_index_flags_cannot_hide_modified_source(self):
        path, _ = self.success()
        source = "crates/demo/src/lib.rs"
        for flag in ("assume-unchanged", "skip-worktree"):
            with self.subTest(flag=flag):
                self.git("update-index", "--" + flag, source)
                (self.root / source).write_text("pub fn value() -> u8 { 0 }\n", encoding="utf-8")
                self.assertEqual(self.git("status", "--porcelain"), "")
                with self.assertRaisesRegex(proof.ProofError, "index flags"):
                    proof.seal(self.root, path)
                self.git("update-index", "--no-" + flag, source)
                self.git("checkout", "--", source)

    def test_dirty_index_is_rejected_before_execution(self):
        (self.root / "README.md").write_text("Staged change\n", encoding="utf-8")
        self.git("add", "README.md")
        with self.assertRaisesRegex(proof.ProofError, "dirty"):
            self.run_proof()
        self.assertFalse(self.calls)

    def test_untracked_source_is_rejected_before_execution(self):
        (self.root / "untracked.rs").write_text("fn new_code() {}\n", encoding="utf-8")
        with self.assertRaisesRegex(proof.ProofError, "dirty"):
            self.run_proof()
        self.assertFalse(self.calls)

    def test_command_mutating_tracked_source_fails_the_invocation(self):
        write = self.writer()
        def mutate(command, workspace, base):
            result = write(command, workspace, base)
            (workspace / "README.md").write_text("Mutated by proof\n", encoding="utf-8")
            return result
        with self.assertRaisesRegex(proof.ProofError, "dirty"):
            self.run_proof(mutate)

    def test_new_commit_during_execution_invalidates_subject(self):
        write = self.writer()
        def commit_during_run(command, workspace, base):
            result = write(command, workspace, base)
            (workspace / "README.md").write_text("Concurrent commit\n", encoding="utf-8")
            self.commit("concurrent change")
            return result
        with self.assertRaisesRegex(proof.ProofError, "subject changed"):
            self.run_proof(commit_during_run)

    def test_missing_base_never_falls_back(self):
        with patch.object(proof, "execute") as execute:
            with self.assertRaises(proof.ProofError):
                proof.run_invocation(self.root, "0" * 40, self.plan_path)
            execute.assert_not_called()

    def test_unrelated_base_is_rejected(self):
        tree = self.git("rev-parse", "HEAD^{tree}")
        unrelated = self.git("commit-tree", tree, "-m", "unrelated root")
        with patch.object(proof, "execute") as execute:
            with self.assertRaises(proof.ProofError):
                proof.run_invocation(self.root, unrelated, self.plan_path)
            execute.assert_not_called()

    def test_unknown_paths_require_classification_before_execution(self):
        (self.root / "unknown.asset").write_text("Unknown format\n", encoding="utf-8")
        self.commit("unclassified change")
        with self.assertRaisesRegex(proof.ProofError, "unclassified"):
            self.run_proof()
        self.assertFalse(self.calls)

    def test_pure_contract_rejects_hand_labelled_classification(self):
        path, envelope = self.success()
        envelope["classification"]["classes"] = ["docs"]
        self.update_envelope(path, envelope)
        with self.assertRaisesRegex(proof.ProofError, "does not match its changed paths"):
            proof.validate_envelope(path.read_bytes())

    def test_pure_classification_uses_repository_relative_posix_paths(self):
        result = proof.classify_paths(["scripts/test_control.py", ".github/settings.yml"])
        self.assertEqual(result["classes"], ["control"])
        self.assertEqual(result["profile"], "control")
        for path in ("/README.md", "../README.md", "docs//README.md"):
            with self.subTest(path=path):
                with self.assertRaisesRegex(proof.ProofError, "repository-relative"):
                    proof.classify_paths([path])

    def test_leading_whitespace_is_not_removed_from_changed_paths(self):
        (self.root / " scripts").mkdir()
        (self.root / " scripts/test_fake.py").write_text("pass\n", encoding="utf-8")
        self.commit("unclassified space-prefixed path")
        with self.assertRaisesRegex(proof.ProofError, "unclassified"):
            self.run_proof()
        self.assertFalse(self.calls)

    def test_product_cannot_claim_documentation_profile(self):
        self.set_plan("docs", ["spec-check"], focused=[
            {"name": "docs", "kind": "docs", "command": ["cargo", "xtask", "docs-sync", "--check"]},
        ])
        with self.assertRaisesRegex(proof.ProofError, "profile"):
            self.run_proof()

    def test_control_changes_need_explicit_control_proof(self):
        script = self.root / "scripts/test_candidate.py"
        script.parent.mkdir()
        script.write_text("pass\n", encoding="utf-8")
        self.commit("control and product change")
        with self.assertRaisesRegex(proof.ProofError, "focused control"):
            self.run_proof()
        self.assertFalse(self.calls)

    def test_echo_and_help_commands_are_not_focused_proof(self):
        commands = [
            ["echo", "passed"],
            ["cargo", "test", "--help"],
            ["cargo", "test", "--no-run"],
            ["python3", "-c", "print('passed')"],
            ["cargo", "xtask", "publish"],
        ]
        for command in commands:
            with self.subTest(command=command):
                self.set_plan(focused=[{"name": "fake", "kind": "product", "command": command}])
                with self.assertRaises(proof.ProofError):
                    self.run_proof()

    def test_python_focus_requires_an_explicit_test_runner(self):
        self.set_plan(focused=[{
            "name": "missing-runner", "kind": "control",
            "command": ["python3", "-B", "scripts/test_behavior.py"],
        }])
        with self.assertRaisesRegex(proof.ProofError, "unrecognized control"):
            self.run_proof()
        self.assertFalse(self.calls)

    def test_plan_digest_tampering_is_rejected(self):
        path, envelope = self.success()
        envelope["plan"]["toml"] += "\n# changed plan\n"
        self.update_envelope(path, envelope)
        with self.assertRaisesRegex(proof.ProofError, "plan digest"):
            proof.seal(self.root, path)

    def test_raw_digest_tampering_is_rejected(self):
        path, envelope = self.success()
        envelope["xtask_receipt"]["utf8"] += " "
        self.update_envelope(path, envelope)
        with self.assertRaisesRegex(proof.ProofError, "xtask_receipt digest"):
            proof.seal(self.root, path)

    def test_matching_new_digest_does_not_hide_missing_obligation(self):
        path, envelope = self.success()
        raw = json.loads(envelope["xtask_receipt"]["utf8"])
        raw["steps"] = raw["steps"][:3]
        data = proof.encode(raw)
        envelope["xtask_receipt"] = {"utf8": data.decode(), "sha256": proof.sha256(data)}
        self.update_envelope(path, envelope)
        with self.assertRaisesRegex(proof.ProofError, "required successful obligation"):
            proof.seal(self.root, path)

    def test_embedded_raw_nonfinite_numbers_fail_even_with_rebound_digest(self):
        path, envelope = self.success()
        for value in (float("nan"), float("inf"), float("-inf")):
            with self.subTest(value=value):
                changed = copy.deepcopy(envelope)
                raw = json.loads(changed["xtask_receipt"]["utf8"])
                raw["bdd_counts"]["passed"] = value
                # Deliberately generate Python's nonstandard JSON numeric tokens.
                data = proof.encode(raw)
                changed["xtask_receipt"] = {"utf8": data.decode(), "sha256": proof.sha256(data)}
                self.update_envelope(path, changed)
                with self.assertRaisesRegex(proof.ProofError, "non-finite JSON"):
                    proof.seal(self.root, path)

    def test_seal_validates_and_hashes_one_read_snapshot(self):
        path, _ = self.success()
        original = path.read_bytes()
        reader = Path.read_bytes
        reads = []
        def replace_after_read(current):
            data = reader(current)
            if current == path:
                reads.append(current)
                current.write_bytes(b'{"replaced":true}')
            return data
        with patch.object(Path, "read_bytes", replace_after_read):
            _, digest = proof.seal(self.root, path)
        self.assertEqual(reads, [path])
        self.assertEqual(digest, proof.sha256(original))
        self.assertNotEqual(digest, proof.sha256(path.read_bytes()))

    def test_raw_receipt_is_embedded_and_hashed_from_one_read(self):
        raw_path = self.root / proof.RAW_RECEIPT
        reader = Path.read_bytes
        snapshots = []
        def replace_after_read(current):
            data = reader(current)
            if current == raw_path:
                snapshots.append(data)
                current.write_bytes(b'{"replaced":true}')
            return data
        with patch.object(Path, "read_bytes", replace_after_read):
            path = self.run_proof()
        envelope = proof.validate_envelope(path.read_bytes())
        self.assertEqual(len(snapshots), 1)
        self.assertEqual(envelope["xtask_receipt"]["sha256"], proof.sha256(snapshots[0]))
        self.assertEqual(envelope["xtask_receipt"]["utf8"].encode(), snapshots[0])

    def test_concurrent_wrapper_is_refused(self):
        with proof.invocation_lock(self.root):
            with self.assertRaisesRegex(proof.ProofError, "another invocation"):
                self.run_proof()

    def test_run_returns_its_archive_when_a_later_invocation_updates_latest(self):
        first = self.run_proof()
        original = first.read_bytes()
        second = self.run_proof()
        self.assertNotEqual(first, second)
        self.assertEqual(first.read_bytes(), original)
        envelope, digest = proof.seal(self.root, first)
        self.assertEqual(first.parent.name, envelope["invocation_id"])
        self.assertEqual(digest, proof.sha256(original))

    def test_duplicate_json_fields_are_rejected(self):
        with self.assertRaisesRegex(proof.ProofError, "duplicate JSON"):
            proof.json_object(b'{"kind":"one","kind":"two"}')

    def test_malformed_types_raise_proof_error(self):
        path, envelope = self.success()
        for field, value in (("classes", [[]]), ("changed_paths", [{}]), ("profile", [])):
            with self.subTest(field=field):
                changed = copy.deepcopy(envelope)
                changed["classification"][field] = value
                with self.assertRaises(proof.ProofError):
                    proof.validate_envelope(proof.encode(changed))
        for field in ("profile", "required_steps"):
            with self.subTest(plan_field=field):
                data = self.plan_path.read_bytes().replace(
                    b'profile = "product"' if field == "profile" else b'required_steps = ["test:demo"]',
                    (field + " = [[]]").encode(),
                )
                with self.assertRaises(proof.ProofError):
                    proof.parse_plan(data)
        for status in ([], {}):
            with self.subTest(status=status):
                raw = self.raw_receipt()
                raw["steps"][-1]["status"] = status
                with self.assertRaises(proof.ProofError):
                    proof.validate_xtask(
                        proof.encode(raw), envelope["subject"], proof.parse_plan(self.plan_path.read_bytes()),
                        1, proof.now_ms(),
                    )

    def test_legacy_receipt_cli_fails_clearly(self):
        path = self.root / proof.RAW_RECEIPT
        path.parent.mkdir(parents=True)
        path.write_bytes(proof.encode(self.raw_receipt()))
        result = subprocess.run([
            sys.executable, "-B", str(Path(proof.__file__).resolve()),
            "--workspace", str(self.root), "--receipt", str(path),
        ], capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 2)
        self.assertIn("bare xtask receipts", result.stderr)
        self.assertNotIn("proof_receipt_sha256", result.stdout)


if __name__ == "__main__":
    unittest.main()
