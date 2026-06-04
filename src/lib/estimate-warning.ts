import type { TargetImageFormat, UiQueueItem } from "@/lib/types";

/** Queue is mostly HEIC while exporting to PNG — ratio estimates are unreliable. */
export function shouldWarnHeicToPng(
  queue: UiQueueItem[],
  toFormat: TargetImageFormat,
): boolean {
  if (toFormat !== "png" || queue.length === 0) {
    return false;
  }
  const heicCount = queue.filter((item) => item.sourceFormat === "heic").length;
  return heicCount * 2 > queue.length;
}

export function shouldSampleEstimate(
  queue: UiQueueItem[],
  toFormat: TargetImageFormat,
): boolean {
  if (toFormat !== "png" || queue.length === 0) {
    return false;
  }
  return queue.some((item) => item.sourceFormat === "heic");
}
