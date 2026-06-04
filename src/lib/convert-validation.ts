import type { AppConfig } from "@/lib/types";

export function validateBeforeConvert(config: AppConfig): string | null {
  if (config.outputMode === "customDir" && !config.customOutputDir?.trim()) {
    return "Choose an output folder before converting";
  }
  if (config.maxWidth === 0 || config.maxHeight === 0) {
    return "Max width and height must be greater than zero";
  }
  return null;
}
