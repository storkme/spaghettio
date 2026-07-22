import { describe, expect, it } from "vitest";
import { parseSimReport } from "./simReportLoader.js";
import ec10Fail from "./testdata/sim-report-ec10-fail.json";
import gear10Pass from "./testdata/sim-report-gear10-pass.json";

// Real `spaghettio-sim run --out report.json` artifacts from the #345
// dogfood (RFC-050): a FAIL case (electronic-circuit@10/s, ore-fed,
// undersized on measured delivery) and a PASS case (iron-gear-wheel@10/s).
describe("parseSimReport — real fixtures", () => {
  it("parses the FAIL fixture (ec@10/s)", () => {
    const report = parseSimReport(JSON.stringify(ec10Fail));
    expect(report.game_version).toBe("2.0.76");
    expect(report.report.overall_verdict).toBe("FAIL");
    expect(report.report.entities).toBe(805);
    const target = report.report.items.find((i) => i.is_target);
    expect(target?.item).toBe("electronic-circuit");
    expect(target?.verdict).toBe("FAIL");
    expect(report.sim_state.machines.length).toBeGreaterThan(0);
    expect(report.sim_state.belts.length).toBeGreaterThan(0);
    expect(report.sim_state.inserters.length).toBeGreaterThan(0);
  });

  it("parses the PASS fixture (gear@10/s)", () => {
    const report = parseSimReport(JSON.stringify(gear10Pass));
    expect(report.report.overall_verdict).toBe("PASS");
    const target = report.report.items.find((i) => i.is_target);
    expect(target?.item).toBe("iron-gear-wheel");
    expect(target?.verdict).toBe("PASS");
  });
});

describe("parseSimReport — defensive parsing", () => {
  it("rejects non-JSON with a readable message", () => {
    expect(() => parseSimReport("not json at all")).toThrowError(/not valid json/i);
  });

  it("rejects a JSON array (not an object)", () => {
    expect(() => parseSimReport("[1,2,3]")).toThrowError(/top level/i);
  });

  it("rejects an unrelated JSON object", () => {
    expect(() => parseSimReport(JSON.stringify({ hello: "world" }))).toThrowError(
      /not a valid sim report/i,
    );
  });

  it("rejects a report missing sim_state", () => {
    const partial: Record<string, unknown> = JSON.parse(JSON.stringify(gear10Pass));
    delete partial.sim_state;
    expect(() => parseSimReport(JSON.stringify(partial))).toThrowError(/sim_state/);
  });

  it("rejects a report with malformed belts", () => {
    const partial: any = JSON.parse(JSON.stringify(gear10Pass));
    partial.sim_state.belts = "not an array";
    expect(() => parseSimReport(JSON.stringify(partial))).toThrowError(/belts/);
  });

  it("rejects a report with malformed items", () => {
    const partial: any = JSON.parse(JSON.stringify(gear10Pass));
    partial.report.items = [{ item: "no-rate" }];
    expect(() => parseSimReport(JSON.stringify(partial))).toThrowError(/report\.items/);
  });
});
