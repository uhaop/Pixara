import type { Preset, TargetImageFormat } from "@/lib/types";

/** Matches `EncodeSettings::from_preset` in src-tauri/src/formats.rs */
export const PRESET_ENCODE: Record<
  Preset,
  { jpeg: number; webp: number; avif: number; heic: number }
> = {
  web: { jpeg: 85, webp: 82, avif: 55, heic: 50 },
  high: { jpeg: 95, webp: 92, avif: 75, heic: 75 },
  smallest: { jpeg: 72, webp: 65, avif: 40, heic: 40 },
};

/** PNG zlib effort from preset (encode step). */
export const PRESET_PNG: Record<Preset, string> = {
  web: "default compression",
  high: "best zlib compression",
  smallest: "best zlib + optional oxipng pass",
};

const FORMATS_USING_QUALITY: TargetImageFormat[] = [
  "jpeg",
  "webp",
  "avif",
  "heic",
];

export function presetUsesQuality(toFormat: TargetImageFormat): boolean {
  return FORMATS_USING_QUALITY.includes(toFormat);
}

export function presetTooltip(preset: Preset, toFormat: TargetImageFormat): string {
  const q = PRESET_ENCODE[preset];
  if (toFormat === "png") {
    const ox =
      preset === "smallest"
        ? " Enable Optimize PNG in More options for an extra lossless pass (slower)."
        : "";
    return `${labelForPreset(preset)} — PNG uses ${PRESET_PNG[preset]}.${ox}`;
  }
  if (!presetUsesQuality(toFormat)) {
    return `${labelForPreset(preset)} — quality presets apply to JPEG, WebP, AVIF, HEIC, and PNG. GIF, BMP, and TIFF use fixed encoder settings.`;
  }
  switch (toFormat) {
    case "jpeg":
      return `${labelForPreset(preset)} — JPEG quality ${q.jpeg}. Lower = smaller files, more loss.`;
    case "webp":
      return `${labelForPreset(preset)} — WebP quality ${q.webp}. Lower = smaller files.`;
    case "avif":
      return `${labelForPreset(preset)} — AVIF quality ${q.avif}. Lower = smaller files.`;
    case "heic":
      return `${labelForPreset(preset)} — HEIC quality ${q.heic}. Lower = smaller files.`;
    default:
      return labelForPreset(preset);
  }
}

function labelForPreset(preset: Preset): string {
  switch (preset) {
    case "web":
      return "Web — balanced size and quality for sharing online";
    case "high":
      return "High — larger files, minimal visible compression";
    case "smallest":
      return "Smallest — smallest files, more visible compression";
  }
}
