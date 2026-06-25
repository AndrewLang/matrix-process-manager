import { Component, inject } from "@angular/core";
import { WorkareaStateService } from "../../../../services/workarea-state.service";
import { ResourceDetailMonitorComponent } from "../resource-detail-monitor/resource-detail-monitor.component";

@Component({
    selector: "mtx-gpu-monitor",
    imports: [ResourceDetailMonitorComponent],
    templateUrl: "./gpu-monitor.component.html",
})
export class GpuMonitorComponent {
    state = inject(WorkareaStateService);
    gpuEngines = ["3D", "Copy", "Video Decode", "Video Encode"];

    metricValue(label: string): string {
        return this.state.metrics().find((metric) => metric.label === label)?.value ?? "0%";
    }

    latest(): number {
        const history = this.state.resourceHistory();
        return history.at(-1)?.gpu ?? 0;
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
}