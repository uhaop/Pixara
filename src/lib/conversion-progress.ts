import { basename } from "@/lib/formats";
import type { ConvertProgress } from "@/lib/types";

export function progressPercent(progress: ConvertProgress | null): number {
  if (!progress || progress.total <= 0) {
    return 0;
  }
  const completed =
    progress.status === "converting"
      ? progress.current + 1
      : progress.current;
  return Math.min(100, Math.round((completed / progress.total) * 100));
}

export function progressFileName(progress: ConvertProgress): string {
  return basename(progress.sourcePath);
}

export function progressHeadline(progress: ConvertProgress): string {
  const name = progressFileName(progress);
  if (progress.status === "converting") {
    const fileNumber = Math.min(progress.current + 1, progress.total);
    return `Converting ${name} (${fileNumber} of ${progress.total})`;
  }
  if (progress.status === "done") {
    return `Finished ${name} (${progress.current} of ${progress.total})`;
  }
  if (progress.status === "skipped") {
    return `Skipped ${name} (${progress.current} of ${progress.total})`;
  }
  return `Failed ${name} (${progress.current} of ${progress.total})`;
}

export function progressDetail(progress: ConvertProgress): string {
  if (progress.status === "converting") {
    if (progress.workerId != null) {
      return `Worker ${progress.workerId} — large images can take a moment.`;
    }
    return "Working — this can take a moment for large images.";
  }
  if (progress.message) {
    return progress.message;
  }
  return progress.status;
}
