import { memo } from "react";
import { ImageIcon, Trash2Icon } from "lucide-react";
import { QueueFileName } from "@/components/queue-file-name";
import { QueueThumbnail } from "@/components/queue-thumbnail";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { formatBytes, formatLabel } from "@/lib/formats";
import { queueItemThumbnailPath } from "@/lib/preview-path";
import type { TargetImageFormat, UiQueueItem, UiQueueItemStatus } from "@/lib/types";
import { cn } from "@/lib/utils";

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

type QueueListRowProps = {
  item: UiQueueItem;
  toFormat: TargetImageFormat;
  toLabel: string;
  showThumbnails: boolean;
  disabled?: boolean;
  onToggleSelect: (id: string, selected: boolean) => void;
  onRemoveItem: (id: string) => void;
  onInlineRename: (id: string, outputBaseName: string | null) => void;
};

function QueueListRowInner({
  item,
  toFormat,
  toLabel,
  showThumbnails,
  disabled,
  onToggleSelect,
  onRemoveItem,
  onInlineRename,
}: QueueListRowProps) {
  return (
    <tr
      className={cn(
        "border-b last:border-b-0",
        item.status === "converting" && "bg-primary/5",
      )}
    >
      <td className="px-2 py-2">
        <input
          type="checkbox"
          checked={item.selected}
          disabled={disabled}
          onChange={(e) => onToggleSelect(item.id, e.target.checked)}
        />
      </td>
      <td className="px-2 py-2">
        {showThumbnails ? (
          <QueueThumbnail
            itemId={item.id}
            sourcePath={queueItemThumbnailPath(item)}
            className="size-8"
          />
        ) : (
          <div className="flex size-8 items-center justify-center rounded-md bg-muted text-muted-foreground">
            <ImageIcon className="size-4" />
          </div>
        )}
      </td>
      <td className="max-w-[9rem] px-2 py-2">
        <QueueFileName
          item={item}
          toFormat={toFormat}
          disabled={disabled}
          onCommit={onInlineRename}
        />
      </td>
      <td className="px-2 py-2 text-muted-foreground">{formatLabel(item.sourceFormat)}</td>
      <td className="px-2 py-2 text-muted-foreground">{toLabel}</td>
      <td className="px-2 py-2">
        <Badge variant={statusVariant(item.status)}>{statusLabel[item.status]}</Badge>
      </td>
      <td className="px-2 py-2 text-right tabular-nums text-muted-foreground">
        {formatBytes(item.sizeBytes)}
      </td>
      <td
        className={cn(
          "px-2 py-2 text-right tabular-nums",
          item.bytesAfter != null && item.bytesAfter < item.sizeBytes
            ? "text-primary"
            : "text-muted-foreground",
        )}
      >
        {formatBytes(item.bytesAfter)}
      </td>
      <td className="px-2 py-2">
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
      </td>
    </tr>
  );
}

export const QueueListRow = memo(QueueListRowInner);
