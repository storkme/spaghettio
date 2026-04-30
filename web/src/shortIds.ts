// Short-id table for compact URL fragments. TS port of the algorithm in
// `crates/core/src/short_ids.rs` — kept identical (round-robin first letters
// of each hyphen-separated word, escalating by 1 char on collision) so the
// codes match between Rust unit tests and the web app.
//
// Drift protection: the Rust side has anchor-code unit tests
// (`known_factorio_codes`) and so does this module — if the two algorithms
// ever disagree on a known slug, both test sets fail loudly.

const BASE_LENGTH = 3;
const MAX_LENGTH = 16;

/** Generate the deterministic round-robin code for `slug` at `length`. */
export function roundRobinCode(slug: string, length: number): string {
  const words = slug.split("-").filter((w) => w.length > 0);
  if (words.length === 0) return "";
  if (words.length === 1) return words[0].slice(0, length);
  let out = "";
  let pos = 0;
  // Walk pos 0, 1, 2, ... pulling one char from each word per round
  // until we've collected `length` chars or every word is exhausted.
  while (out.length < length) {
    let appended = false;
    for (const w of words) {
      if (out.length >= length) break;
      if (pos < w.length) {
        out += w[pos];
        appended = true;
      }
    }
    if (!appended || out.length >= length) break;
    pos += 1;
  }
  return out;
}

export interface ShortIdMaps {
  forward: Map<string, string>;
  reverse: Map<string, string>;
}

/** Build slug↔code maps from a slug universe. Sorted + deduped internally
 * so order of `slugs` doesn't matter. Throws if collisions can't be
 * resolved within `MAX_LENGTH` chars (effectively never for real data). */
export function buildShortIdMaps(slugs: readonly string[]): ShortIdMaps {
  const universe = [...new Set(slugs)].sort();
  const length = new Map<string, number>(universe.map((s) => [s, BASE_LENGTH]));

  for (let attempt = 0; attempt <= MAX_LENGTH; attempt++) {
    const byCode = new Map<string, string[]>();
    for (const slug of universe) {
      const code = roundRobinCode(slug, length.get(slug)!);
      const owners = byCode.get(code);
      if (owners) {
        owners.push(slug);
      } else {
        byCode.set(code, [slug]);
      }
    }
    let collided = false;
    for (const owners of byCode.values()) {
      if (owners.length > 1) {
        collided = true;
        for (const owner of owners) {
          length.set(owner, length.get(owner)! + 1);
        }
      }
    }
    if (!collided) {
      const forward = new Map<string, string>();
      const reverse = new Map<string, string>();
      for (const slug of universe) {
        const code = roundRobinCode(slug, length.get(slug)!);
        forward.set(slug, code);
        reverse.set(code, slug);
      }
      return { forward, reverse };
    }
  }
  throw new Error("buildShortIdMaps: failed to converge within MAX_LENGTH");
}

let MAPS: ShortIdMaps | null = null;

/** Install the slug↔code table. Called once during boot, after WASM init,
 * with the producible-items + machine-names union. Subsequent calls
 * replace the table — useful for tests, harmless in normal flow. */
export function initShortIds(slugs: readonly string[]): void {
  MAPS = buildShortIdMaps(slugs);
}

/** Has [`initShortIds`] been called yet? Callers that run before WASM init
 * (e.g. early skip-landing checks) can use this to decide whether the new
 * hash-form URL parser is usable yet. */
export function shortIdsReady(): boolean {
  return MAPS !== null;
}

/** Forward lookup: slug → short code. Returns null if the slug isn't in
 * the universe or the table hasn't been built yet. */
export function shortIdForSlug(slug: string): string | null {
  return MAPS?.forward.get(slug) ?? null;
}

/** Reverse lookup: short code → slug. */
export function slugForShortId(code: string): string | null {
  return MAPS?.reverse.get(code) ?? null;
}
