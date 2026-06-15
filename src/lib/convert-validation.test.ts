import { describe, expect, it } from "vitest";
import { validateBeforeConvert } from "@/lib/convert-validation";
import { DEFAULT_APP_CONFIG } from "@/lib/types";
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

describe("convert-validation", () => {
  it("requires custom output directory when mode is customDir", () => {
    expect(
      validateBeforeConvert({
        ...DEFAULT_APP_CONFIG,
        outputMode: "customDir",
        customOutputDir: null,
      }),
    ).toBe("Choose an output folder before converting");
  });

  it("allows convert when custom output directory is set", () => {
    expect(
      validateBeforeConvert({
        ...DEFAULT_APP_CONFIG,
        outputMode: "customDir",
        customOutputDir: "C:\\output",
      }),
    ).toBeNull();
  });

  it("allows same-folder output without custom directory", () => {
    expect(
      validateBeforeConvert({
        ...DEFAULT_APP_CONFIG,
        outputMode: "sameFolder",
        customOutputDir: null,
      }),
    ).toBeNull();
  });

  it("rejects zero resize dimensions", () => {
    expect(
      validateBeforeConvert({
        ...DEFAULT_APP_CONFIG,
        maxWidth: 0,
      }),
    ).toBe("Max width and height must be greater than zero");
  });

  it("rejects HEIC export when the build cannot write HEIC", () => {
    expect(
      validateBeforeConvert(
        { ...DEFAULT_APP_CONFIG, toFormat: "heic" },
        CAPS_NO_HEIC,
      ),
    ).toBe("HEIC export is not available in this build");
  });

  it("rejects HEIC input filter when the build cannot read HEIC", () => {
    expect(
      validateBeforeConvert(
        { ...DEFAULT_APP_CONFIG, fromFormat: "heic" },
        CAPS_NO_HEIC,
      ),
    ).toBe("HEIC input is not available in this build");
  });

  it("rejects queued HEIC files when the build cannot read HEIC", () => {
    expect(
      validateBeforeConvert(DEFAULT_APP_CONFIG, CAPS_NO_HEIC, [
        { sourceFormat: "heic" },
      ]),
    ).toBe("Queue contains HEIC files but this build cannot read HEIC");
  });

  it("rejects HEIC export while capabilities are still loading", () => {
    expect(
      validateBeforeConvert({ ...DEFAULT_APP_CONFIG, toFormat: "heic" }, null),
    ).toBe("HEIC export is not available in this build");
  });

  it("rejects HEIC input while capabilities are still loading", () => {
    expect(
      validateBeforeConvert({ ...DEFAULT_APP_CONFIG, fromFormat: "heic" }, null),
    ).toBe("HEIC input is not available in this build");
  });

  it("allows non-HEIC convert while capabilities are still loading", () => {
    expect(validateBeforeConvert(DEFAULT_APP_CONFIG, null)).toBeNull();
  });
});
