import { FolderOpenIcon, TriangleAlertIcon } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldTitle,
} from "@/components/ui/field";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { selectFormatValues } from "@/lib/config-capabilities";
import { formatOptionsForCapabilities } from "@/lib/formats";
import { presetTooltip, presetUsesQuality } from "@/lib/preset-info";
import type { AppConfig, OutputMode, Preset, SystemCapabilities } from "@/lib/types";
import { shouldWarnHeicToPng } from "@/lib/estimate-warning";
import type { UiQueueItem } from "@/lib/types";
import { cn } from "@/lib/utils";

type ControlPanelProps = {
  config: AppConfig;
  queue: UiQueueItem[];
  systemCaps: SystemCapabilities | null;
  onConfigChange: (patch: Partial<AppConfig>) => void;
  onBrowseOutputDirectory: () => void | Promise<void>;
};

const inputClassName =
  "flex h-8 w-full rounded-lg border border-input bg-transparent px-2.5 text-sm shadow-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-input/30";

export function ControlPanel({
  config,
  queue,
  systemCaps,
  onConfigChange,
  onBrowseOutputDirectory,
}: ControlPanelProps) {
  const formatOptions = formatOptionsForCapabilities(systemCaps);
  const { fromFormat, toFormat } = selectFormatValues(config, systemCaps);
  const showJpegWarning = config.toFormat === "jpeg";
  const showSameFormatWarning =
    config.fromFormat !== "any" && config.fromFormat === config.toFormat;
  const showHeicPngWarning = shouldWarnHeicToPng(queue, config.toFormat);

  return (
    <div className="flex min-h-0 flex-col gap-3 p-4">
      <div>
        <h2 className="font-heading text-sm font-medium">Conversion</h2>
      </div>

      <FieldGroup className="gap-3">
        <Field>
          <FieldLabel htmlFor="from-format">From</FieldLabel>
          <Select
            disabled={!systemCaps}
            value={fromFormat}
            onValueChange={(value) =>
              onConfigChange({ fromFormat: value as AppConfig["fromFormat"] })
            }
          >
            <SelectTrigger id="from-format" className="w-full">
              <SelectValue placeholder="Source format" />
            </SelectTrigger>
            <SelectContent>
              {formatOptions.from.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>

        <Field>
          <FieldLabel htmlFor="to-format">To</FieldLabel>
          <Select
            disabled={!systemCaps}
            value={toFormat}
            onValueChange={(value) =>
              onConfigChange({ toFormat: value as AppConfig["toFormat"] })
            }
          >
            <SelectTrigger id="to-format" className="w-full">
              <SelectValue placeholder="Target format" />
            </SelectTrigger>
            <SelectContent>
              {formatOptions.to.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>

        {showSameFormatWarning && (
          <Alert>
            <TriangleAlertIcon />
            <AlertTitle>Same source and target format</AlertTitle>
            <AlertDescription>
              Re-encoding only helps if you resize or change quality. Consider another
              target format to avoid generation loss.
            </AlertDescription>
          </Alert>
        )}

        {showHeicPngWarning && (
          <Alert>
            <TriangleAlertIcon />
            <AlertTitle>HEIC → PNG size</AlertTitle>
            <AlertDescription>
              PNG output is often much larger than HEIC originals (uncompressed pixels).
              Use Estimate for a sample-based size before converting a large batch.
            </AlertDescription>
          </Alert>
        )}

        <Field>
          <FieldTitle>Preset</FieldTitle>
          {!presetUsesQuality(config.toFormat) && (
            <FieldDescription>
              For {config.toFormat.toUpperCase()}, Web / High / Smallest do not change
              compression yet — only JPEG, WebP, AVIF, and HEIC use these quality levels.
            </FieldDescription>
          )}
          <FieldContent>
            <TooltipProvider delay={300}>
              <ToggleGroup
                variant="outline"
                spacing={0}
                className="w-full"
                value={[config.preset]}
                onValueChange={(values) => {
                  const next = values[values.length - 1] as Preset | undefined;
                  if (next) {
                    onConfigChange({ preset: next });
                  }
                }}
              >
                {(["web", "high", "smallest"] as const).map((preset) => (
                  <Tooltip key={preset}>
                    <TooltipTrigger
                      render={
                        <ToggleGroupItem className="flex-1" value={preset} />
                      }
                    >
                      {preset === "web" ? "Web" : preset === "high" ? "High" : "Smallest"}
                    </TooltipTrigger>
                    <TooltipContent side="bottom" className="max-w-[16rem] text-left">
                      {presetTooltip(preset, config.toFormat)}
                    </TooltipContent>
                  </Tooltip>
                ))}
              </ToggleGroup>
            </TooltipProvider>
          </FieldContent>
        </Field>

        <Field>
          <FieldTitle>Output</FieldTitle>
          <FieldContent>
            <ToggleGroup
              variant="outline"
              spacing={0}
              className="w-full"
              value={[config.outputMode]}
              onValueChange={(values) => {
                const next = values[values.length - 1] as OutputMode | undefined;
                if (next) {
                  onConfigChange({ outputMode: next });
                }
              }}
            >
              <ToggleGroupItem className="flex-1" value="sameFolder">
                Same folder
              </ToggleGroupItem>
              <ToggleGroupItem className="flex-1" value="customDir">
                Custom
              </ToggleGroupItem>
            </ToggleGroup>
          </FieldContent>
        </Field>

        {config.outputMode === "customDir" && (
          <Field>
            <FieldLabel htmlFor="custom-output-dir">Output directory</FieldLabel>
            <div className="flex gap-2">
              <input
                id="custom-output-dir"
                readOnly
                value={config.customOutputDir ?? ""}
                placeholder="Choose a folder…"
                className={cn(inputClassName, "min-w-0 flex-1")}
              />
              <Button
                type="button"
                variant="outline"
                size="icon-sm"
                aria-label="Browse output folder"
                onClick={() => void onBrowseOutputDirectory()}
              >
                <FolderOpenIcon />
              </Button>
            </div>
          </Field>
        )}

        {showJpegWarning && (
          <Alert variant="destructive">
            <TriangleAlertIcon />
            <AlertTitle>JPEG is lossy</AlertTitle>
            <AlertDescription>
              Converting to JPEG permanently discards detail. Prefer WebP or AVIF when
              transparency is not required.
            </AlertDescription>
          </Alert>
        )}
      </FieldGroup>
    </div>
  );
}
