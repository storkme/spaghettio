import { describe, expect, it } from "vitest";
import { shortIdForSlug, slugForShortId } from "./shortIds.js";

// Anchor-code parity with the Rust unit test
// `short_ids::tests::known_factorio_codes`. If these two test sets ever
// disagree, either the JSON snapshot is stale or one of the readers is
// broken — both worth catching loudly.
describe("shortIds", () => {
  it("looks up well-known slugs", () => {
    expect(shortIdForSlug("iron-plate")).toBe("ipr");
    expect(shortIdForSlug("copper-plate")).toBe("cpo");
    expect(shortIdForSlug("iron-gear-wheel")).toBe("igw");
    expect(shortIdForSlug("electronic-circuit")).toBe("ecl");
    expect(shortIdForSlug("advanced-circuit")).toBe("acd");
    expect(shortIdForSlug("processing-unit")).toBe("pur");
    expect(shortIdForSlug("water")).toBe("wat");
    expect(shortIdForSlug("coal")).toBe("coa");
    expect(shortIdForSlug("assembling-machine-3")).toBe("am3");
  });

  it("reverses to the original slug", () => {
    expect(slugForShortId("ipr")).toBe("iron-plate");
    expect(slugForShortId("igw")).toBe("iron-gear-wheel");
    expect(slugForShortId("am3")).toBe("assembling-machine-3");
    expect(slugForShortId("ftb")).toBe("fast-transport-belt");
  });

  it("returns null for unknown tokens", () => {
    expect(shortIdForSlug("definitely-not-a-real-thing")).toBeNull();
    expect(slugForShortId("zzqx")).toBeNull();
    expect(slugForShortId("")).toBeNull();
  });
});
