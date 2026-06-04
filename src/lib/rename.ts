import type { QueueItem, TargetImageFormat } from "@/lib/types";

export type RenameMode = "suffix" | "template";

const INVALID_WIN = /[<>:"/\\|?*\x00-\x1f]/;

function isInvalidWinChar(ch: string): boolean {
  return INVALID_WIN.test(ch);
}

export function sanitizeOutputStem(stem: string): string {
  let out = "";
  for (const ch of stem) {
    const code = ch.charCodeAt(0);
    if (code < 32 || isInvalidWinChar(ch)) {
      out += "_";
    } else {
      out += ch;
    }
  }
  const trimmed = out.replace(/^[._\s]+|[._\s]+$/g, "").trim();
  return trimmed.length > 0 ? trimmed : "image";
}

export function defaultStemFromItem(item: QueueItem): string {
  const rel = item.relativePath.replace(/\\/g, "/");
  const base = rel.split("/").pop() ?? rel;
  const dot = base.lastIndexOf(".");
  const stem = dot > 0 ? base.slice(0, dot) : base;
  return stem || "image";
}

/** Stem used for queue display and inline rename (never includes target extension). */
export function outputStemForItem(item: {
  relativePath: string;
  outputBaseName?: string | null;
}): string {
  if (item.outputBaseName) {
    return item.outputBaseName;
  }
  return defaultStemFromItem(item as QueueItem);
}

const KNOWN_EXTENSIONS = [
  "png",
  "jpg",
  "jpeg",
  "webp",
  "heic",
  "heif",
  "gif",
  "bmp",
  "tif",
  "tiff",
  "avif",
] as const;

/** Normalize user input from inline rename (strip pasted extensions). */
export function normalizeEditedStem(raw: string, targetExt: string): string {
  let stem = sanitizeOutputStem(raw.trim());
  const stripSuffix = (value: string, ext: string) => {
    const suffix = `.${ext}`;
    if (value.toLowerCase().endsWith(suffix)) {
      return value.slice(0, -suffix.length);
    }
    return value;
  };

  stem = stripSuffix(stem, targetExt);
  for (const ext of KNOWN_EXTENSIONS) {
    const next = stripSuffix(stem, ext);
    if (next !== stem) {
      stem = next;
    }
  }
  return sanitizeOutputStem(stem);
}

function padIndex(index: number, width: number): string {
  return String(index).padStart(width, "0");
}

function parseIndexToken(raw: string, index: number): string {
  const match = /^index(?::(\d+))?$/i.exec(raw.trim());
  if (!match) {
    return String(index);
  }
  const width = match[1] ? Number.parseInt(match[1], 10) : 0;
  return width > 0 ? padIndex(index, width) : String(index);
}

export function applyTemplate(
  template: string,
  item: QueueItem,
  index: number,
  toFormat: TargetImageFormat,
): string {
  const rel = item.relativePath.replace(/\\/g, "/");
  const name = defaultStemFromItem(item);
  const parent = rel.includes("/") ? rel.slice(0, rel.lastIndexOf("/")) : "";
  const relativeNoExt = rel.includes(".")
    ? rel.slice(0, rel.lastIndexOf("."))
    : rel;

  const withTokens = template.replace(/\{([^}]+)\}/g, (_, token: string) => {
    const key = token.trim().toLowerCase();
    if (key === "name") {
      return name;
    }
    if (key === "ext") {
      return toFormat;
    }
    if (key === "parent") {
      return parent;
    }
    if (key === "relative") {
      return relativeNoExt;
    }
    if (key.startsWith("index")) {
      return parseIndexToken(key, index);
    }
    return "";
  });

  return sanitizeOutputStem(withTokens);
}

export function applySuffix(stem: string, suffix: string): string {
  const cleanSuffix = suffix.trim();
  if (!cleanSuffix) {
    return sanitizeOutputStem(stem);
  }
  return sanitizeOutputStem(`${stem}${cleanSuffix}`);
}

export function previewOutputBasenames(
  items: QueueItem[],
  mode: RenameMode,
  value: string,
  toFormat: TargetImageFormat,
): string[] {
  return items.slice(0, 3).map((item, i) => {
    const index = i + 1;
    const stem = defaultStemFromItem(item);
    if (mode === "suffix") {
      return applySuffix(stem, value);
    }
    return applyTemplate(value, item, index, toFormat);
  });
}

export function applyRenameToItems(
  items: QueueItem[],
  mode: RenameMode,
  value: string,
  toFormat: TargetImageFormat,
  startIndex = 1,
): QueueItem[] {
  return items.map((item, i) => {
    const index = startIndex + i;
    const stem = defaultStemFromItem(item);
    const outputBaseName =
      mode === "suffix"
        ? applySuffix(stem, value)
        : applyTemplate(value, item, index, toFormat);
    return { ...item, outputBaseName };
  });
}
