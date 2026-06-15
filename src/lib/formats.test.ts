import { describe, expect, it } from "vitest";
import {
  basename,
  formatBytes,
  formatOptionsForCapabilities,
  FROM_FORMAT_OPTIONS,
  TO_FORMAT_OPTIONS,
} from "@/lib/formats";
import type { SystemCapabilities } from "@/lib/types";

const CAPS_NO_HEIC: SystemCapabilities = {
  logicalCpus: 8,
  convertWorkers: 4,
  heicReadAvailable: false,
  heicWriteAvailable: false,
  gpuDetected: false,
  convertBackend: "cpuParallel",
  heicDecodeBackend: "unknown",
  ffmpegHwaccels: [],
  backendNote: "",
  decodeNote: "",
  encodeNote: "",
};

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

describe("formatOptionsForCapabilities", () => {
  it("hides HEIC when the build cannot read or write it", () => {
    const options = formatOptionsForCapabilities(CAPS_NO_HEIC);
    expect(options.from.some((o) => o.value === "heic")).toBe(false);
    expect(options.to.some((o) => o.value === "heic")).toBe(false);
    expect(options.from.length).toBe(FROM_FORMAT_OPTIONS.length - 1);
    expect(options.to.length).toBe(TO_FORMAT_OPTIONS.length - 1);
  });

  it("hides HEIC while capabilities are still loading", () => {
    const options = formatOptionsForCapabilities(null);
    expect(options.from.some((o) => o.value === "heic")).toBe(false);
    expect(options.to.some((o) => o.value === "heic")).toBe(false);
  });
});

describe("basename", () => {
  it("supports windows and unix style paths", () => {
    expect(basename("C:\\images\\photo.png")).toBe("photo.png");
    expect(basename("/tmp/archive/image.webp")).toBe("image.webp");
  });
});
