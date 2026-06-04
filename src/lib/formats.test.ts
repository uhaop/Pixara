import { describe, expect, it } from "vitest";
import { basename, formatBytes } from "@/lib/formats";

describe("formatBytes", () => {
  it("returns placeholder for nullish and NaN values", () => {
    expect(formatBytes(null)).toBe("—");
    expect(formatBytes(undefined)).toBe("—");
    expect(formatBytes(Number.NaN)).toBe("—");
  });

  it("formats byte counts with readable units", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024)).toBe("1.00 KB");
    expect(formatBytes(1536)).toBe("1.50 KB");
    expect(formatBytes(5 * 1024 * 1024)).toBe("5.00 MB");
  });
});

describe("basename", () => {
  it("supports windows and unix style paths", () => {
    expect(basename("C:\\images\\photo.png")).toBe("photo.png");
    expect(basename("/tmp/archive/image.webp")).toBe("image.webp");
  });
});
