import { LayoutGridIcon, ListIcon, TriangleAlertIcon } from "lucide-react";
import { MetadataNotice } from "@/components/metadata-notice";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldTitle,
} from "@/components/ui/field";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { parsePositiveDimension } from "@/lib/dimensions";
import type {
  AppConfig,
  NamingMode,
  OverwriteMode,
  QueueView,
} from "@/lib/types";
import { queueHasZipSources, type UiQueueItem } from "@/lib/types";

const inputClassName =
  "flex h-8 w-full rounded-lg border border-input bg-transparent px-2.5 text-sm shadow-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-input/30";

type SettingsPanelProps = {
  config: AppConfig;
  queue: UiQueueItem[];
  onConfigChange: (patch: Partial<AppConfig>) => void;
};

export function SettingsPanel({ config, queue, onConfigChange }: SettingsPanelProps) {
  const hasZipInQueue = queueHasZipSources(queue);

  return (
    <div className="flex min-h-0 flex-col gap-3 p-4">
      <p className="text-xs text-muted-foreground">
        Saved automatically to your profile and restored when you reopen the app.
      </p>

      <FieldGroup className="gap-4">
        <section className="flex flex-col gap-3">
          <h2 className="font-heading text-sm font-medium">Display</h2>
          <Field>
            <FieldTitle>Queue layout</FieldTitle>
            <FieldDescription>
              Default list or grid view for the file queue. You can still switch from the
              queue toolbar.
            </FieldDescription>
            <FieldContent>
              <ToggleGroup
                variant="outline"
                spacing={0}
                className="w-full"
                value={[config.queueView]}
                onValueChange={(values) => {
                  const next = values[values.length - 1] as QueueView | undefined;
                  if (next) {
                    onConfigChange({ queueView: next });
                  }
                }}
              >
                <ToggleGroupItem className="flex-1 gap-1.5" value="list">
                  <ListIcon className="size-4" />
                  List
                </ToggleGroupItem>
                <ToggleGroupItem className="flex-1 gap-1.5" value="grid">
                  <LayoutGridIcon className="size-4" />
                  Grid
                </ToggleGroupItem>
              </ToggleGroup>
            </FieldContent>
          </Field>
        </section>

        <section className="flex flex-col gap-3">
          <h2 className="font-heading text-sm font-medium">Output defaults</h2>

          <Field>
            <FieldTitle>Naming</FieldTitle>
            <FieldContent>
              <ToggleGroup
                variant="outline"
                spacing={0}
                className="w-full"
                value={[config.naming]}
                onValueChange={(values) => {
                  const next = values[values.length - 1] as NamingMode | undefined;
                  if (next) {
                    onConfigChange({ naming: next });
                  }
                }}
              >
                <ToggleGroupItem className="flex-1" value="replaceExtension">
                  Replace extension
                </ToggleGroupItem>
                <ToggleGroupItem className="flex-1" value="appendSuffix">
                  Append suffix
                </ToggleGroupItem>
              </ToggleGroup>
            </FieldContent>
          </Field>

          <Field orientation="horizontal">
            <FieldContent>
              <Label htmlFor="settings-preserve-structure">Preserve folder structure</Label>
              <FieldDescription>
                For folder and ZIP drops, mirror subfolders under the output directory.
              </FieldDescription>
            </FieldContent>
            <Switch
              id="settings-preserve-structure"
              checked={config.preserveStructure}
              onCheckedChange={(checked) =>
                onConfigChange({ preserveStructure: Boolean(checked) })
              }
            />
          </Field>

          <Field>
            <FieldLabel htmlFor="settings-overwrite-mode">If output exists</FieldLabel>
            <select
              id="settings-overwrite-mode"
              className={inputClassName}
              value={config.overwriteMode}
              onChange={(event) =>
                onConfigChange({
                  overwriteMode: event.target.value as OverwriteMode,
                })
              }
            >
              <option value="autoRename">Auto-rename</option>
              <option value="replace">Replace</option>
              <option value="skip">Skip</option>
            </select>
          </Field>
        </section>

        <section className="flex flex-col gap-3">
          <h2 className="font-heading text-sm font-medium">Conversion defaults</h2>

          <div className="grid gap-3">
            <Field>
              <FieldLabel htmlFor="settings-max-width">Max width (px)</FieldLabel>
              <input
                id="settings-max-width"
                type="number"
                min={0}
                className={inputClassName}
                value={config.maxWidth ?? ""}
                onChange={(event) => {
                  const raw = event.target.value;
                  const parsed = parsePositiveDimension(raw);
                  if (raw !== "" && parsed == null) {
                    return;
                  }
                  onConfigChange({ maxWidth: parsed });
                }}
              />
              <FieldDescription>Leave empty for no limit.</FieldDescription>
            </Field>
            <Field>
              <FieldLabel htmlFor="settings-max-height">Max height (px)</FieldLabel>
              <input
                id="settings-max-height"
                type="number"
                min={0}
                className={inputClassName}
                value={config.maxHeight ?? ""}
                onChange={(event) => {
                  const raw = event.target.value;
                  const parsed = parsePositiveDimension(raw);
                  if (raw !== "" && parsed == null) {
                    return;
                  }
                  onConfigChange({ maxHeight: parsed });
                }}
              />
            </Field>
          </div>

          <Field>
            <FieldLabel htmlFor="settings-flatten-color">Flatten transparency color</FieldLabel>
            <input
              id="settings-flatten-color"
              type="text"
              className={inputClassName}
              value={config.flattenColor}
              placeholder="#ffffff"
              onChange={(event) =>
                onConfigChange({ flattenColor: event.target.value })
              }
            />
            <FieldDescription>
              Used when exporting to JPEG or HEIC (hex, e.g. #ffffff).
            </FieldDescription>
          </Field>

          <Field orientation="horizontal">
            <FieldContent>
              <Label htmlFor="settings-keep-icc">Keep color profile (ICC)</Label>
              <FieldDescription>
                Embeds ICC on PNG when the source had a profile.
              </FieldDescription>
            </FieldContent>
            <Switch
              id="settings-keep-icc"
              checked={!config.stripIcc}
              onCheckedChange={(checked) =>
                onConfigChange({ stripIcc: !Boolean(checked) })
              }
            />
          </Field>

          <Field orientation="horizontal">
            <FieldContent>
              <Label htmlFor="settings-optimize-png">Optimize PNG files</Label>
              <FieldDescription>
                After encode, run oxipng for smaller PNGs (slower).
              </FieldDescription>
            </FieldContent>
            <Switch
              id="settings-optimize-png"
              checked={config.optimizePng}
              onCheckedChange={(checked) =>
                onConfigChange({ optimizePng: Boolean(checked) })
              }
            />
          </Field>

          <Field orientation="horizontal">
            <FieldContent>
              <Label htmlFor="settings-slow-drive">Slow drive mode</Label>
              <FieldDescription>
                Caps parallel workers to 2 for USB or network folders.
              </FieldDescription>
            </FieldContent>
            <Switch
              id="settings-slow-drive"
              checked={config.slowDriveMode}
              onCheckedChange={(checked) =>
                onConfigChange({ slowDriveMode: Boolean(checked) })
              }
            />
          </Field>

          <Field orientation="horizontal">
            <FieldContent>
              <Label htmlFor="settings-skip-same-format">Skip same format</Label>
              <FieldDescription>
                When source and target format match, leave the file on disk unchanged.
              </FieldDescription>
            </FieldContent>
            <Switch
              id="settings-skip-same-format"
              checked={config.skipSameFormat}
              onCheckedChange={(checked) =>
                onConfigChange({ skipSameFormat: Boolean(checked) })
              }
            />
          </Field>

          {config.skipSameFormat && (
            <Alert>
              <TriangleAlertIcon />
              <AlertTitle>Skipped files keep original metadata</AlertTitle>
              <AlertDescription>
                Turn this off to re-encode and remove GPS and other tags from matching
                files.
              </AlertDescription>
            </Alert>
          )}

          <Field orientation="horizontal">
            <FieldContent>
              <Label htmlFor="settings-rezip-outputs">Re-zip converted outputs</Label>
              <FieldDescription>
                {hasZipInQueue
                  ? "Creates {name}_converted.zip beside each source ZIP after conversion."
                  : "When the queue includes ZIP drops, write a converted archive beside each source."}
              </FieldDescription>
            </FieldContent>
            <Switch
              id="settings-rezip-outputs"
              checked={config.rezipOutputs}
              disabled={!hasZipInQueue}
              onCheckedChange={(checked) =>
                onConfigChange({ rezipOutputs: Boolean(checked) })
              }
            />
          </Field>
        </section>

        <section className="flex flex-col gap-2">
          <h2 className="font-heading text-sm font-medium">Privacy & metadata</h2>
          <MetadataNotice />
        </section>
      </FieldGroup>
    </div>
  );
}
