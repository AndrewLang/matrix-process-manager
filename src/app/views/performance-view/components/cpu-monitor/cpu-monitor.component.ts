import { Component, HostListener, inject, signal } from "@angular/core";
import { WorkareaStateService } from "../../../../services/workarea-state.service";

@Component({
    selector: "mtx-cpu-monitor",
    templateUrl: "./cpu-monitor.component.html",
})
export class CpuMonitorComponent {
    state = inject(WorkareaStateService);
    viewportWidth = signal(window.innerWidth);
    logicalProcessors = Array.from({ length: Math.min(32, navigator.hardwareConcurrency || 8) }, (_, index) => index);

    logicalProcessorColumns(): number {
        const maxColumns = this.viewportWidth() < 1120 ? 4 : 8;
        return Math.min(maxColumns, Math.max(1, this.logicalProcessors.length));
    }

    @HostListener("window:resize")
    updateViewportWidth(): void {
        this.viewportWidth.set(window.innerWidth);
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

    metricValue(label: string): string {
        return this.state.metrics().find((metric) => metric.label === label)?.value ?? "0%";
    }

    systemInfoValue(label: string): string {
        return this.state.systemInfo().find((item) => item.label === label)?.value ?? "Unavailable";
    }
}