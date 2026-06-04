const NATIVE_PREVIEW_EXTENSIONS = new Set([
  "png",
  "jpg",
  "jpeg",
  "webp",
  "gif",
  "bmp",
]);

/** Formats the WebView can display directly via convertFileSrc (no Rust decode). */
export function supportsNativePreview(sourcePath: string): boolean {
  const normalized = sourcePath.replace(/\\/g, "/");
  const base = normalized.split("/").pop() ?? normalized;
  const dot = base.lastIndexOf(".");
  if (dot <= 0) {
    return false;
  }
  return NATIVE_PREVIEW_EXTENSIONS.has(base.slice(dot + 1).toLowerCase());
}
