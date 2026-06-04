import { describe, expect, it } from "vitest";
import { shouldSampleEstimate, shouldWarnHeicToPng } from "@/lib/estimate-warning";
import type { UiQueueItem } from "@/lib/types";

function item(sourceFormat: UiQueueItem["sourceFormat"]): UiQueueItem {
  return {
    id: "1",
    batchId: "b",
    sourcePath: "/a",
    relativePath: "a",
    sourceFormat,
    sizeBytes: 100,
    status: "pending",
    selected: false,
  };
}

describe("shouldWarnHeicToPng", () => {
  it("warns when majority HEIC and target PNG", () => {
    expect(
      shouldWarnHeicToPng([item("heic"), item("heic"), item("png")], "png"),
    ).toBe(true);
  });

  it("does not warn for JPEG target", () => {
    expect(shouldWarnHeicToPng([item("heic")], "jpeg")).toBe(false);
  });
});

describe("shouldSampleEstimate", () => {
  it("samples when any HEIC to PNG", () => {
    expect(shouldSampleEstimate([item("heic"), item("png")], "png")).toBe(true);
  });
});
