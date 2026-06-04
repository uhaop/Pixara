import { describe, expect, it } from "vitest";
import { parsePositiveDimension } from "@/lib/dimensions";

describe("parsePositiveDimension", () => {
  it("returns null for empty or invalid values", () => {
    expect(parsePositiveDimension("")).toBeNull();
    expect(parsePositiveDimension("   ")).toBeNull();
    expect(parsePositiveDimension("abc")).toBeNull();
    expect(parsePositiveDimension("0")).toBeNull();
    expect(parsePositiveDimension("-10")).toBeNull();
  });

  it("returns positive integers", () => {
    expect(parsePositiveDimension("1920")).toBe(1920);
    expect(parsePositiveDimension(" 800 ")).toBe(800);
  });
});
