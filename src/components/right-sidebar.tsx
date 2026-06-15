import { ControlPanel } from "@/components/control-panel";
import { SettingsPanel } from "@/components/settings-panel";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { AppConfig, SystemCapabilities, UiQueueItem } from "@/lib/types";
import { cn } from "@/lib/utils";

export type SidebarTab = "conversion" | "settings";

type RightSidebarProps = {
  activeTab: SidebarTab;
  onActiveTabChange: (tab: SidebarTab) => void;
  config: AppConfig;
  queue: UiQueueItem[];
  systemCaps: SystemCapabilities | null;
  onConfigChange: (patch: Partial<AppConfig>) => void;
  onBrowseOutputDirectory: () => void | Promise<void>;
};

export function RightSidebar({
  activeTab,
  onActiveTabChange,
  config,
  queue,
  systemCaps,
  onConfigChange,
  onBrowseOutputDirectory,
}: RightSidebarProps) {
  return (
    <div className="flex min-h-0 flex-col">
      <div className="shrink-0 border-b px-3 py-2">
        <ToggleGroup
          variant="outline"
          spacing={0}
          className="w-full"
          value={[activeTab]}
          onValueChange={(values) => {
            const next = values[values.length - 1] as SidebarTab | undefined;
            if (next) {
              onActiveTabChange(next);
            }
          }}
        >
          <ToggleGroupItem className="flex-1" value="conversion">
            Convert
          </ToggleGroupItem>
          <ToggleGroupItem className="flex-1" value="settings">
            Settings
          </ToggleGroupItem>
        </ToggleGroup>
      </div>
      <div
        className={cn(
          "min-h-0 flex-1 overflow-y-auto",
          activeTab !== "conversion" && "hidden",
        )}
        aria-hidden={activeTab !== "conversion"}
      >
        <ControlPanel
          config={config}
          queue={queue}
          systemCaps={systemCaps}
          onConfigChange={onConfigChange}
          onBrowseOutputDirectory={onBrowseOutputDirectory}
        />
      </div>
      <div
        className={cn(
          "min-h-0 flex-1 overflow-y-auto",
          activeTab !== "settings" && "hidden",
        )}
        aria-hidden={activeTab !== "settings"}
      >
        <SettingsPanel config={config} queue={queue} onConfigChange={onConfigChange} />
      </div>
    </div>
  );
}
