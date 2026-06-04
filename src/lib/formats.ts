import type { ImageFormat, TargetImageFormat } from "@/lib/types";

export type FormatOption<T extends string = string> = {
  value: T;
  label: string;
};

export const FROM_FORMAT_OPTIONS: FormatOption<ImageFormat>[] = [
  { value: "any", label: "Any format" },
  { value: "png", label: "PNG" },
  { value: "jpeg", label: "JPEG" },
  { value: "webp", label: "WebP" },
  { value: "heic", label: "HEIC" },
  { value: "gif", label: "GIF" },
  { value: "bmp", label: "BMP" },
  { value: "tiff", label: "TIFF" },
  { value: "avif", label: "AVIF" },
];

export const TO_FORMAT_OPTIONS: FormatOption<TargetImageFormat>[] =
  FROM_FORMAT_OPTIONS.filter(
    (option): option is FormatOption<TargetImageFormat> => option.value !== "any",
  );

export const FORMAT_OPTIONS = {
  from: FROM_FORMAT_OPTIONS,
  to: TO_FORMAT_OPTIONS,
} as const;

export function formatBytes(bytes: number | null | undefined): string {
  if (bytes == null || Number.isNaN(bytes)) {
    return "—";
  }
  if (bytes === 0) {
    return "0 B";
  }
  const units = ["B", "KB", "MB", "GB", "TB"] as const;
  const exponent = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  );
  const value = bytes / 1024 ** exponent;
  const digits = value >= 100 || exponent === 0 ? 0 : value >= 10 ? 1 : 2;
  return `${value.toFixed(digits)} ${units[exponent]}`;
}

export function basename(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const parts = normalized.split("/");
  return parts[parts.length - 1] || path;
}

const FORMAT_LABELS: Record<string, string> = {
  any: "Any",
  png: "PNG",
  jpeg: "JPEG",
  webp: "WebP",
  heic: "HEIC",
  gif: "GIF",
  bmp: "BMP",
  tiff: "TIFF",
  avif: "AVIF",
};

export function formatLabel(format: string): string {
  return FORMAT_LABELS[format] ?? format.toUpperCase();
}

const TARGET_EXTENSIONS: Record<TargetImageFormat, string> = {
  png: "png",
  jpeg: "jpg",
  webp: "webp",
  heic: "heic",
  gif: "gif",
  bmp: "bmp",
  tiff: "tiff",
  avif: "avif",
};

export function extensionForFormat(format: TargetImageFormat): string {
  return TARGET_EXTENSIONS[format];
}
