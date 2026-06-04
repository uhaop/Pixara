import { describe, expect, it } from "vitest";
import {
  applySuffix,
  applyTemplate,
  normalizeEditedStem,
  outputStemForItem,
  sanitizeOutputStem,
} from "@/lib/rename";
import type { QueueItem } from "@/lib/types";

const sampleItem: QueueItem = {
  id: "1",
  batchId: "b",
  sourcePath: "C:/photos/nested/photo.png",
  relativePath: "nested/photo.png",
  sourceFormat: "png",
  sizeBytes: 100,
};

describe("rename", () => {
  it("sanitizes invalid windows characters", () => {
    expect(sanitizeOutputStem("a<b>")).toBe("a_b");
  });

  it("matches rust-style empty stem fallback", () => {
    expect(sanitizeOutputStem("<<<")).toBe("image");
    expect(sanitizeOutputStem("  hello__  ")).toBe("hello");
  });

  it("applies suffix", () => {
    expect(applySuffix("photo", "_web")).toBe("photo_web");
  });

  it("applies template tokens", () => {
    expect(
      applyTemplate("{name}_{index:03}", sampleItem, 2, "webp"),
    ).toBe("photo_002");
  });

  it("uses stem without source extension for display", () => {
    expect(outputStemForItem(sampleItem)).toBe("photo");
  });

  it("strips pasted extensions when normalizing inline edits", () => {
    expect(normalizeEditedStem("photo.png.webp", "webp")).toBe("photo");
    expect(normalizeEditedStem("IMG_0072.png", "webp")).toBe("IMG_0072");
  });
});
