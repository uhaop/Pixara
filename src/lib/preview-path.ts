import type { UiQueueItem } from "@/lib/types";

/**
 * Queue thumbnails always use the original source path so previews stay stable
 * when status changes (e.g. pending → done) and we do not re-decode on every row update.
 */
export function queueItemThumbnailPath(item: UiQueueItem): string {
  return item.sourcePath;
}
