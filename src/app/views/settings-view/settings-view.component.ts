import { Component, inject, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { AppSettings, NativeToolId, UpdateFrequency } from "../../app.models";
import { WorkareaStateService } from "../../services/workarea-state.service";

type GeneralSettingKey = keyof Pick<AppSettings, "startWithWindows" | "minimizeToTray" | "confirmBeforeKillingProcesses">;

@Component({
    selector: "mtx-settings-view",
    templateUrl: "./settings-view.component.html",
})
export class SettingsViewComponent {
    state = inject(WorkareaStateService);
    activeCategory = signal("General");
    categories = [
        { label: "General", icon: "bi-gear" },
        { label: "Tools", icon: "bi-puzzle" },
        { label: "About", icon: "bi-info-circle" },
    ];
    frequencies: { value: UpdateFrequency; label: string; detail: string }[] = [
        { value: "high", label: "High", detail: "Refresh every second" },
        { value: "normal", label: "Normal", detail: "Refresh every 2 seconds" },
        { value: "low", label: "Low", detail: "Refresh every 5 seconds" },
        { value: "paused", label: "Paused", detail: "Stop automatic updates" },
    ];
    generalToggles: Array<{ key: GeneralSettingKey; label: string; detail: string }> = [
        { key: "startWithWindows", label: "Start with Windows", detail: "Launch Process Manager automatically when Windows starts." },
        { key: "minimizeToTray", label: "Minimize to system tray", detail: "Minimize the application to system tray instead of the taskbar." },
        { key: "confirmBeforeKillingProcesses", label: "Confirm before killing processes", detail: "Show confirmation dialog before terminating a process." },
    ];
    generalSelects = [
        { label: "Language", detail: "Select the application language.", value: "System Default", options: ["System Default", "English"] },
        { label: "Date & time format", detail: "Choose how dates and times are displayed.", value: "System Default", options: ["System Default", "12-hour", "24-hour"] },
    ];
    toolSettings: Array<{ key: NativeToolId; label: string; detail: string }> = [
        { key: "taskManager", label: "Task Manager", detail: "Open Windows Task Manager from the Tools menu." },
        { key: "systemSettings", label: "System Settings", detail: "Open Windows Settings to the About page." },
        { key: "diskManager", label: "Disk Manager", detail: "Open Windows Disk Management from the Tools menu." },
        { key: "terminal", label: "Terminal", detail: "Open Windows Terminal from the Tools menu." },
    ];
    aboutItems = [
        { label: "Product", value: "Process Manager" },
        { label: "Version", value: "1.0.0" },
        { label: "Website", value: "https://matrixrepublic.net/", link: "https://matrixrepublic.net/" },
    ];

    activeDescription(): string {
        switch (this.activeCategory()) {
            case "Tools":
                return "Configure external Windows tools and launch behavior.";
            case "About":
                return "Application version and product information.";
            default:
                return "Basic application settings and behavior.";
        }
    }

    setCategory(category: string): void {
        this.activeCategory.set(category);
    }

    settingEnabled(key: GeneralSettingKey): boolean {
        return this.state.appSettings()[key];
    }

    toggleGeneralSetting(key: GeneralSettingKey): void {
        const next = !this.settingEnabled(key);
        this.state.setAppSetting(key, next);
        if (key === "startWithWindows") {
            invoke<void>("set_start_with_windows", { enabled: next }).catch(() => this.state.setAppSetting(key, !next));
        }
    }

    toolEnabled(key: NativeToolId): boolean {
        return this.state.appSettings().toolSettings[key];
    }

    toggleToolSetting(key: NativeToolId): void {
        this.state.setToolSetting(key, !this.toolEnabled(key));
    }

    openWebsite(url: string | undefined): void {
        if (!url) {
            return;
        }

        openUrl(url).catch(() => undefined);
    }
}