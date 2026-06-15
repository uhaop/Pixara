import type { AppConfig, SystemCapabilities } from "@/lib/types";

/** Assume HEIC is unavailable until capabilities are known. */
export const CONSERVATIVE_SYSTEM_CAPS: SystemCapabilities = {
  logicalCpus: 1,
  convertWorkers: 1,
  heicReadAvailable: false,
  heicWriteAvailable: false,
  gpuDetected: false,
  convertBackend: "cpuParallel",
  heicDecodeBackend: "unknown",
  ffmpegHwaccels: [],
  backendNote: "",
  decodeNote: "",
  encodeNote: "",
};

/** Reset HEIC source/target when the running build cannot read or write it. */
export function sanitizeConfigForCapabilities(
  config: AppConfig,
  caps: SystemCapabilities | null,
): AppConfig {
  if (!caps) {
    return config;
  }
  const next = { ...config };
  if (!caps.heicReadAvailable && next.fromFormat === "heic") {
    next.fromFormat = "any";
  }
  if (!caps.heicWriteAvailable && next.toFormat === "heic") {
    next.toFormat = "webp";
  }
  return next;
}

/** Select-safe format values while capabilities are still loading. */
export function selectFormatValues(
  config: AppConfig,
  caps: SystemCapabilities | null,
): Pick<AppConfig, "fromFormat" | "toFormat"> {
  if (caps) {
    return { fromFormat: config.fromFormat, toFormat: config.toFormat };
  }
  return {
    fromFormat: config.fromFormat === "heic" ? "any" : config.fromFormat,
    toFormat: config.toFormat === "heic" ? "webp" : config.toFormat,
  };
}

export function configNeedsHeicSanitize(
  before: AppConfig,
  after: AppConfig,
): boolean {
  return (
    before.fromFormat !== after.fromFormat || before.toFormat !== after.toFormat
  );
}
