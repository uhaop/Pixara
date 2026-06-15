import { describe, expect, it } from "vitest";
import {
  sanitizeConfigForCapabilities,
  selectFormatValues,
} from "@/lib/config-capabilities";
import { DEFAULT_APP_CONFIG } from "@/lib/types";
import type { AppConfig, SystemCapabilities } from "@/lib/types";

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

describe("sanitizeConfigForCapabilities", () => {
  it("strips HEIC formats when the build cannot read or write them", () => {
    const sanitized = sanitizeConfigForCapabilities(
      { ...DEFAULT_APP_CONFIG, fromFormat: "heic", toFormat: "heic" },
      CAPS_NO_HEIC,
    );
    expect(sanitized.fromFormat).toBe("any");
    expect(sanitized.toFormat).toBe("webp");
  });

  it("leaves config unchanged while capabilities are still loading", () => {
    const input: AppConfig = {
      ...DEFAULT_APP_CONFIG,
      fromFormat: "heic",
      toFormat: "heic",
    };
    expect(sanitizeConfigForCapabilities(input, null)).toEqual(input);
  });

  it("uses safe select values while capabilities are still loading", () => {
    const input: AppConfig = {
      ...DEFAULT_APP_CONFIG,
      fromFormat: "heic",
      toFormat: "heic",
    };
    expect(selectFormatValues(input, null)).toEqual({
      fromFormat: "any",
      toFormat: "webp",
    });
  });
});
