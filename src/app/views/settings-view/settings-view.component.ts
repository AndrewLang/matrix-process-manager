import { Component, inject, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { AppSettings, IndexingSchedule, NativeToolId, TerminalCursorStyle, TerminalDefaultShell, TerminalTheme, UpdateFrequency } from "../../app.models";
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
        { label: "Terminal", icon: "bi-terminal" },
        { label: "Indexing", icon: "bi-arrow-repeat" },
        { label: "Storage", icon: "bi-database" },
        { label: "Tools", icon: "bi-puzzle" },
        { label: "About", icon: "bi-info-circle" },
    ];
    shellOptions: Array<{ value: TerminalDefaultShell; label: string }> = [
        { value: "system", label: "System default" },
        { value: "powerShell", label: "PowerShell" },
        { value: "cmd", label: "CMD" },
        { value: "zsh", label: "zsh" },
        { value: "bash", label: "bash" },
    ];
    cursorStyles: Array<{ value: TerminalCursorStyle; label: string }> = [
        { value: "block", label: "Block" },
        { value: "bar", label: "Bar" },
        { value: "underline", label: "Underline" },
    ];
    terminalThemes: Array<{ value: TerminalTheme; label: string }> = [
        { value: "matrix", label: "Matrix" },
        { value: "midnight", label: "Midnight" },
        { value: "slate", label: "Slate" },
    ];
    indexingSchedules: Array<{ value: IndexingSchedule; label: string; detail: string }> = [
        { value: "manual", label: "Manual", detail: "Only index when requested." },
        { value: "startup", label: "Startup", detail: "Index once when the app starts." },
        { value: "hourly", label: "Hourly", detail: "Refresh command knowledge every hour." },
        { value: "daily", label: "Daily", detail: "Refresh command knowledge once per day." },
    ];
    frequencies: { value: UpdateFrequency; label: string; detail: string }[] = [
        { value: "high", label: "High", detail: "Refresh every second" },
        { value: "normal", label: "Normal", detail: "Refresh every 2 seconds" },
        { value: "low", label: "Low", detail: "Refresh every 5 seconds" },
        { value: "paused", label: "Paused", detail: "Stop automatic updates" },
    ];
    generalToggles: Array<{ key: GeneralSettingKey; label: string; detail: string }> = [
        { key: "startWithWindows", label: "Start with Windows", detail: "Launch Workstation Console automatically when Windows starts." },
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
        { key: "envVariables", label: "Env Variables", detail: "Open Windows Environment Variables from the Tools menu." },
    ];
    aboutItems = [
        { label: "Product", value: "Workstation Console" },
        { label: "Version", value: "1.0.0" },
        { label: "Website", value: "https://matrixrepublic.net/", link: "https://matrixrepublic.net/" },
    ];

    activeDescription(): string {
        switch (this.activeCategory()) {
            case "Terminal":
                return "Configure Command Center terminal behavior and autocomplete.";
            case "Indexing":
                return "Configure command knowledge indexing cadence.";
            case "Storage":
                return "Configure where command knowledge data is stored.";
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

    setDefaultShell(value: string): void {
        this.state.setTerminalSetting("defaultShell", value as TerminalDefaultShell);
    }

    setTerminalFont(value: string): void {
        this.state.setTerminalSetting("fontFamily", value);
    }

    setTerminalNumber(key: "fontSize" | "opacity" | "historySize" | "autocompleteDelayMs", value: string): void {
        this.state.setTerminalSetting(key, Number(value));
    }

    setCursorStyle(value: string): void {
        this.state.setTerminalSetting("cursorStyle", value as TerminalCursorStyle);
    }

    setTerminalTheme(value: string): void {
        this.state.setTerminalSetting("theme", value as TerminalTheme);
    }

    setIndexingSchedule(value: string): void {
        this.state.setIndexingSetting("schedule", value as IndexingSchedule);
    }

    setSqliteLocation(value: string): void {
        this.state.setStorageSetting("sqliteLocation", value.trim() || "default");
    }

    openWebsite(url: string | undefined): void {
        if (!url) {
            return;
        }

        openUrl(url).catch(() => undefined);
    }
}