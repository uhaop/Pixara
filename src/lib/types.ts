export type ImageFormat =
  | "any"
  | "png"
  | "jpeg"
  | "webp"
  | "heic"
  | "gif"
  | "bmp"
  | "tiff"
  | "avif";

export type TargetImageFormat = Exclude<ImageFormat, "any">;

export type Preset = "web" | "high" | "smallest";

export type OutputMode = "sameFolder" | "customDir";

export type NamingMode = "replaceExtension" | "appendSuffix";

export type OverwriteMode = "autoRename" | "replace" | "skip";

export type QueueView = "list" | "grid";

export type ProgressStatus = "converting" | "skipped" | "done" | "error";

export type UiQueueItemStatus = "pending" | ProgressStatus;

export interface QueueItem {
  id: string;
  batchId: string;
  sourcePath: string;
  relativePath: string;
  sourceFormat: ImageFormat;
  sizeBytes: number;
  zipSourcePath?: string | null;
  outputBaseName?: string | null;
}

export interface UiQueueItem extends QueueItem {
  status: UiQueueItemStatus;
  bytesAfter?: number;
  message?: string;
  selected: boolean;
}

export interface ConvertSettings {
  toFormat: TargetImageFormat;
  preset: Preset;
  outputMode: OutputMode;
  customOutputDir?: string | null;
  preserveStructure: boolean;
  naming: NamingMode;
  maxWidth?: number | null;
  maxHeight?: number | null;
  skipSameFormat: boolean;
  stripIcc: boolean;
  rezipOutputs: boolean;
  flattenColor: string;
  overwriteMode: OverwriteMode;
  optimizePng: boolean;
  slowDriveMode?: boolean;
}

export interface AppConfig {
  fromFormat: ImageFormat;
  toFormat: TargetImageFormat;
  preset: Preset;
  outputMode: OutputMode;
  customOutputDir?: string | null;
  preserveStructure: boolean;
  naming: NamingMode;
  maxWidth?: number | null;
  maxHeight?: number | null;
  skipSameFormat: boolean;
  stripIcc: boolean;
  rezipOutputs: boolean;
  flattenColor: string;
  overwriteMode: OverwriteMode;
  /** When exporting PNG, run oxipng after encode (slower, smaller). */
  optimizePng: boolean;
  /** Cap parallel workers to reduce disk contention (USB / network drives). */
  slowDriveMode: boolean;
  queueView: QueueView;
}

export interface IngestResult {
  batchId: string;
  items: QueueItem[];
  skipped: number;
  truncated: boolean;
}

export interface BatchEstimate {
  inputBytes: number;
  estimatedOutputBytes: number;
  previewPaths: string[];
  lowConfidence?: boolean;
  warning?: string;
  sampled?: boolean;
}

export interface ConvertProgress {
  current: number;
  total: number;
  itemId: string;
  sourcePath: string;
  status: ProgressStatus;
  message: string;
  bytesAfter?: number;
  workerId?: number;
}

export interface ConvertErrorEntry {
  itemId: string;
  sourcePath: string;
  message: string;
}

export interface ConvertSummary {
  succeeded: number;
  failed: number;
  skipped: number;
  errors: ConvertErrorEntry[];
}

export type ConvertBackend = "cpuParallel" | "gpuAssisted";

export type HeicDecodeBackend =
  | "cpu"
  | "mediaFoundation"
  | "ffmpegHw"
  | "unknown";

export interface SystemCapabilities {
  logicalCpus: number;
  convertWorkers: number;
  heicReadAvailable: boolean;
  heicWriteAvailable: boolean;
  gpuDetected: boolean;
  gpuAdapterName?: string;
  convertBackend: ConvertBackend;
  heicDecodeBackend: HeicDecodeBackend;
  ffmpegHwaccels: string[];
  backendNote: string;
  decodeNote: string;
  encodeNote: string;
}

export const DEFAULT_APP_CONFIG: AppConfig = {
  fromFormat: "any",
  toFormat: "webp",
  preset: "web",
  outputMode: "sameFolder",
  customOutputDir: null,
  preserveStructure: true,
  naming: "replaceExtension",
  maxWidth: null,
  maxHeight: null,
  skipSameFormat: true,
  stripIcc: false,
  rezipOutputs: false,
  flattenColor: "#ffffff",
  overwriteMode: "autoRename",
  optimizePng: true,
  slowDriveMode: false,
  queueView: "list",
};

export function appConfigToConvertSettings(config: AppConfig): ConvertSettings {
  return {
    toFormat: config.toFormat,
    preset: config.preset,
    outputMode: config.outputMode,
    customOutputDir: config.customOutputDir ?? null,
    preserveStructure: config.preserveStructure,
    naming: config.naming,
    maxWidth: config.maxWidth ?? null,
    maxHeight: config.maxHeight ?? null,
    skipSameFormat: config.skipSameFormat,
    stripIcc: config.stripIcc,
    rezipOutputs: config.rezipOutputs,
    flattenColor: config.flattenColor,
    overwriteMode: config.overwriteMode,
    optimizePng: config.optimizePng,
    slowDriveMode: config.slowDriveMode ?? false,
  };
}

export function queueItemToUi(item: QueueItem, existing?: UiQueueItem): UiQueueItem {
  return {
    ...item,
    status: existing?.status ?? "pending",
    bytesAfter: existing?.bytesAfter,
    message: existing?.message,
    selected: existing?.selected ?? false,
    outputBaseName: item.outputBaseName ?? existing?.outputBaseName,
  };
}

export function queueHasZipSources(items: UiQueueItem[]): boolean {
  return items.some((item) => item.zipSourcePath);
}
