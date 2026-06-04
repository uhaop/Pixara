import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ArchiveIcon, FolderOpenIcon, ImagePlusIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { cn } from "@/lib/utils";

type DropZoneProps = {
  onPaths: (paths: string[]) => void | Promise<void>;
  onBrowseFiles: () => void | Promise<void>;
  onBrowseFolder: () => void | Promise<void>;
  onBrowseZip: () => void | Promise<void>;
  disabled?: boolean;
  compact?: boolean;
  className?: string;
};

function BrowseButtons({
  disabled,
  onBrowseFiles,
  onBrowseFolder,
  onBrowseZip,
}: Pick<
  DropZoneProps,
  "disabled" | "onBrowseFiles" | "onBrowseFolder" | "onBrowseZip"
>) {
  return (
    <>
      <Button
        type="button"
        variant="outline"
        size="sm"
        disabled={disabled}
        onClick={() => void onBrowseFiles()}
      >
        <ImagePlusIcon data-icon="inline-start" />
        Images
      </Button>
      <Button
        type="button"
        variant="outline"
        size="sm"
        disabled={disabled}
        onClick={() => void onBrowseFolder()}
      >
        <FolderOpenIcon data-icon="inline-start" />
        Folder
      </Button>
      <Button
        type="button"
        variant="outline"
        size="sm"
        disabled={disabled}
        onClick={() => void onBrowseZip()}
      >
        <ArchiveIcon data-icon="inline-start" />
        ZIP
      </Button>
    </>
  );
}

export function DropZone({
  onPaths,
  onBrowseFiles,
  onBrowseFolder,
  onBrowseZip,
  disabled,
  compact = false,
  className,
}: DropZoneProps) {
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void getCurrentWindow()
      .onDragDropEvent((event) => {
        if (disabled) {
          return;
        }
        const { type } = event.payload;
        if (type === "enter" || type === "over") {
          setIsDragging(true);
        } else if (type === "leave") {
          setIsDragging(false);
        } else if (type === "drop") {
          setIsDragging(false);
          void onPaths(event.payload.paths);
        }
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => {
      unlisten?.();
    };
  }, [disabled, onPaths]);

  if (compact) {
    return (
      <div
        className={cn(
          "flex shrink-0 flex-wrap items-center gap-2 border-b border-dashed px-3 py-2 transition-colors",
          isDragging && "border-primary bg-accent/40",
          disabled && "opacity-60",
          className,
        )}
      >
        <ImagePlusIcon className="size-4 shrink-0 text-muted-foreground" />
        <span className="min-w-0 flex-1 text-sm text-muted-foreground">
          Add more images, folders, or ZIP files
        </span>
        <BrowseButtons
          disabled={disabled}
          onBrowseFiles={onBrowseFiles}
          onBrowseFolder={onBrowseFolder}
          onBrowseZip={onBrowseZip}
        />
      </div>
    );
  }

  return (
    <div
      className={cn(
        "flex min-h-0 w-full flex-1 flex-col items-center justify-center gap-4 p-6 text-center transition-colors",
        isDragging && "bg-accent/40",
        disabled && "opacity-60",
        className,
      )}
    >
      <button
        type="button"
        disabled={disabled}
        aria-label="Add images"
        onClick={() => void onBrowseFiles()}
        className={cn(
          "flex w-full flex-col items-center gap-4 rounded-xl border border-dashed p-6 outline-none focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed",
          isDragging && "border-primary bg-accent/40",
        )}
      >
        <Empty className="border-0 p-0">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <ImagePlusIcon />
            </EmptyMedia>
            <EmptyTitle>Drop images, folders, or ZIP files</EmptyTitle>
            <EmptyDescription>
              Drag onto this window, or browse below.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      </button>
      <div className="flex flex-wrap justify-center gap-2">
        <BrowseButtons
          disabled={disabled}
          onBrowseFiles={onBrowseFiles}
          onBrowseFolder={onBrowseFolder}
          onBrowseZip={onBrowseZip}
        />
      </div>
    </div>
  );
}
