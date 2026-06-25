import { NgClass } from "@angular/common";
import { Component, inject, input, output } from "@angular/core";
import { WorkareaStateService } from "../../../../services/workarea-state.service";
import { PerformanceMetric, PerformanceNavItem } from "../../performance-view.models";

@Component({
    selector: "mtx-performance-resource-nav",
    imports: [NgClass],
    host: { class: "block h-full min-h-0" },
    templateUrl: "./performance-resource-nav.component.html",
})
export class PerformanceResourceNavComponent {
    private state = inject(WorkareaStateService);

    items = input.required<PerformanceNavItem[]>();
    selectedMetric = input.required<PerformanceMetric>();
    metricSelected = output<PerformanceMetric>();

    metricValue(label: string): string {
        return this.state.metrics().find((metric) => metric.label === label)?.value ?? "0%";
    }

    metricDetail(label: string): string {
        return this.state.metrics().find((metric) => metric.label === label)?.detail ?? "-";
    }
}