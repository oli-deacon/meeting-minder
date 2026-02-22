import { describe, expect, it } from "vitest";
import { formatPercent, formatSeconds } from "../src/lib/format";

describe("format helpers", () => {
  it("formats seconds as mm:ss", () => {
    expect(formatSeconds(65)).toBe("01:05");
  });

  it("clamps negative seconds", () => {
    expect(formatSeconds(-10)).toBe("00:00");
  });

  it("formats percentages to one decimal", () => {
    expect(formatPercent(12.345)).toBe("12.3%");
  });
});
