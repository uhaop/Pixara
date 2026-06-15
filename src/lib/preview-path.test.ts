import { describe, expect, it } from "vitest";
import { queueItemThumbnailPath } from "@/lib/preview-path";
import type { UiQueueItem } from "@/lib/types";

function item(partial: Partial<UiQueueItem>): UiQueueItem {
  return {
    id: "1",
    batchId: "b",
    sourcePath: "C:\\temp\\pixara\\batch\\photo.heic",
    relativePath: "photo.heic",
    sourceFormat: "heic",
    sizeBytes: 100,
    status: "pending",
    selected: false,
    ...partial,
  };
}

describe("queueItemThumbnailPath", () => {
  it("always uses source path even when done", () => {
    const source = "C:\\temp\\pixara\\batch\\photo.heic";
    expect(
      queueItemThumbnailPath(
        item({
          status: "done",
          message: "D:\\photos\\photo.png",
        }),
      ),
    ).toBe(source);
  });
});
