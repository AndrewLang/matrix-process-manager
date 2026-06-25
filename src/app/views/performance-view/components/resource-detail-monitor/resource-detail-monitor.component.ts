import { NgClass } from "@angular/common";
import { Component, inject, input } from "@angular/core";
import { WorkareaStateService } from "../../../../services/workarea-state.service";
import { PerformanceMetric } from "../../performance-view.models";

@Component({
    selector: "mtx-resource-detail-monitor",
    imports: [NgClass],
    templateUrl: "./resource-detail-monitor.component.html",
})
export class ResourceDetailMonitorComponent {
    state = inject(WorkareaStateService);

    metric = input.required<PerformanceMetric>();
    label = input.required<string>();
    accent = input.required<string>();

    chartPath(): string {
        const metric = this.metric();
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

    chartAreaPath(): string {
        return `0,104 ${this.chartPath()} 320,104`;
    }

    metricValue(): string {
        return this.state.metrics().find((metric) => metric.label === this.label())?.value ?? "0%";
    }

    metricDetail(): string {
        return this.state.metrics().find((metric) => metric.label === this.label())?.detail ?? "-";
    }

    lineWidth(): string {
        return this.metric() === "gpu" ? "0.55px" : "0.8px";
    }

    latest(): number {
        const history = this.state.resourceHistory();
        return history.at(-1)?.[this.metric()] ?? 0;
    }
}