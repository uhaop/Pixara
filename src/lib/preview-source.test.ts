import { describe, expect, it } from "vitest";
import { supportsNativePreview } from "@/lib/preview-source";

describe("preview-source", () => {
  it("allows native preview for common web formats", () => {
    expect(supportsNativePreview("C:\\photos\\IMG_0072.png")).toBe(true);
    expect(supportsNativePreview("/tmp/photo.JPG")).toBe(true);
    expect(supportsNativePreview("x.webp")).toBe(true);
  });

  it("requires generated thumbnails for heic and tiff", () => {
    expect(supportsNativePreview("photo.heic")).toBe(false);
    expect(supportsNativePreview("scan.tiff")).toBe(false);
  });
});
