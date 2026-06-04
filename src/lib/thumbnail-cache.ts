/** In-memory preview URLs so rows do not flash icon→image on re-render. */
const assetUrlByKey = new Map<string, string>();

export function thumbnailCacheKey(itemId: string, filePath: string): string {
  return `${itemId}::${filePath}`;
}

export function getCachedThumbnailUrl(key: string): string | undefined {
  return assetUrlByKey.get(key);
}

export function setCachedThumbnailUrl(key: string, assetUrl: string): void {
  assetUrlByKey.set(key, assetUrl);
}

export function clearThumbnailCache(): void {
  assetUrlByKey.clear();
}
