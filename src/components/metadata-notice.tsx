import { InfoIcon } from "lucide-react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";

export function MetadataNotice() {
  return (
    <Collapsible defaultOpen={false}>
      <CollapsibleTrigger className="flex w-full items-start gap-2 rounded-lg border border-border/60 bg-muted/40 px-3 py-2 text-left text-xs hover:bg-muted/60">
        <InfoIcon className="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
        <span className="min-w-0 flex-1">
          <span className="font-medium text-foreground">Metadata on convert</span>
          <span className="text-muted-foreground">
            {" "}
            — GPS, camera info, and dates are removed from converted files only.
          </span>
        </span>
      </CollapsibleTrigger>
      <CollapsibleContent className="space-y-2 px-1 pt-2 text-xs text-muted-foreground">
        <p>
          <strong className="text-foreground">Previews</strong> show your original
          files unchanged. Nothing is stripped until you click Convert.
        </p>
        <p>
          <strong className="text-foreground">Converted outputs</strong> have EXIF/XMP
          removed (including GPS and camera tags). Rotation from the original is baked
          into the pixels.
        </p>
        <p>
          <strong className="text-foreground">Originals on disk</strong> are never
          modified. If you enable Skip same format, matching files are left untouched
          and keep all metadata.
        </p>
        <p>
          Optional <strong className="text-foreground">Keep color profile</strong> in
          More options embeds ICC on PNG when the source had a profile.
        </p>
      </CollapsibleContent>
    </Collapsible>
  );
}
