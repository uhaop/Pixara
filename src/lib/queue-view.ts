import type { UiQueueItem, UiQueueItemStatus } from "@/lib/types";

export type QueueSortKey = "name" | "size" | "format" | "status";
export type QueueSortDir = "asc" | "desc";
export type QueueFilterStatus = "all" | UiQueueItemStatus;

export function sortQueue(
  items: UiQueueItem[],
  key: QueueSortKey,
  dir: QueueSortDir,
): UiQueueItem[] {
  const sorted = [...items].sort((a, b) => {
    let cmp = 0;
    switch (key) {
      case "name":
        cmp = a.relativePath.localeCompare(b.relativePath);
        break;
      case "size":
        cmp = a.sizeBytes - b.sizeBytes;
        break;
      case "format":
        cmp = a.sourceFormat.localeCompare(b.sourceFormat);
        break;
      case "status":
        cmp = a.status.localeCompare(b.status);
        break;
    }
    return dir === "asc" ? cmp : -cmp;
  });
  return sorted;
}

export function filterQueue(
  items: UiQueueItem[],
  status: QueueFilterStatus,
): UiQueueItem[] {
  if (status === "all") {
    return items;
  }
  return items.filter((item) => item.status === status);
}

export function selectedItems(items: UiQueueItem[]): UiQueueItem[] {
  return items.filter((item) => item.selected);
}

export function pendingTargets(
  items: UiQueueItem[],
  onlySelected: boolean,
): UiQueueItem[] {
  const pool = onlySelected ? selectedItems(items) : items;
  return pool.filter((item) => item.status === "pending");
}
