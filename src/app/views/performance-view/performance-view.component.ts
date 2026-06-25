import { NgClass } from "@angular/common";
import { Component, inject, signal } from "@angular/core";
import { MetricBlockComponent } from "../../components/metric-block/metric-block.component";
import { MiniConsumersComponent } from "../../components/mini-consumers/mini-consumers.component";
import { ResourceBarsComponent } from "../../components/resource-bars/resource-bars.component";
import { WorkareaStateService } from "../../services/workarea-state.service";

@Component({
    selector: "mtx-performance-view",
    imports: [NgClass, MetricBlockComponent, MiniConsumersComponent, ResourceBarsComponent],
    templateUrl: "./performance-view.component.html",
})
export class PerformanceViewComponent {
    state = inject(WorkareaStateService);
    detailsOpen = signal(true);

    logicalProcessors = Array.from({ length: Math.min(32, navigator.hardwareConcurrency || 8) }, (_, index) => index);
    gpuEngines = ["3D", "Copy", "Video Decode", "Video Encode"];

    chartPath(metric: "cpu" | "gpu" | "memory" | "disk" | "network"): string {
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

    latest(metric: "cpu" | "gpu" | "memory" | "disk" | "network"): number {
        const history = this.state.resourceHistory();
        return history.at(-1)?.[metric] ?? 0;
    }

    closeDetails(): void {
        this.detailsOpen.set(false);
    }
}