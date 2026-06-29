import { NgClass } from "@angular/common";
import { Component, computed, inject, signal } from "@angular/core";
import { Router } from "@angular/router";
import { MetricCard, ProcessRow, ResourceSample } from "../../app.models";
import { TopResourceConsumersComponent } from "../../components/top-resource-consumers/top-resource-consumers.component";
import { WorkareaStateService } from "../../services/workarea-state.service";
import { PerformanceMetric } from "../performance-view/performance-view.models";

@Component({
    selector: "mtx-dashboard-view",
    imports: [NgClass, TopResourceConsumersComponent],
    templateUrl: "./dashboard-view.component.html",
})
export class DashboardViewComponent {
    state = inject(WorkareaStateService);
    private router = inject(Router);
    detailsOpen = signal(true);

    summaryMetrics = computed(() => this.metricOrder().map((label) => this.metric(label)).filter((metric): metric is MetricCard => Boolean(metric)));
    topConsumers = computed(() => this.state.rows().slice(0, 5));
    cpuDetails = computed(() => [
        { label: "Utilization", value: this.metric("CPU")?.value ?? "0%" },
        { label: "Speed", value: this.systemInfoValue("CPU current speed") },
        { label: "Processes", value: this.state.totalProcesses().toString() },
        { label: "Threads", value: this.systemInfoValue("Threads") },
        { label: "Handles", value: this.systemInfoValue("Handles") },
    ]);
    memoryDetails = computed(() => {
        const info = this.state.memoryInfo();
        return [
            { label: "In Use", value: info ? this.formatBytes(info.inUseBytes) : "Unavailable" },
            { label: "Available", value: info ? this.formatBytes(info.availableBytes) : "Unavailable" },
            { label: "Committed", value: info ? `${this.formatBytes(info.committedBytes)} / ${this.formatBytes(info.commitLimitBytes)}` : "Unavailable" },
            { label: "Cached", value: info ? this.formatBytes(info.cachedBytes) : "Unavailable" },
            { label: "Paged Pool", value: info ? this.formatBytes(info.pagedPoolBytes) : "Unavailable" },
            { label: "Non-paged Pool", value: info ? this.formatBytes(info.nonPagedPoolBytes) : "Unavailable" },
        ];
    });
    diskDetails = computed(() => {
        const drives = this.state.diskDrives();
        const read = drives.reduce((total, drive) => total + drive.readBytesPerSec, 0);
        const write = drives.reduce((total, drive) => total + drive.writeBytesPerSec, 0);
        return [
            { label: "Active Time", value: this.metric("Disk")?.value ?? "0%" },
            { label: "Read Speed", value: `${this.formatBytes(read)}/s` },
            { label: "Write Speed", value: `${this.formatBytes(write)}/s` },
            { label: "Drives", value: drives.length.toString() },
        ];
    });
    networkDetails = computed(() => {
        const adapters = this.state.networkAdapters();
        const receive = adapters.reduce((total, adapter) => total + adapter.receiveBytesPerSec, 0);
        const send = adapters.reduce((total, adapter) => total + adapter.sendBytesPerSec, 0);
        return [
            { label: "Send", value: `${this.formatBytes(send)}/s` },
            { label: "Receive", value: `${this.formatBytes(receive)}/s` },
            { label: "Adapters", value: adapters.length.toString() },
        ];
    });
    gpuDetails = computed(() => {
        const adapters = this.state.gpuAdapters();
        return [
            { label: "Utilization", value: this.metric("GPU")?.value ?? "0%" },
            { label: "Adapters", value: adapters.length.toString() },
            { label: "Engines", value: adapters.reduce((total, adapter) => total + adapter.engines.length, 0).toString() },
        ];
    });

    metricOrder(): string[] {
        return ["CPU", "GPU", "Memory", "Disk", "Network"];
    }

    metric(label: string): MetricCard | undefined {
        return this.state.metrics().find((metric) => metric.label === label);
    }

    metricKey(label: string): keyof ResourceSample {
        return label.toLowerCase() as keyof ResourceSample;
    }

    accentClass(accent: string): string {
        return `text-(--${accent})`;
    }

    openPerformanceMetric(label: string): void {
        const metric = this.performanceMetric(label);
        if (metric) {
            this.router.navigate(["performance", metric]);
        }
    }

    closeDetails(): void {
        this.detailsOpen.set(false);
    }

    chartPath(metric: keyof ResourceSample, width = 320, height = 112): string {
        const history = this.state.resourceHistory();
        if (history.length === 0) {
            return `0,${height - 8} ${width},${height - 8}`;
        }

        return history.map((sample, index) => {
            const x = history.length === 1 ? width : index * (width / (history.length - 1));
            const y = height - 8 - Math.max(0, Math.min(100, sample[metric])) / 100 * (height - 16);
            return `${x.toFixed(1)},${y.toFixed(1)}`;
        }).join(" ");
    }

    chartAreaPath(metric: keyof ResourceSample, width = 320, height = 112): string {
        return `0,${height - 8} ${this.chartPath(metric, width, height)} ${width},${height - 8}`;
    }

    rowValue(row: ProcessRow, label: string): string {
        if (label === "CPU") {
            return row.cpu;
        }

        if (label === "Memory") {
            return row.memory;
        }

        if (label === "Disk") {
            return row.disk;
        }

        return row.network;
    }

    systemInfoValue(label: string): string {
        return this.state.systemInfo().find((item) => item.label === label)?.value ?? "Unavailable";
    }

    private performanceMetric(label: string): PerformanceMetric | undefined {
        const metric = label.toLowerCase();
        return metric === "cpu" || metric === "gpu" || metric === "memory" || metric === "disk" || metric === "network" ? metric : undefined;
    }

    private formatBytes(bytes: number): string {
        if (bytes >= 1024 * 1024 * 1024 * 1024) {
            return `${(bytes / 1024 / 1024 / 1024 / 1024).toFixed(1)} TB`;
        }

        if (bytes >= 1024 * 1024 * 1024) {
            return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
        }

        if (bytes >= 1024 * 1024) {
            return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
        }

        if (bytes >= 1024) {
            return `${(bytes / 1024).toFixed(0)} KB`;
        }

        return `${bytes} B`;
    }
}