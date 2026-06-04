import { useMemo, useState } from "react";
import {
  LayoutGridIcon,
  ListIcon,
  PencilIcon,
  Trash2Icon,
} from "lucide-react";
import { RenameDialog } from "@/components/rename-dialog";
import { QueueFileName } from "@/components/queue-file-name";
import { QueueListRow } from "@/components/queue-list-row";
import { QueueThumbnail } from "@/components/queue-thumbnail";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { formatLabel } from "@/lib/formats";
import { queueItemThumbnailPath } from "@/lib/preview-path";
import {
  filterQueue,
  sortQueue,
  type QueueFilterStatus,
  type QueueSortKey,
} from "@/lib/queue-view";
import type { QueueView, TargetImageFormat, UiQueueItem, UiQueueItemStatus } from "@/lib/types";
import { cn } from "@/lib/utils";

type FileQueueProps = {
  items: UiQueueItem[];
  toFormat: TargetImageFormat;
  queueView: QueueView;
  onQueueViewChange: (view: QueueView) => void;
  disabled?: boolean;
  /** Pause thumbnail decode (e.g. during batch convert) to reduce CPU contention. */
  suspendThumbnails?: boolean;
  className?: string;
  onClear: () => void;
  onRemoveItem: (id: string) => void;
  onToggleSelect: (id: string, selected: boolean) => void;
  onSelectAll: (selected: boolean) => void;
  onRenameApply: (updates: { id: string; outputBaseName: string | null }[]) => void;
  onInlineRename: (id: string, outputBaseName: string | null) => void;
};

const statusLabel: Record<UiQueueItemStatus, string> = {
  pending: "Pending",
  converting: "Converting",
  done: "Done",
  skipped: "Skipped",
  error: "Error",
};

function statusVariant(status: UiQueueItemStatus) {
  switch (status) {
    case "done":
      return "secondary" as const;
    case "error":
      return "destructive" as const;
    case "converting":
      return "default" as const;
    default:
      return "outline" as const;
  }
}

export function FileQueue({
  items,
  toFormat,
  queueView,
  onQueueViewChange,
  disabled,
  suspendThumbnails = false,
  className,
  onClear,
  onRemoveItem,
  onToggleSelect,
  onSelectAll,
  onRenameApply,
  onInlineRename,
}: FileQueueProps) {
  const [sortKey, setSortKey] = useState<QueueSortKey>("name");
  const [filterStatus, setFilterStatus] = useState<QueueFilterStatus>("all");
  const [renameOpen, setRenameOpen] = useState(false);

  const toLabel = formatLabel(toFormat);
  const allSelected = items.length > 0 && items.every((item) => item.selected);
  const someSelected = items.some((item) => item.selected);

  const displayed = useMemo(
    () => sortQueue(filterQueue(items, filterStatus), sortKey, "asc"),
    [filterStatus, items, sortKey],
  );

  const renameTargets = useMemo(() => {
    const selected = items.filter((item) => item.selected && item.status === "pending");
    const pool = selected.length > 0 ? selected : items.filter((i) => i.status === "pending");
    return pool.map(({ id, batchId, sourcePath, relativePath, sourceFormat, sizeBytes, zipSourcePath, outputBaseName }) => ({
      id,
      batchId,
      sourcePath,
      relativePath,
      sourceFormat,
      sizeBytes,
      zipSourcePath,
      outputBaseName,
    }));
  }, [items]);

  if (items.length === 0) {
    return null;
  }

  return (
    <div className={cn("flex min-h-0 flex-1 flex-col gap-2 p-3 pt-2", className)}>
      <div className="flex flex-wrap items-center gap-2">
        <ToggleGroup
          variant="outline"
          spacing={0}
          value={[queueView]}
          onValueChange={(values) => {
            const next = values[values.length - 1] as QueueView | undefined;
            if (next) {
              onQueueViewChange(next);
            }
          }}
        >
          <ToggleGroupItem value="list" aria-label="List view">
            <ListIcon className="size-4" />
          </ToggleGroupItem>
          <ToggleGroupItem value="grid" aria-label="Grid view">
            <LayoutGridIcon className="size-4" />
          </ToggleGroupItem>
        </ToggleGroup>

        <select
          className="h-8 rounded-lg border border-input bg-transparent px-2 text-xs"
          value={sortKey}
          onChange={(e) => setSortKey(e.target.value as QueueSortKey)}
          disabled={disabled}
        >
          <option value="name">Sort: name</option>
          <option value="size">Sort: size</option>
          <option value="format">Sort: format</option>
          <option value="status">Sort: status</option>
        </select>

        <select
          className="h-8 rounded-lg border border-input bg-transparent px-2 text-xs"
          value={filterStatus}
          onChange={(e) => setFilterStatus(e.target.value as QueueFilterStatus)}
          disabled={disabled}
        >
          <option value="all">All statuses</option>
          <option value="pending">Pending</option>
          <option value="done">Done</option>
          <option value="error">Errors</option>
          <option value="skipped">Skipped</option>
        </select>

        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={disabled || renameTargets.length === 0}
          onClick={() => setRenameOpen(true)}
        >
          <PencilIcon data-icon="inline-start" />
          Rename…
        </Button>

        <span className="ml-auto text-sm text-muted-foreground">
          {items.length} in queue
          {someSelected ? ` (${items.filter((i) => i.selected).length} selected)` : ""}
        </span>

        <Button
          type="button"
          variant="ghost"
          size="sm"
          disabled={disabled}
          onClick={onClear}
        >
          <Trash2Icon data-icon="inline-start" />
          Clear
        </Button>
      </div>

      <label className="flex items-center gap-2 text-xs text-muted-foreground">
        <input
          type="checkbox"
          checked={allSelected}
          disabled={disabled}
          onChange={(e) => onSelectAll(e.target.checked)}
        />
        Select all
      </label>

      {queueView === "list" ? (
        <ScrollArea className="min-h-0 flex-1 rounded-lg ring-1 ring-foreground/10">
          <table className="w-full text-left text-xs">
            <thead className="sticky top-0 z-10 bg-muted/80 backdrop-blur">
              <tr className="border-b">
                <th className="w-8 px-2 py-2" />
                <th className="w-10 px-2 py-2" />
                <th className="px-2 py-2 font-medium">File</th>
                <th className="px-2 py-2 font-medium">From</th>
                <th className="px-2 py-2 font-medium">To</th>
                <th className="px-2 py-2 font-medium">Status</th>
                <th className="px-2 py-2 text-right font-medium">Size</th>
                <th className="px-2 py-2 text-right font-medium">After</th>
                <th className="w-8 px-2 py-2" />
              </tr>
            </thead>
            <tbody>
              {displayed.map((item) => (
                <QueueListRow
                  key={item.id}
                  item={item}
                  toFormat={toFormat}
                  toLabel={toLabel}
                  showThumbnails={!suspendThumbnails}
                  disabled={disabled}
                  onToggleSelect={onToggleSelect}
                  onRemoveItem={onRemoveItem}
                  onInlineRename={onInlineRename}
                />
              ))}
            </tbody>
          </table>
        </ScrollArea>
      ) : (
        <ScrollArea className="min-h-0 flex-1 rounded-lg ring-1 ring-foreground/10">
          <div className="grid grid-cols-2 gap-3 p-3 sm:grid-cols-3 md:grid-cols-4">
            {displayed.map((item) => (
              <div
                key={item.id}
                className="flex flex-col gap-2 rounded-lg border bg-card p-2 ring-1 ring-foreground/10"
              >
                <div className="flex items-start justify-between gap-1">
                  <input
                    type="checkbox"
                    checked={item.selected}
                    disabled={disabled}
                    onChange={(e) => onToggleSelect(item.id, e.target.checked)}
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon-sm"
                    disabled={disabled}
                    aria-label="Remove"
                    onClick={() => onRemoveItem(item.id)}
                  >
                    <Trash2Icon className="size-3.5" />
                  </Button>
                </div>
                <QueueThumbnail
                  itemId={item.id}
                  sourcePath={queueItemThumbnailPath(item)}
                  className="aspect-square w-full"
                  enabled={!suspendThumbnails}
                />
                <div className="flex flex-col gap-1">
                  <QueueFileName
                    item={item}
                    toFormat={toFormat}
                    disabled={disabled}
                    className="text-xs font-medium"
                    onCommit={onInlineRename}
                  />
                  <p className="text-[10px] text-muted-foreground">
                    {formatLabel(item.sourceFormat)} → {toLabel}
                  </p>
                  <Badge variant={statusVariant(item.status)} className="w-fit">
                    {statusLabel[item.status]}
                  </Badge>
                </div>
              </div>
            ))}
          </div>
        </ScrollArea>
      )}

      <RenameDialog
        open={renameOpen}
        items={renameTargets}
        toFormat={toFormat}
        onClose={() => setRenameOpen(false)}
        onApply={(updated) => {
          onRenameApply(
            updated.map((item) => ({
              id: item.id,
              outputBaseName: item.outputBaseName ?? null,
            })),
          );
        }}
      />
    </div>
  );
}
