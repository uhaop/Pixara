import type { AppConfig, SystemCapabilities } from "@/lib/types";

type ConvertQueueItem = {
  sourceFormat: AppConfig["fromFormat"] | string;
};

export function validateBeforeConvert(
  config: AppConfig,
  caps?: SystemCapabilities | null,
  queue?: ConvertQueueItem[],
): string | null {
  if (config.outputMode === "customDir" && !config.customOutputDir?.trim()) {
    return "Choose an output folder before converting";
  }
  if (config.maxWidth === 0 || config.maxHeight === 0) {
    return "Max width and height must be greater than zero";
  }
  const heicReadAvailable = caps?.heicReadAvailable ?? false;
  const heicWriteAvailable = caps?.heicWriteAvailable ?? false;

  if (!heicReadAvailable) {
    if (config.fromFormat === "heic") {
      return "HEIC input is not available in this build";
    }
    if (queue?.some((item) => item.sourceFormat === "heic")) {
      return "Queue contains HEIC files but this build cannot read HEIC";
    }
  }
  if (!heicWriteAvailable && config.toFormat === "heic") {
    return "HEIC export is not available in this build";
  }
  return null;
}
