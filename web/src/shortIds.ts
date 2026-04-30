// Slug → short-code lookup, sourced from the Rust-generated snapshot at
// `crates/core/data/short-ids.json`. The Rust unit test
// `short_ids::tests::snapshot_matches_algorithm` guarantees the snapshot
// stays in sync with the algorithm — TS just imports it.
//
// Vite resolves the relative path at build time and inlines the JSON,
// so no runtime fetch and no algorithm port lives on this side anymore.

import snapshot from "../../crates/core/data/short-ids.json";

interface SnapshotShape {
  version: number;
  codes: Record<string, string>;
}

const TYPED = snapshot as SnapshotShape;

const FORWARD = new Map<string, string>(Object.entries(TYPED.codes));
const REVERSE = new Map<string, string>(
  Object.entries(TYPED.codes).map(([slug, code]) => [code, slug]),
);

/** Forward lookup: slug → short code. Returns null for unknown slugs. */
export function shortIdForSlug(slug: string): string | null {
  return FORWARD.get(slug) ?? null;
}

/** Reverse lookup: short code → slug. Returns null for unknown codes — the
 * URL parser uses this to detect malformed inputs and fall back to legacy
 * behaviour rather than treating an unknown token as a literal slug. */
export function slugForShortId(code: string): string | null {
  return REVERSE.get(code) ?? null;
}
