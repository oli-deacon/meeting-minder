import { describe, expect, it } from "vitest";
import { pickSessionStarters } from "../src/lib/statementStarters";

describe("pickSessionStarters", () => {
  it("returns at most requested count", () => {
    const source = ["a", "b", "c", "d"];
    const picked = pickSessionStarters(source, 3);
    expect(picked).toHaveLength(3);
  });

  it("returns unique prompts", () => {
    const source = ["a", "b", "c", "d", "e"];
    const picked = pickSessionStarters(source, 5);
    const unique = new Set(picked);
    expect(unique.size).toBe(picked.length);
  });

  it("does not mutate the input array", () => {
    const source = ["a", "b", "c", "d"];
    const original = [...source];
    void pickSessionStarters(source, 2);
    expect(source).toEqual(original);
  });

  it("handles count larger than source length", () => {
    const source = ["a", "b", "c"];
    const picked = pickSessionStarters(source, 10);
    expect(picked).toHaveLength(3);
  });

  it("handles empty source safely", () => {
    const picked = pickSessionStarters([], 5);
    expect(picked).toEqual([]);
  });
});
