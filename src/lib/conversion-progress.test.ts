import { describe, expect, it } from "vitest";
import {
  progressDetail,
  progressHeadline,
  progressPercent,
} from "@/lib/conversion-progress";
import type { ConvertProgress } from "@/lib/types";

const base: ConvertProgress = {
  current: 2,
  total: 20,
  itemId: "1",
  sourcePath: "C:\\photos\\IMG_0073.png",
  status: "converting",
  message: "IMG_0073.png",
};

describe("conversion-progress", () => {
  it("shows file number while converting", () => {
    expect(progressHeadline(base)).toBe("Converting IMG_0073.png (3 of 20)");
  });

  it("uses in-progress count for progress bar while converting", () => {
    expect(progressPercent(base)).toBe(15);
    expect(progressPercent({ ...base, current: 0, total: 3, status: "converting" })).toBe(
      33,
    );
    expect(progressPercent({ ...base, current: 20, status: "done" })).toBe(100);
  });

  it("includes worker id while converting", () => {
    expect(progressDetail({ ...base, workerId: 2 })).toContain("Worker 2");
  });
});
