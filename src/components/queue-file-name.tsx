import { memo, useState } from "react";
import { extensionForFormat } from "@/lib/formats";
import { normalizeEditedStem, defaultStemFromItem, outputStemForItem } from "@/lib/rename";
import type { TargetImageFormat, UiQueueItem } from "@/lib/types";
import { cn } from "@/lib/utils";

type QueueFileNameProps = {
  item: UiQueueItem;
  toFormat: TargetImageFormat;
  disabled?: boolean;
  className?: string;
  onCommit: (id: string, outputBaseName: string | null) => void;
};

function QueueFileNameInner({
  item,
  toFormat,
  disabled,
  className,
  onCommit,
}: QueueFileNameProps) {
  const ext = extensionForFormat(toFormat);
  const stem = outputStemForItem(item);
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(stem);

  const canEdit = !disabled && item.status === "pending";

  function startEdit() {
    if (!canEdit) {
      return;
    }
    setDraft(stem);
    setEditing(true);
  }

  function commit() {
    setEditing(false);
    const next = normalizeEditedStem(draft, ext);
    const defaultStem = defaultStemFromItem(item);
    const outputBaseName = next === defaultStem ? null : next;
    const current = item.outputBaseName ?? null;
    if (outputBaseName === current) {
      return;
    }
    onCommit(item.id, outputBaseName);
  }

  if (editing) {
    return (
      <input
        className={cn(
          "w-full rounded border border-input bg-background px-1 py-0.5 text-xs",
          className,
        )}
        value={draft}
        autoFocus
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          e.stopPropagation();
          if (e.key === "Enter") {
            commit();
          }
          if (e.key === "Escape") {
            setDraft(stem);
            setEditing(false);
          }
        }}
        onClick={(e) => e.stopPropagation()}
      />
    );
  }

  return (
    <button
      type="button"
      className={cn(
        "max-w-full truncate text-left hover:underline disabled:cursor-default disabled:no-underline",
        className,
      )}
      title={item.sourcePath}
      disabled={!canEdit}
      onClick={startEdit}
    >
      <span>{stem}</span>
      <span className="text-muted-foreground">.{ext}</span>
    </button>
  );
}

export const QueueFileName = memo(QueueFileNameInner);
