import { Loader2Icon } from "lucide-react";
import { Progress, ProgressLabel } from "@/components/ui/progress";
import {
  progressDetail,
  progressHeadline,
  progressPercent,
} from "@/lib/conversion-progress";
import type { ConvertProgress, ConvertSummary, SystemCapabilities } from "@/lib/types";

type ConversionStatusBarProps = {
  progress: ConvertProgress | null;
  isConverting: boolean;
  summary: ConvertSummary | null;
  systemCaps: SystemCapabilities | null;
  slowDriveMode?: boolean;
};

export function ConversionStatusBar({
  progress,
  isConverting,
  summary,
  systemCaps,
  slowDriveMode = false,
}: ConversionStatusBarProps) {
  if (isConverting) {
    const percent = progressPercent(progress);
    const headline = progress
      ? progressHeadline(progress)
      : "Preparing conversion…";
    const detail = progress ? progressDetail(progress) : "Starting batch";
    const workerCount =
      systemCaps &&
      (slowDriveMode
        ? Math.min(systemCaps.convertWorkers, 2)
        : systemCaps.convertWorkers);

    return (
      <div
        className="shrink-0 border-b bg-muted/30 px-4 py-3"
        role="status"
        aria-live="polite"
        aria-busy="true"
      >
        <div className="flex flex-col gap-2">
          <div className="flex items-start gap-2">
            <Loader2Icon className="mt-0.5 size-4 shrink-0 animate-spin text-primary" />
            <div className="min-w-0 flex-1">
              <p className="truncate text-sm font-medium">{headline}</p>
              <p className="truncate text-xs text-muted-foreground">{detail}</p>
              {systemCaps && workerCount != null && (
                <p className="truncate text-xs text-muted-foreground/80">
                  {workerCount} parallel worker
                  {workerCount === 1 ? "" : "s"}
                  {slowDriveMode ? " (slow drive)" : ""} ·{" "}
                  {systemCaps.logicalCpus} CPU threads
                  {systemCaps.gpuAdapterName
                    ? ` · ${systemCaps.gpuAdapterName}`
                    : systemCaps.gpuDetected
                      ? " · GPU detected"
                      : ""}
                </p>
              )}
            </div>
            <span className="shrink-0 text-xs tabular-nums text-muted-foreground">
              {progress ? `${progressPercent(progress)}%` : "…"}
            </span>
          </div>
          <Progress value={percent} className="w-full">
            <ProgressLabel className="sr-only">{headline}</ProgressLabel>
          </Progress>
        </div>
      </div>
    );
  }

  if (summary) {
    return (
      <div
        className="shrink-0 border-b bg-muted/20 px-4 py-2"
        role="status"
        aria-live="polite"
      >
        <p className="text-sm text-muted-foreground">
          Batch complete — {summary.succeeded} succeeded
          {summary.skipped > 0 ? `, ${summary.skipped} skipped` : ""}
          {summary.failed > 0 ? `, ${summary.failed} failed` : ""}
        </p>
      </div>
    );
  }

  return null;
}
