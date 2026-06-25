import { Injectable, computed, signal } from "@angular/core";
import { BackendGpuAdapterUsage, MetricCard, ProcessRow, ResourceBar, ResourceSample, SystemInfoItem, UpdateFrequency, ViewId } from "../app.models";

@Injectable({ providedIn: "root" })
export class WorkareaStateService {
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
    gpuAdapters = signal<BackendGpuAdapterUsage[]>([]);
    gpuAdapterHistory = signal<BackendGpuAdapterUsage[][]>([]);

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
        gpuAdapters?: BackendGpuAdapterUsage[];
    }): void {
        this.activeView.set(state.activeView);
        this.totalProcesses.set(state.totalProcesses);
        this.metrics.set(state.metrics);
        this.rows.set(state.rows);
        this.selectedProcess.set(state.selectedProcess);
        this.bars.set(state.bars);
        this.activeTitle.set(state.activeTitle);
        this.systemInfo.set(state.systemInfo ?? this.systemInfo());
        this.gpuAdapters.set(state.gpuAdapters ?? this.gpuAdapters());
        this.recordResourceSample(state.metrics);
    }

    setUpdateFrequency(frequency: UpdateFrequency): void {
        this.updateFrequency.set(frequency);
    }

    selectProcess(row: ProcessRow): void {
        this.selectedProcess.set(row.name);
        this.selectedPid.set(row.pid);
    }

    setGpuAdapters(adapters: BackendGpuAdapterUsage[]): void {
        this.gpuAdapters.set(adapters);
        this.gpuAdapterHistory.update((history) => [...history.slice(-59), adapters]);
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
}