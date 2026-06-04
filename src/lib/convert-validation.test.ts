import { describe, expect, it } from "vitest";
import { validateBeforeConvert } from "@/lib/convert-validation";
import { DEFAULT_APP_CONFIG } from "@/lib/types";

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
});
