import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { clearThumbnailCache } from "@/lib/thumbnail-cache";
import { validateBeforeConvert } from "@/lib/convert-validation";
import {
  shouldSampleEstimate,
  shouldWarnHeicToPng,
} from "@/lib/estimate-warning";
import { basename, formatBytes } from "@/lib/formats";
import { pendingTargets } from "@/lib/queue-view";
import type {
  AppConfig,
  BatchEstimate,
  ConvertProgress,
  ConvertSummary,
  IngestResult,
  ProgressStatus,
  QueueItem,
  SystemCapabilities,
  UiQueueItem,
  UiQueueItemStatus,
} from "@/lib/types";
import {
  DEFAULT_APP_CONFIG,
  appConfigToConvertSettings,
  queueItemToUi,
} from "@/lib/types";

function parentDir(filePath: string): string {
  const normalized = filePath.replace(/\\/g, "/");
  const lastSlash = normalized.lastIndexOf("/");
  if (lastSlash <= 0) {
    return filePath;
  }
  return filePath.slice(0, filePath.length - (normalized.length - lastSlash));
}

function mergeQueue(existing: UiQueueItem[], incoming: QueueItem[]): UiQueueItem[] {
  const byId = new Map(existing.map((item) => [item.id, item]));
  for (const item of incoming) {
    const prev = byId.get(item.id);
    byId.set(item.id, queueItemToUi(item, prev));
  }
  return Array.from(byId.values());
}

function progressToUiStatus(
  status: ProgressStatus,
  message: string,
): UiQueueItemStatus {
  if (status === "skipped" && message === "Cancelled") {
    return "pending";
  }
  return status;
}

function stripUiItem(item: UiQueueItem): QueueItem {
  const {
    status: _s,
    bytesAfter: _b,
    message: _m,
    selected: _sel,
    ...rest
  } = item;
  return rest;
}

export function useConverter() {
  const [config, setConfigState] = useState<AppConfig>(DEFAULT_APP_CONFIG);
  const [queue, setQueue] = useState<UiQueueItem[]>([]);
  const [progress, setProgress] = useState<ConvertProgress | null>(null);
  const [summary, setSummary] = useState<ConvertSummary | null>(null);
  const [estimate, setEstimate] = useState<BatchEstimate | null>(null);
  const [isConverting, setIsConverting] = useState(false);
  const [isIngesting, setIsIngesting] = useState(false);
  const [isBrowsing, setIsBrowsing] = useState(false);
  const [systemCaps, setSystemCaps] = useState<SystemCapabilities | null>(null);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const browseLockRef = useRef(false);
  const configRef = useRef(config);
  configRef.current = config;

  const persistConfig = useCallback(async (next: AppConfig) => {
    try {
      await invoke("save_config_cmd", { config: next });
    } catch (error) {
      console.error(error);
      toast.error("Could not save settings");
    }
  }, []);

  const updateConfig = useCallback(
    (patch: Partial<AppConfig>) => {
      setConfigState((current) => {
        const next = { ...current, ...patch };
        if (saveTimer.current) {
          clearTimeout(saveTimer.current);
        }
        saveTimer.current = setTimeout(() => {
          void persistConfig(next);
        }, 400);
        return next;
      });
      setEstimate(null);
    },
    [persistConfig],
  );

  const ingestPaths = useCallback(async (paths: string[]) => {
    if (paths.length === 0) {
      return;
    }
    setIsIngesting(true);
    try {
      const result = await invoke<IngestResult>("ingest_paths_cmd", {
        paths,
        fromFormat: configRef.current.fromFormat,
      });
      setQueue((current) => mergeQueue(current, result.items));
      setEstimate(null);
      if (result.skipped > 0) {
        toast.warning(`${result.skipped} path(s) skipped during ingest`);
      }
      if (result.truncated) {
        toast.warning("Some paths were not added (queue limit reached)");
      }
      if (result.items.length > 0) {
        toast.success(`Added ${result.items.length} file(s) to the queue`);
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setIsIngesting(false);
    }
  }, []);

  const runBrowse = useCallback(
    async (action: () => Promise<void>) => {
      if (browseLockRef.current || isConverting || isIngesting) {
        return;
      }
      browseLockRef.current = true;
      setIsBrowsing(true);
      try {
        await action();
      } finally {
        browseLockRef.current = false;
        setIsBrowsing(false);
      }
    },
    [isConverting, isIngesting],
  );

  const browseFiles = useCallback(async () => {
    await runBrowse(async () => {
      try {
        const paths = await invoke<string[]>("browse_files");
        if (paths.length === 0) {
          return;
        }
        await ingestPaths(paths);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : String(error));
      }
    });
  }, [ingestPaths, runBrowse]);

  const browseFolder = useCallback(async () => {
    await runBrowse(async () => {
      try {
        const folder = await invoke<string | null>("browse_folder");
        if (!folder) {
          return;
        }
        await ingestPaths([folder]);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : String(error));
      }
    });
  }, [ingestPaths, runBrowse]);

  const browseZip = useCallback(async () => {
    await runBrowse(async () => {
      try {
        const paths = await invoke<string[]>("browse_zip");
        if (paths.length === 0) {
          return;
        }
        await ingestPaths(paths);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : String(error));
      }
    });
  }, [ingestPaths, runBrowse]);

  const browseOutputDirectory = useCallback(async () => {
    await runBrowse(async () => {
      try {
        const selected = await invoke<string | null>("pick_output_dir");
        if (selected) {
          updateConfig({ customOutputDir: selected, outputMode: "customDir" });
        }
      } catch (error) {
        toast.error(error instanceof Error ? error.message : String(error));
      }
    });
  }, [runBrowse, updateConfig]);

  const removeItem = useCallback((id: string) => {
    setQueue((current) => current.filter((q) => q.id !== id));
    setEstimate(null);
  }, []);

  const toggleSelect = useCallback((id: string, selected: boolean) => {
    setQueue((current) =>
      current.map((item) => (item.id === id ? { ...item, selected } : item)),
    );
  }, []);

  const selectAll = useCallback((selected: boolean) => {
    setQueue((current) => current.map((item) => ({ ...item, selected })));
  }, []);

  const applyRename = useCallback(
    (updates: { id: string; outputBaseName: string | null }[]) => {
      const byId = new Map(updates.map((u) => [u.id, u.outputBaseName]));
      setQueue((current) =>
        current.map((item) => {
          const next = byId.get(item.id);
          if (next === undefined) {
            return item;
          }
          return { ...item, outputBaseName: next };
        }),
      );
      setEstimate(null);
      toast.success(`Renamed ${updates.length} output name(s)`);
    },
    [],
  );

  const inlineRename = useCallback((id: string, outputBaseName: string | null) => {
    setQueue((current) =>
      current.map((item) =>
        item.id === id ? { ...item, outputBaseName } : item,
      ),
    );
    setEstimate(null);
  }, []);

  const clearQueue = useCallback(async () => {
    const batchIds = [...new Set(queue.map((item) => item.batchId))];
    const itemIds = queue.map((item) => item.id);
    if (batchIds.length > 0) {
      try {
        await invoke("cleanup_temp_batches_cmd", { batchIds, itemIds });
      } catch (error) {
        console.error(error);
      }
    }
    setQueue([]);
    setProgress(null);
    setSummary(null);
    setEstimate(null);
    clearThumbnailCache();
  }, [queue]);

  const runConvert = useCallback(
    async (targets: UiQueueItem[]) => {
      if (targets.length === 0) {
        toast.message("No files to convert");
        return;
      }

      const cfg = configRef.current;
      const validationError = validateBeforeConvert(cfg);
      if (validationError) {
        toast.error(validationError);
        return;
      }

      const targetIds = new Set(targets.map((t) => t.id));
      const first = targets[0];
      setIsConverting(true);
      setProgress(null);
      setSummary(null);
      if (first) {
        setProgress({
          current: 0,
          total: targets.length,
          itemId: first.id,
          sourcePath: first.sourcePath,
          status: "converting",
          message: basename(first.sourcePath),
        });
        setQueue((current) =>
          current.map((item) => {
            if (item.id === first.id) {
              return { ...item, status: "converting" };
            }
            if (targetIds.has(item.id) && item.status === "converting") {
              return { ...item, status: "pending" };
            }
            return item;
          }),
        );
      }

      try {
        const settings = appConfigToConvertSettings(configRef.current);
        const items = targets.map(stripUiItem);
        const result = await invoke<ConvertSummary>("convert_batch", {
          items,
          settings,
        });
        setSummary(result);

        const packagingIssues = result.errors.length;
        if (result.failed === 0 && result.skipped === 0 && packagingIssues === 0) {
          toast.success(`Converted ${result.succeeded} file(s)`);
        } else if (result.succeeded > 0) {
          const parts = [`Converted ${result.succeeded}`];
          if (result.failed > 0) {
            parts.push(`${result.failed} failed`);
          }
          if (result.skipped > 0) {
            parts.push(`${result.skipped} skipped`);
          }
          if (packagingIssues > 0) {
            parts.push(`${packagingIssues} re-zip issue(s)`);
          }
          toast.warning(parts.join("; "));
        } else {
          toast.error(`Conversion finished with ${result.failed} failure(s)`);
        }

        await persistConfig(configRef.current);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : String(error));
        setQueue((current) =>
          current.map((item) =>
            targetIds.has(item.id) && item.status === "converting"
              ? { ...item, status: "pending" }
              : item,
          ),
        );
      } finally {
        setIsConverting(false);
        setProgress(null);
        setQueue((current) =>
          current.map((item) =>
            item.message === "Cancelled"
              ? {
                  ...item,
                  status: "pending",
                  message: undefined,
                  bytesAfter: undefined,
                }
              : item,
          ),
        );
      }
    },
    [persistConfig],
  );

  const convert = useCallback(async () => {
    const targets = pendingTargets(queue, false);
    await runConvert(targets);
  }, [queue, runConvert]);

  const convertSelected = useCallback(async () => {
    const targets = pendingTargets(queue, true);
    if (targets.length === 0) {
      toast.message("Select pending files to convert");
      return;
    }
    await runConvert(targets);
  }, [queue, runConvert]);

  const cancelConvert = useCallback(async () => {
    try {
      await invoke("cancel_convert_batch");
      toast.message("Cancelling batch…");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }, []);

  const retryFailed = useCallback(() => {
    setQueue((current) =>
      current.map((item) =>
        item.status === "error" ? { ...item, status: "pending" } : item,
      ),
    );
    setSummary(null);
    toast.message("Failed items moved back to pending");
  }, []);

  const dryRunEstimate = useCallback(async () => {
    const targets = pendingTargets(queue, queue.some((i) => i.selected));
    if (targets.length === 0) {
      toast.message("No pending files to estimate");
      return;
    }
    try {
      const settings = appConfigToConvertSettings(configRef.current);
      const accurateSample = shouldSampleEstimate(targets, configRef.current.toFormat);
      const result = await invoke<BatchEstimate>("estimate_batch_cmd", {
        items: targets.map(stripUiItem),
        settings,
        accurateSample,
      });
      setEstimate(result);
      const sizeLine = `Estimate: ${formatBytes(result.inputBytes)} → ~${formatBytes(result.estimatedOutputBytes)}`;
      if (result.sampled) {
        toast.message(`${sizeLine} (sampled ${Math.min(3, targets.length)} file(s))`);
      } else if (result.warning) {
        toast.message(sizeLine, { description: result.warning });
      } else if (
        result.lowConfidence ||
        shouldWarnHeicToPng(targets, configRef.current.toFormat)
      ) {
        toast.message(sizeLine, {
          description:
            "HEIC → PNG sizes are approximate. Run Estimate again for sample-based sizing.",
        });
      } else {
        toast.message(sizeLine);
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }, [queue]);

  const openOutputFolder = useCallback(async () => {
    const cfg = configRef.current;
    let target: string | null = null;
    if (cfg.outputMode === "customDir" && cfg.customOutputDir) {
      target = cfg.customOutputDir;
    } else {
      const lastDone = [...queue].reverse().find((item) => item.status === "done");
      if (lastDone?.message) {
        target = parentDir(lastDone.message);
      } else if (queue.length > 0) {
        target = parentDir(queue[0].sourcePath);
      }
    }
    if (!target) {
      toast.message("No output folder to open");
      return;
    }
    try {
      await invoke("open_folder", { path: target });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }, [queue]);

  useEffect(() => {
    void invoke<SystemCapabilities>("get_system_capabilities_cmd")
      .then(setSystemCaps)
      .catch((error) => console.error(error));
  }, []);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const loaded = await invoke<AppConfig>("load_config_cmd");
        if (cancelled) {
          return;
        }
        setConfigState({ ...DEFAULT_APP_CONFIG, ...loaded });
      } catch (error) {
        console.error(error);
        toast.error("Could not load saved settings");
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    const pending = new Map<string, ConvertProgress>();
    let rafId: number | null = null;

    const flush = () => {
      rafId = null;
      if (pending.size === 0) {
        return;
      }
      const updates = new Map(pending);
      pending.clear();

      let latest: ConvertProgress | undefined;
      for (const payload of updates.values()) {
        latest = payload;
      }
      if (!latest) {
        return;
      }

      setProgress(latest);
      setQueue((current) =>
        current.map((item) => {
          const payload = updates.get(item.id);
          if (!payload) {
            return item;
          }
          const status = progressToUiStatus(payload.status, payload.message);
          return {
            ...item,
            status,
            bytesAfter: payload.bytesAfter ?? item.bytesAfter,
            message:
              status === "pending" && payload.message === "Cancelled"
                ? undefined
                : payload.message || undefined,
          };
        }),
      );
    };

    const scheduleFlush = () => {
      if (rafId != null) {
        return;
      }
      rafId = requestAnimationFrame(flush);
    };

    void listen<ConvertProgress>("convert-progress", (event) => {
      pending.set(event.payload.itemId, event.payload);
      scheduleFlush();
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (rafId != null) {
        cancelAnimationFrame(rafId);
      }
      void unlisten?.();
    };
  }, []);

  return {
    config,
    updateConfig,
    queue,
    progress,
    summary,
    estimate,
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
  };
}
