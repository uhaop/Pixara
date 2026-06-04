import { useEffect, useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import {
  applyRenameToItems,
  previewOutputBasenames,
  type RenameMode,
} from "@/lib/rename";
import { extensionForFormat } from "@/lib/formats";
import type { QueueItem, TargetImageFormat } from "@/lib/types";

type RenameDialogProps = {
  open: boolean;
  items: QueueItem[];
  toFormat: TargetImageFormat;
  onClose: () => void;
  onApply: (updated: QueueItem[]) => void;
};

export function RenameDialog({
  open,
  items,
  toFormat,
  onClose,
  onApply,
}: RenameDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [mode, setMode] = useState<RenameMode>("suffix");
  const [value, setValue] = useState("_web");

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) {
      return;
    }
    if (open) {
      dialog.showModal();
    } else {
      dialog.close();
    }
  }, [open]);

  const previews = useMemo(
    () => previewOutputBasenames(items, mode, value, toFormat),
    [items, mode, value, toFormat],
  );

  const ext = extensionForFormat(toFormat);

  function handleApply() {
    const updated = applyRenameToItems(items, mode, value, toFormat);
    onApply(updated);
    onClose();
  }

  return (
    <dialog
      ref={dialogRef}
      className="fixed top-1/2 left-1/2 w-[min(100%,28rem)] -translate-x-1/2 -translate-y-1/2 rounded-xl border bg-background p-4 shadow-lg backdrop:bg-black/40"
      onClose={onClose}
    >
      <form
        method="dialog"
        className="flex flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          handleApply();
        }}
      >
        <div>
          <h2 className="font-heading text-base font-medium">Rename outputs</h2>
          <p className="text-sm text-muted-foreground">
            Changes output filenames only ({items.length} file
            {items.length === 1 ? "" : "s"}). Use <strong>Template</strong> to rename
            every file at once (e.g. {"{name}_{index:03}"} → photo_001, photo_002).
          </p>
        </div>

        <ToggleGroup
          variant="outline"
          spacing={0}
          value={[mode]}
          onValueChange={(values) => {
            const next = values[values.length - 1] as RenameMode | undefined;
            if (next) {
              setMode(next);
            }
          }}
        >
          <ToggleGroupItem className="flex-1" value="suffix">
            Suffix
          </ToggleGroupItem>
          <ToggleGroupItem className="flex-1" value="template">
            Template
          </ToggleGroupItem>
        </ToggleGroup>

        <div className="flex flex-col gap-2">
          <Label htmlFor="rename-value">
            {mode === "suffix" ? "Suffix" : "Template"}
          </Label>
          <input
            id="rename-value"
            className="flex h-8 w-full rounded-lg border border-input bg-transparent px-2.5 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            value={value}
            onChange={(event) => setValue(event.target.value)}
            placeholder={mode === "suffix" ? "_web" : "{name}_{index:03}"}
          />
          {mode === "template" && (
            <p className="text-xs text-muted-foreground">
              Tokens: {"{name}"}, {"{index}"}, {"{index:03}"}, {"{ext}"},{" "}
              {"{parent}"}, {"{relative}"}
            </p>
          )}
        </div>

        <div className="rounded-lg bg-muted/50 p-3 text-xs">
          <p className="mb-1 font-medium">Preview</p>
          <ul className="flex flex-col gap-1 text-muted-foreground">
            {previews.map((stem) => (
              <li key={stem} className="truncate font-mono">
                {stem}.{ext}
              </li>
            ))}
            {items.length > previews.length && (
              <li>…and {items.length - previews.length} more</li>
            )}
          </ul>
        </div>

        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit">Apply</Button>
        </div>
      </form>
    </dialog>
  );
}
