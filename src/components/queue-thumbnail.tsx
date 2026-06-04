import { memo, useEffect, useRef, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { ImageIcon } from "lucide-react";
import { supportsNativePreview } from "@/lib/preview-source";
import { runThumbnailTask } from "@/lib/thumbnail-queue";
import {
  getCachedThumbnailUrl,
  setCachedThumbnailUrl,
  thumbnailCacheKey,
} from "@/lib/thumbnail-cache";
import { cn } from "@/lib/utils";

function isConversionBusyError(error: unknown): boolean {
  return (
    typeof error === "string" && error.includes("conversion_in_progress")
  );
}

type QueueThumbnailProps = {
  itemId: string;
  sourcePath: string;
  className?: string;
  /** When false, skips new loads but keeps an already-resolved preview visible. */
  enabled?: boolean;
};

function QueueThumbnailInner({
  itemId,
  sourcePath,
  className,
  enabled = true,
}: QueueThumbnailProps) {
  const cacheKey = thumbnailCacheKey(itemId, sourcePath);
  const rootRef = useRef<HTMLDivElement>(null);
  const [inView, setInView] = useState(false);
  const [src, setSrc] = useState<string | null>(
    () => getCachedThumbnailUrl(cacheKey) ?? null,
  );
  const [failed, setFailed] = useState(false);
  const nativePreview = supportsNativePreview(sourcePath);

  useEffect(() => {
    if (!enabled) {
      return;
    }
    const el = rootRef.current;
    if (!el) {
      return;
    }
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry?.isIntersecting) {
          setInView(true);
        }
      },
      { rootMargin: "160px" },
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, [enabled, itemId]);

  useEffect(() => {
    if (!enabled || !inView) {
      return;
    }

    const cached = getCachedThumbnailUrl(cacheKey);
    if (cached) {
      setSrc(cached);
      setFailed(false);
      return;
    }

    let cancelled = false;

    if (nativePreview) {
      const url = convertFileSrc(sourcePath);
      setCachedThumbnailUrl(cacheKey, url);
      setSrc(url);
      setFailed(false);
      return;
    }

    void runThumbnailTask(async () =>
      invoke<string>("get_thumbnail_cmd", {
        itemId,
        sourcePath,
      }),
    )
      .then((path) => {
        if (cancelled) {
          return;
        }
        const url = convertFileSrc(path);
        setCachedThumbnailUrl(cacheKey, url);
        setSrc(url);
        setFailed(false);
      })
      .catch((error) => {
        if (!cancelled && !isConversionBusyError(error)) {
          setFailed(true);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [cacheKey, enabled, inView, itemId, nativePreview, sourcePath]);

  async function handleNativeError() {
    if (!nativePreview || failed) {
      return;
    }
    setFailed(true);
    try {
      const path = await runThumbnailTask(() =>
        invoke<string>("get_thumbnail_cmd", {
          itemId,
          sourcePath,
        }),
      );
      const url = convertFileSrc(path);
      setCachedThumbnailUrl(cacheKey, url);
      setSrc(url);
      setFailed(false);
    } catch (error) {
      if (!isConversionBusyError(error)) {
        setFailed(true);
      }
    }
  }

  const showImage = Boolean(src && !failed);

  return (
    <div
      ref={rootRef}
      className={cn(
        "flex items-center justify-center overflow-hidden rounded-md bg-muted text-muted-foreground",
        className,
      )}
    >
      {showImage ? (
        <img
          src={src!}
          alt=""
          className="size-full object-cover"
          decoding="async"
          onError={() => {
            void handleNativeError();
          }}
        />
      ) : (
        <ImageIcon className="size-4" />
      )}
    </div>
  );
}

export const QueueThumbnail = memo(QueueThumbnailInner);
