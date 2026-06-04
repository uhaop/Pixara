import { useState } from "react";
import {
  CircleHelpIcon,
  FolderOpenIcon,
  Loader2Icon,
  RotateCcwIcon,
  SettingsIcon,
  SquareIcon,
} from "lucide-react";
import appLogo from "@/assets/gv-logo.png";
import { RightSidebar, type SidebarTab } from "@/components/right-sidebar";
import { ConversionStatusBar } from "@/components/conversion-status-bar";
import { DropZone } from "@/components/drop-zone";
import { FileQueue } from "@/components/file-queue";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useConverter } from "@/hooks/use-converter";
import { basename } from "@/lib/formats";

export default function App() {
  const [sidebarTab, setSidebarTab] = useState<SidebarTab>("conversion");
  const {
    config,
    updateConfig,
    queue,
    progress,
    summary,
    isConverting,
    isIngesting,
    isBrowsing,
    ingestPaths,
    browseFiles,
    browseFolder,
    browseZip,
    browseOutputDirectory,
    clearQueue,
    removeItem,
    toggleSelect,
    selectAll,
    applyRename,
    inlineRename,
    convert,
    convertSelected,
    cancelConvert,
    retryFailed,
    dryRunEstimate,
    openOutputFolder,
    systemCaps,
  } = useConverter();

  const helpText = systemCaps
    ? `${systemCaps.decodeNote} ${systemCaps.encodeNote} Thumbnails stay cached while you scroll. Clear queue to free temp files.`
    : "Drop images, folders, or ZIP archives. Thumbnails stay cached while you scroll.";

  const busy = isConverting || isIngesting || isBrowsing;
  const hasQueue = queue.length > 0;
  const hasErrors = queue.some((item) => item.status === "error");
  const hasSelection = queue.some((item) => item.selected);

  return (
    <div className="flex h-svh flex-col overflow-hidden">
      <header className="flex shrink-0 items-center justify-between gap-3 border-b px-4 py-2">
        <h1 className="m-0 flex min-w-0 items-center gap-2 p-0">
          <img
            src={appLogo}
            alt=""
            aria-hidden
            width={28}
            height={28}
            className="h-7 w-auto shrink-0 object-contain"
          />
          <span className="text-base font-semibold tracking-tight">Pixara</span>
        </h1>
        <div className="flex items-center gap-1">
          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  type="button"
                  variant={sidebarTab === "settings" ? "secondary" : "ghost"}
                  size="icon-sm"
                  onClick={() => setSidebarTab("settings")}
                />
              }
            >
              <SettingsIcon />
              <span className="sr-only">Settings</span>
            </TooltipTrigger>
            <TooltipContent side="bottom">Saved preferences</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger
              render={<Button type="button" variant="ghost" size="icon-sm" />}
            >
              <CircleHelpIcon />
              <span className="sr-only">Help</span>
            </TooltipTrigger>
            <TooltipContent side="bottom" className="max-w-xs">
              {helpText}
            </TooltipContent>
          </Tooltip>
        </div>
      </header>

      <ConversionStatusBar
        progress={progress}
        isConverting={isConverting}
        summary={summary}
        systemCaps={systemCaps}
        slowDriveMode={config.slowDriveMode}
      />

      <main className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_17.5rem] gap-3 p-3">
        <section className="flex min-h-0 flex-col overflow-hidden rounded-xl bg-card ring-1 ring-foreground/10">
          <DropZone
            disabled={busy}
            compact={hasQueue}
            onBrowseFiles={browseFiles}
            onBrowseFolder={browseFolder}
            onBrowseZip={browseZip}
            onPaths={ingestPaths}
          />
          {hasQueue && (
            <FileQueue
              items={queue}
              toFormat={config.toFormat}
              queueView={config.queueView}
              onQueueViewChange={(queueView) => updateConfig({ queueView })}
              disabled={busy}
              suspendThumbnails={isConverting}
              onClear={() => void clearQueue()}
              onRemoveItem={removeItem}
              onToggleSelect={toggleSelect}
              onSelectAll={selectAll}
              onRenameApply={applyRename}
              onInlineRename={inlineRename}
            />
          )}
        </section>

        <aside className="flex min-h-0 flex-col overflow-hidden rounded-xl bg-card ring-1 ring-foreground/10">
          <RightSidebar
            activeTab={sidebarTab}
            onActiveTabChange={setSidebarTab}
            config={config}
            queue={queue}
            onConfigChange={updateConfig}
            onBrowseOutputDirectory={browseOutputDirectory}
          />
        </aside>
      </main>

      <footer className="shrink-0 border-t bg-card px-4 py-2.5">
        <div className="flex flex-col gap-2">
          {summary && !isConverting && summary.errors.length > 0 && (
            <ul className="max-h-16 list-disc overflow-y-auto pl-5 text-xs text-destructive">
              {summary.errors.map((error) => (
                <li key={error.itemId || error.sourcePath}>
                  {basename(error.sourcePath)}: {error.message}
                </li>
              ))}
            </ul>
          )}

          <div className="flex flex-wrap items-center gap-2">
            <Button
              type="button"
              disabled={busy || queue.length === 0}
              onClick={() => void convert()}
            >
              {isConverting ? (
                <Loader2Icon className="animate-spin" data-icon="inline-start" />
              ) : null}
              Convert all
            </Button>

            <Button
              type="button"
              variant="secondary"
              disabled={busy || !hasSelection}
              onClick={() => void convertSelected()}
            >
              Convert selected
            </Button>

            {isConverting && (
              <Button
                type="button"
                variant="outline"
                onClick={() => void cancelConvert()}
              >
                <SquareIcon data-icon="inline-start" />
                Cancel
              </Button>
            )}

            <Button
              type="button"
              variant="outline"
              disabled={busy || queue.length === 0}
              onClick={() => void dryRunEstimate()}
            >
              Estimate
            </Button>

            {hasErrors && (
              <Button
                type="button"
                variant="outline"
                disabled={busy}
                onClick={retryFailed}
              >
                <RotateCcwIcon data-icon="inline-start" />
                Retry failed
              </Button>
            )}

            <Tooltip>
              <TooltipTrigger
                render={
                  <Button
                    type="button"
                    variant="outline"
                    disabled={!summary && !isConverting}
                    onClick={() => void openOutputFolder()}
                  />
                }
              >
                <FolderOpenIcon data-icon="inline-start" />
                Open output folder
              </TooltipTrigger>
              <TooltipContent>Open the conversion output directory</TooltipContent>
            </Tooltip>
          </div>
        </div>
      </footer>
    </div>
  );
}
