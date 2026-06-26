import { Injectable, computed, signal } from "@angular/core";
import { AppSettings, BackendDiskDriveUsage, BackendGpuAdapterUsage, BackendMemoryInfo, BackendNetworkAdapterUsage, MetricCard, ProcessRow, ResourceBar, ResourceSample, SystemInfoItem, UpdateFrequency, ViewId } from "../app.models";

@Injectable({ providedIn: "root" })
export class WorkareaStateService {
    private readonly appSettingsKey = "matrix-process-manager.app-settings";
    private readonly defaultAppSettings: AppSettings = {
        startWithWindows: false,
        minimizeToTray: false,
        confirmBeforeKillingProcesses: true,
        toolSettings: {
            taskManager: true,
            systemSettings: true,
            diskManager: true,
            terminal: true,
        },
    };

    activeView = signal<ViewId>("dashboard");
    totalProcesses = signal(0);
    metrics = signal<MetricCard[]>([]);
    rows = signal<ProcessRow[]>([]);
    selectedProcess = signal("");
    selectedPid = signal<number | undefined>(undefined);
    bars = signal<ResourceBar[]>([]);
    activeTitle = signal("Dashboard");
    updateFrequency = signal<UpdateFrequency>("high");
    resourceHistory = signal<ResourceSample[]>([]);
    systemInfo = signal<SystemInfoItem[]>([]);
    memoryInfo = signal<BackendMemoryInfo | undefined>(undefined);
    gpuAdapters = signal<BackendGpuAdapterUsage[]>([]);
    gpuAdapterHistory = signal<BackendGpuAdapterUsage[][]>([]);
    diskDrives = signal<BackendDiskDriveUsage[]>([]);
    diskDriveHistory = signal<BackendDiskDriveUsage[][]>([]);
    networkAdapters = signal<BackendNetworkAdapterUsage[]>([]);
    networkAdapterHistory = signal<BackendNetworkAdapterUsage[][]>([]);
    appSettings = signal<AppSettings>(this.loadAppSettings());

    selectedRow = computed(() => {
        const rows = this.rows();
        const selectedPid = this.selectedPid();
        return rows.find((row) => row.pid === selectedPid)
            ?? rows.find((row) => row.name === this.selectedProcess())
            ?? rows.find((row) => row.selected)
            ?? rows[0];
    });

    setState(state: {
        activeView: ViewId;
        totalProcesses: number;
        metrics: MetricCard[];
        rows: ProcessRow[];
        selectedProcess: string;
        bars: ResourceBar[];
        activeTitle: string;
        systemInfo?: SystemInfoItem[];
        memoryInfo?: BackendMemoryInfo;
        gpuAdapters?: BackendGpuAdapterUsage[];
        diskDrives?: BackendDiskDriveUsage[];
        networkAdapters?: BackendNetworkAdapterUsage[];
    }): void {
        this.activeView.set(state.activeView);
        this.totalProcesses.set(state.totalProcesses);
        this.metrics.set(state.metrics);
        this.rows.set(state.rows);
        this.selectedProcess.set(state.selectedProcess);
        this.bars.set(state.bars);
        this.activeTitle.set(state.activeTitle);
        this.systemInfo.set(state.systemInfo ?? this.systemInfo());
        this.memoryInfo.set(state.memoryInfo ?? this.memoryInfo());
        this.gpuAdapters.set(state.gpuAdapters ?? this.gpuAdapters());
        this.diskDrives.set(state.diskDrives ?? this.diskDrives());
        this.networkAdapters.set(state.networkAdapters ?? this.networkAdapters());
        this.recordResourceSample(state.metrics);
    }

    setUpdateFrequency(frequency: UpdateFrequency): void {
        this.updateFrequency.set(frequency);
    }

    setAppSetting<Key extends keyof AppSettings>(key: Key, value: AppSettings[Key]): void {
        this.appSettings.update((settings) => {
            const next = { ...settings, [key]: value };
            localStorage.setItem(this.appSettingsKey, JSON.stringify(next));
            return next;
        });
    }

    setToolSetting(toolId: keyof AppSettings["toolSettings"], enabled: boolean): void {
        this.appSettings.update((settings) => {
            const next = { ...settings, toolSettings: { ...settings.toolSettings, [toolId]: enabled } };
            localStorage.setItem(this.appSettingsKey, JSON.stringify(next));
            return next;
        });
    }

    resetAppSettings(): void {
        this.appSettings.set(this.defaultAppSettings);
        localStorage.setItem(this.appSettingsKey, JSON.stringify(this.defaultAppSettings));
    }

    selectProcess(row: ProcessRow): void {
        this.selectedProcess.set(row.name);
        this.selectedPid.set(row.pid);
    }

    setGpuAdapters(adapters: BackendGpuAdapterUsage[]): void {
        this.gpuAdapters.set(adapters);
        this.gpuAdapterHistory.update((history) => [...history.slice(-59), adapters]);
    }

    setMemoryInfo(info: BackendMemoryInfo): void {
        this.memoryInfo.set(info);
    }

    setDiskDrives(drives: BackendDiskDriveUsage[]): void {
        this.diskDrives.set(drives);
        this.diskDriveHistory.update((history) => [...history.slice(-59), drives]);
    }

    setNetworkAdapters(adapters: BackendNetworkAdapterUsage[]): void {
        this.networkAdapters.set(adapters);
        this.networkAdapterHistory.update((history) => [...history.slice(-59), adapters]);
    }

    private recordResourceSample(metrics: MetricCard[]): void {
        const sample: ResourceSample = {
            cpu: this.metricValue(metrics, "CPU"),
            gpu: this.metricValue(metrics, "GPU"),
            memory: this.metricValue(metrics, "Memory"),
            disk: this.metricValue(metrics, "Disk"),
            network: this.metricValue(metrics, "Network"),
        };

        this.resourceHistory.update((history) => [...history.slice(-59), sample]);
    }

    private metricValue(metrics: MetricCard[], label: string): number {
        return Number.parseFloat(metrics.find((metric) => metric.label === label)?.value ?? "0") || 0;
    }

    private loadAppSettings(): AppSettings {
        try {
            const saved = JSON.parse(localStorage.getItem(this.appSettingsKey) ?? "{}");
            return {
                ...this.defaultAppSettings,
                ...saved,
                toolSettings: { ...this.defaultAppSettings.toolSettings, ...saved.toolSettings },
            };
        } catch {
            return this.defaultAppSettings;
        }
    }
}