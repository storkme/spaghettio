#!/usr/bin/env python3
"""Join validator findings to an immutable Factorio calibration bank.

The ignored Rust driver writes current category counts and a rebuilt-blueprint
fingerprint. This tool joins that output to the bank's reports without changing
the bank. It keeps declared fixtures with no report as awaiting measurement and
keeps non-converged and kit-errored reports as labelled evidence rows.
"""

from __future__ import annotations

import argparse
import csv
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SHORTFALL_THRESHOLD_PCT = 95.0
REQUIRED_MATRIX_FIXTURE = {"label", "blueprint_sha256", "validator"}
REQUIRED_REPORT = {"converged", "kit_errors", "overall_verdict", "items"}
REQUIRED_ITEM = {
    "item",
    "is_target",
    "planned_rate",
    "measured_delivered_rate",
    "measured_produced_rate",
    "delta_pct_delivered",
    "delta_pct_produced",
    "verdict",
}


class SchemaError(ValueError):
    """The declared input contract is not present; do not infer a substitute."""


def fail(message: str) -> None:
    raise SchemaError(message)


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text())
    except FileNotFoundError:
        fail(f"missing required file: {path}")
    except json.JSONDecodeError as error:
        fail(f"invalid JSON in {path}: {error}")


def require_object(value: Any, path: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        fail(f"{path} must be an object")
    return value


def require_keys(value: dict[str, Any], keys: set[str], path: str) -> None:
    missing = sorted(keys - value.keys())
    if missing:
        fail(f"{path} missing required field(s): {', '.join(missing)}")


def require_string(value: Any, path: str) -> str:
    if not isinstance(value, str):
        fail(f"{path} must be a string")
    return value


def require_bool(value: Any, path: str) -> bool:
    if not isinstance(value, bool):
        fail(f"{path} must be a boolean")
    return value


def require_number_or_none(value: Any, path: str) -> float | None:
    if value is None:
        return None
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        fail(f"{path} must be a number or null")
    return float(value)


def kit_error_class(errors: list[Any], path: str) -> str:
    if not errors:
        return ""
    first = require_string(errors[0], f"{path}[0]")
    # Kit messages put a stable class before either a colon or a location
    # suffix (for example, "overlapping kit chests at (-2,127): ...").
    return first.split(" at (", 1)[0].split(":", 1)[0].strip()


def pct_of_plan(value: float | None, planned: float, path: str) -> float | None:
    if planned <= 0:
        fail(f"{path}.planned_rate must be greater than zero")
    return None if value is None else 100.0 * value / planned


@dataclass
class Row:
    label: str
    status: str
    converged: str
    kit_error_class: str
    delivered_pct_of_plan: float | None
    produced_pct_of_plan: float | None
    counts: dict[str, dict[str, int]]
    exclusion_reason: str

    def measured_clean(self) -> bool:
        return self.status == "measured"

    def shortfall(self) -> bool:
        rates = [self.delivered_pct_of_plan, self.produced_pct_of_plan]
        return self.measured_clean() and any(rate is not None and rate < SHORTFALL_THRESHOLD_PCT for rate in rates)

    def at_plan(self) -> bool:
        rates = [self.delivered_pct_of_plan, self.produced_pct_of_plan]
        return self.measured_clean() and all(rate is not None and rate >= SHORTFALL_THRESHOLD_PCT for rate in rates)


def parse_probe(path: Path) -> tuple[dict[str, dict[str, dict[str, int]]], dict[str, dict[str, Any]]]:
    root = require_object(load_json(path), str(path))
    require_keys(root, {"fixtures", "determinism"}, str(path))
    fixtures = require_object(root["fixtures"], f"{path}.fixtures")
    determinism = require_object(root["determinism"], f"{path}.determinism")
    parsed: dict[str, dict[str, dict[str, int]]] = {}
    for label, categories_raw in fixtures.items():
        require_string(label, f"{path}.fixtures key")
        categories = require_object(categories_raw, f"{path}.fixtures.{label}")
        parsed[label] = {}
        for category, counts_raw in categories.items():
            counts = require_object(counts_raw, f"{path}.fixtures.{label}.{category}")
            require_keys(counts, {"errors", "warnings"}, f"{path}.fixtures.{label}.{category}")
            errors, warnings = counts["errors"], counts["warnings"]
            if isinstance(errors, bool) or not isinstance(errors, int) or errors < 0:
                fail(f"{path}.fixtures.{label}.{category}.errors must be a non-negative integer")
            if isinstance(warnings, bool) or not isinstance(warnings, int) or warnings < 0:
                fail(f"{path}.fixtures.{label}.{category}.warnings must be a non-negative integer")
            parsed[label][category] = {"errors": errors, "warnings": warnings}
    parsed_determinism: dict[str, dict[str, Any]] = {}
    for label, record_raw in determinism.items():
        record = require_object(record_raw, f"{path}.determinism.{label}")
        if not isinstance(record.get("matches"), bool):
            fail(f"{path}.determinism.{label}.matches must be a boolean")
        parsed_determinism[label] = record
    if set(parsed) != set(parsed_determinism):
        fail(f"{path}: fixture and determinism label sets differ")
    return parsed, parsed_determinism


def parse_rows(bank: Path, probe_counts: dict[str, dict[str, dict[str, int]]], determinism: dict[str, dict[str, Any]]) -> list[Row]:
    matrix_path = bank / "matrix.json"
    matrix = require_object(load_json(matrix_path), str(matrix_path))
    fixtures = matrix.get("fixtures")
    if not isinstance(fixtures, list):
        fail(f"{matrix_path}.fixtures must be an array")

    rows: list[Row] = []
    labels: set[str] = set()
    for index, fixture_raw in enumerate(fixtures):
        fixture_path = f"{matrix_path}.fixtures[{index}]"
        fixture = require_object(fixture_raw, fixture_path)
        require_keys(fixture, REQUIRED_MATRIX_FIXTURE, fixture_path)
        label = require_string(fixture["label"], f"{fixture_path}.label")
        if label in labels:
            fail(f"{fixture_path}.label duplicates {label!r}")
        labels.add(label)
        require_string(fixture["blueprint_sha256"], f"{fixture_path}.blueprint_sha256")
        validator = require_object(fixture["validator"], f"{fixture_path}.validator")
        require_keys(validator, {"errors", "warnings"}, f"{fixture_path}.validator")
        for severity in ("errors", "warnings"):
            count = validator[severity]
            if isinstance(count, bool) or not isinstance(count, int) or count < 0:
                fail(f"{fixture_path}.validator.{severity} must be a non-negative integer")
        if label not in probe_counts:
            fail(f"probe output has no fixture named {label!r}")
        if label not in determinism:
            fail(f"probe output has no deterministic-build result for {label!r}")

        det = determinism[label]
        excluded = not det["matches"]
        exclusion_reason = require_string(det.get("exclusion_reason"), f"probe determinism.{label}.exclusion_reason") if excluded else ""
        report_path = bank / label / "report.json"
        if not report_path.is_file():
            status = "excluded" if excluded else "awaiting-measurement"
            rows.append(Row(label, status, "", "", None, None, probe_counts[label], exclusion_reason))
            continue

        wrapper = require_object(load_json(report_path), str(report_path))
        report = require_object(wrapper.get("report"), f"{report_path}.report")
        require_keys(report, REQUIRED_REPORT, f"{report_path}.report")
        converged = require_bool(report["converged"], f"{report_path}.report.converged")
        errors = report["kit_errors"]
        if not isinstance(errors, list):
            fail(f"{report_path}.report.kit_errors must be an array")
        error_class = kit_error_class(errors, f"{report_path}.report.kit_errors")
        require_string(report["overall_verdict"], f"{report_path}.report.overall_verdict")
        items = report["items"]
        if not isinstance(items, list):
            fail(f"{report_path}.report.items must be an array")
        targets: list[dict[str, Any]] = []
        for item_index, item_raw in enumerate(items):
            item_path = f"{report_path}.report.items[{item_index}]"
            item = require_object(item_raw, item_path)
            require_keys(item, REQUIRED_ITEM, item_path)
            require_string(item["item"], f"{item_path}.item")
            if not isinstance(item["is_target"], bool):
                fail(f"{item_path}.is_target must be a boolean")
            for numeric in ("planned_rate", "measured_delivered_rate", "measured_produced_rate", "delta_pct_delivered", "delta_pct_produced"):
                require_number_or_none(item[numeric], f"{item_path}.{numeric}")
            if item["is_target"]:
                targets.append(item)
        if len(targets) != 1:
            fail(f"{report_path}.report.items must contain exactly one is_target row; got {len(targets)}")
        target = targets[0]
        planned = require_number_or_none(target["planned_rate"], f"{report_path}.target.planned_rate")
        if planned is None:
            fail(f"{report_path}.target.planned_rate must not be null")
        delivered = pct_of_plan(require_number_or_none(target["measured_delivered_rate"], f"{report_path}.target.measured_delivered_rate"), planned, f"{report_path}.target")
        produced = pct_of_plan(require_number_or_none(target["measured_produced_rate"], f"{report_path}.target.measured_produced_rate"), planned, f"{report_path}.target")
        if excluded:
            status = "excluded"
        elif not converged:
            status = "non-converged"
        elif errors:
            status = "kit-error"
        else:
            status = "measured"
        rows.append(Row(label, status, str(converged).lower(), error_class, delivered, produced, probe_counts[label], exclusion_reason))

    extras = sorted(set(probe_counts) - labels)
    if extras:
        fail(f"probe has fixture(s) absent from matrix.json: {', '.join(extras)}")
    return rows


def category_columns(rows: list[Row]) -> list[str]:
    return sorted({category for row in rows for category in row.counts})


def format_pct(value: float | None) -> str:
    return "" if value is None else f"{value:.3f}"


def write_csv(path: Path, rows: list[Row], categories: list[str]) -> None:
    fields = ["label", "status", "converged", "kit_error_class", "delivered_pct_of_plan", "produced_pct_of_plan", "exclusion_reason"]
    fields.extend(f"{category}_errors" for category in categories)
    fields.extend(f"{category}_warnings" for category in categories)
    with path.open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        for row in rows:
            record: dict[str, Any] = {
                "label": row.label,
                "status": row.status,
                "converged": row.converged,
                "kit_error_class": row.kit_error_class,
                "delivered_pct_of_plan": format_pct(row.delivered_pct_of_plan),
                "produced_pct_of_plan": format_pct(row.produced_pct_of_plan),
                "exclusion_reason": row.exclusion_reason,
            }
            for category in categories:
                counts = row.counts.get(category, {"errors": 0, "warnings": 0})
                record[f"{category}_errors"] = counts["errors"]
                record[f"{category}_warnings"] = counts["warnings"]
            writer.writerow(record)


def markdown_table(rows: list[Row], categories: list[str]) -> str:
    columns = ["label", "status", "converged", "kit error class", "delivered %", "produced %", "exclusion reason"]
    columns += [f"{category} E" for category in categories]
    columns += [f"{category} W" for category in categories]
    lines = ["| " + " | ".join(columns) + " |", "|" + "|".join(["---"] * len(columns)) + "|"]
    for row in rows:
        values = [row.label, row.status, row.converged, row.kit_error_class, format_pct(row.delivered_pct_of_plan), format_pct(row.produced_pct_of_plan), row.exclusion_reason]
        values += [str(row.counts.get(category, {"errors": 0})["errors"]) for category in categories]
        values += [str(row.counts.get(category, {"warnings": 0})["warnings"]) for category in categories]
        lines.append("| " + " | ".join(values) + " |")
    return "\n".join(lines)


def category_labels(rows: list[Row], predicate: Any) -> dict[str, list[str]]:
    result: dict[str, list[str]] = {}
    for row in rows:
        if not predicate(row):
            continue
        for category, counts in row.counts.items():
            if counts["errors"] or counts["warnings"]:
                result.setdefault(category, []).append(row.label)
    return result


def bullet_categories(categories: dict[str, list[str]]) -> list[str]:
    if not categories:
        return ["- None."]
    return [f"- `{category}`: {', '.join(f'`{label}`' for label in labels)}" for category, labels in sorted(categories.items())]


def write_markdown(path: Path, rows: list[Row], categories: list[str], bank: Path, probe: Path, corpus_sha256: str | None) -> None:
    statuses: dict[str, int] = {}
    for row in rows:
        statuses[row.status] = statuses.get(row.status, 0) + 1
    corpus_note = (
        f" Corpus fingerprint: `{corpus_sha256}` — must match the committed "
        "`crates/core/data/calibration-bank/matrix.json` for these rows to describe the shipped engine."
        if corpus_sha256
        else ""
    )
    content = [
        "# Selection-policy calibration evidence",
        "",
        f"Source bank: `{bank}`. Validator probe: `{probe}`.{corpus_note}",
        "",
        "Status preserves campaign state: `awaiting-measurement` has no `report.json`; "
        "`non-converged` and `kit-error` retain their measured values but are excluded from the clean-row findings; "
        "`excluded` covers every probe-side determinism refusal — the probe's `exclusion_reason` names which: "
        "`blueprint-sha256-mismatch`, `manifest-sha256-mismatch`, `validator-totals-mismatch`, or `build-failed`.",
        "",
        "## Table",
        "",
        markdown_table(rows, categories),
        "",
        "## Findings",
        "",
        f"Clean-row comparison uses a {SHORTFALL_THRESHOLD_PCT:.0f}% threshold. A converged, kit-clean row is a shortfall if either available target rate is below it; it is at plan only if both target rates are available and at or above it. Rows with missing target metrics are not classified — note the structural asymmetry this creates: fluid targets carry no delivered rate (RFC-050 fluid boundaries are uncalibrated), so fluid rows can never classify as at-plan and are barred from the false-alarm section by construction; overproduction rows (the two oil fixtures measure ~150% produced) classify as at-plan only when both metrics exist and are otherwise unremarked here.",
        "",
        "### Validator categories co-occurring with clean-row shortfall",
        "",
        *bullet_categories(category_labels(rows, lambda row: row.shortfall())),
        "",
        "### Categories never seen on a clean measured row",
        "(fires only on rows outside the converged, kit-clean set — non-converged, kit-errored, awaiting, or excluded)",
        "",
    ]
    all_categories = category_labels(rows, lambda row: True)
    clean_categories = category_labels(rows, lambda row: row.measured_clean())
    only_broken = {category: labels for category, labels in all_categories.items() if category not in clean_categories}
    content += bullet_categories(only_broken)
    content += [
        "",
        "### Categories firing on clean rows at plan (false-alarm candidates)",
        "",
        *bullet_categories(category_labels(rows, lambda row: row.at_plan())),
        "",
        "These are candidates, not adjudicated false positives: this table establishes co-occurrence, not causal attribution.",
        "",
        "## Coverage",
        "",
        ", ".join(f"{status}: {count}" for status, count in sorted(statuses.items())) + ".",
        "",
    ]
    path.write_text("\n".join(content))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bank", type=Path, help="read-only calibration-bank directory")
    parser.add_argument("probe", type=Path, help="JSON emitted by selection_policy_calibration_issue_breakdown")
    parser.add_argument("--csv", required=True, type=Path, help="output CSV path")
    parser.add_argument("--markdown", required=True, type=Path, help="output readable Markdown path")
    args = parser.parse_args()
    try:
        probe_counts, determinism = parse_probe(args.probe)
        rows = parse_rows(args.bank, probe_counts, determinism)
        categories = category_columns(rows)
        args.csv.parent.mkdir(parents=True, exist_ok=True)
        args.markdown.parent.mkdir(parents=True, exist_ok=True)
        write_csv(args.csv, rows, categories)
        matrix = require_object(load_json(args.bank / "matrix.json"), str(args.bank / "matrix.json"))
        corpus_sha256 = matrix.get("corpus_sha256")
        if corpus_sha256 is not None and not isinstance(corpus_sha256, str):
            fail("matrix.json corpus_sha256 must be a string when present")
        write_markdown(args.markdown, rows, categories, args.bank, args.probe, corpus_sha256)
    except SchemaError as error:
        print(f"calibration evidence schema mismatch: {error}", file=sys.stderr)
        return 2
    print(f"wrote {args.csv}")
    print(f"wrote {args.markdown}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
