import { NgClass } from "@angular/common";
import { Component, HostListener, inject, signal } from "@angular/core";
import { MiniConsumersComponent } from "../../components/mini-consumers/mini-consumers.component";
import { ResourceBarsComponent } from "../../components/resource-bars/resource-bars.component";
import { WorkareaStateService } from "../../services/workarea-state.service";

type PerformanceMetric = "cpu" | "gpu" | "memory" | "network" | "disk";

interface PerformanceNavItem {
    key: PerformanceMetric;
    label: string;
    icon: string;
    accent: string;
}

@Component({
    selector: "mtx-performance-view",
    imports: [NgClass, MiniConsumersComponent, ResourceBarsComponent],
    templateUrl: "./performance-view.component.html",
})
export class PerformanceViewComponent {
    state = inject(WorkareaStateService);
    detailsOpen = signal(true);
    selectedMetric = signal<PerformanceMetric>("cpu");
    viewportWidth = signal(window.innerWidth);

    logicalProcessors = Array.from({ length: Math.min(32, navigator.hardwareConcurrency || 8) }, (_, index) => index);
    gpuEngines = ["3D", "Copy", "Video Decode", "Video Encode"];
    resourceNav: PerformanceNavItem[] = [
        { key: "cpu", label: "CPU", icon: "bi-cpu", accent: "text-(--blue)" },
        { key: "gpu", label: "GPU", icon: "bi-gpu-card", accent: "text-(--cyan)" },
        { key: "memory", label: "Memory", icon: "bi-memory", accent: "text-(--violet)" },
        { key: "network", label: "Network", icon: "bi-ethernet", accent: "text-(--orange)" },
        { key: "disk", label: "Disk", icon: "bi-device-hdd", accent: "text-(--green)" },
    ];

    logicalProcessorColumns(): number {
        const maxColumns = this.viewportWidth() < 1120 ? 4 : 8;
        return Math.min(maxColumns, Math.max(1, this.logicalProcessors.length));
    }

    @HostListener("window:resize")
    updateViewportWidth(): void {
        this.viewportWidth.set(window.innerWidth);
    }

    chartPath(metric: PerformanceMetric): string {
        const history = this.state.resourceHistory();
        if (history.length === 0) {
            return "0,96 320,96";
        }

        return history.map((sample, index) => {
            const x = history.length === 1 ? 320 : index * (320 / (history.length - 1));
            const y = 104 - Math.max(0, Math.min(100, sample[metric]));
            return `${x.toFixed(1)},${y.toFixed(1)}`;
        }).join(" ");
    }

    setSelectedMetric(metric: PerformanceMetric): void {
        this.selectedMetric.set(metric);
    }

    selectedNavItem(): PerformanceNavItem {
        return this.resourceNav.find((item) => item.key === this.selectedMetric()) ?? this.resourceNav[0];
    }

    selectedMetricValue(): string {
        return this.metricValue(this.selectedNavItem().label);
    }

    selectedMetricDetail(): string {
        return this.metricDetail(this.selectedNavItem().label);
    }

    corePath(core: number): string {
        const history = this.state.resourceHistory();
        if (history.length === 0) {
            return "0,52 120,52";
        }

        return history.map((sample, index) => {
            const x = history.length === 1 ? 120 : index * (120 / (history.length - 1));
            const variation = Math.sin(index + core * 1.7) * 9 + Math.cos(index * 0.6 + core) * 6;
            const value = Math.max(0, Math.min(100, sample.cpu + variation));
            return `${x.toFixed(1)},${(60 - value * 0.52).toFixed(1)}`;
        }).join(" ");
    }

    coreAreaPoints(core: number): string {
        return `${this.corePath(core)} 120,64 0,64`;
    }

    enginePath(engine: number): string {
        const history = this.state.resourceHistory();
        if (history.length === 0) {
            return "0,52 120,52";
        }

        return history.map((sample, index) => {
            const x = history.length === 1 ? 120 : index * (120 / (history.length - 1));
            const value = Math.max(0, Math.min(100, sample.gpu + Math.sin(index + engine) * 4));
            return `${x.toFixed(1)},${(60 - value * 0.52).toFixed(1)}`;
        }).join(" ");
    }

    metricValue(label: string): string {
        return this.state.metrics().find((metric) => metric.label === label)?.value ?? "0%";
    }

    metricDetail(label: string): string {
        return this.state.metrics().find((metric) => metric.label === label)?.detail ?? "-";
    }

    systemInfoValue(label: string): string {
        return this.state.systemInfo().find((item) => item.label === label)?.value ?? "Unavailable";
    }

    latest(metric: PerformanceMetric): number {
        const history = this.state.resourceHistory();
        return history.at(-1)?.[metric] ?? 0;
    }

    closeDetails(): void {
        this.detailsOpen.set(false);
    }
}