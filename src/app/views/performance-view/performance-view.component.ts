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

    closeDetails(): void {
        this.detailsOpen.set(false);
    }
}