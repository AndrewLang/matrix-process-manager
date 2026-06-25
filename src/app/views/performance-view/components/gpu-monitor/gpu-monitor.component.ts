import { Component, computed, effect, inject, signal } from "@angular/core";
import { BackendGpuAdapterUsage } from "../../../../app.models";
import { WorkareaStateService } from "../../../../services/workarea-state.service";
import { ResourceDetailMonitorComponent } from "../resource-detail-monitor/resource-detail-monitor.component";

@Component({
    selector: "mtx-gpu-monitor",
    imports: [ResourceDetailMonitorComponent],
    templateUrl: "./gpu-monitor.component.html",
})
export class GpuMonitorComponent {
    state = inject(WorkareaStateService);
    selectedAdapterIndex = signal(0);
    selectedAdapter = computed(() => this.gpuAdapters().find((adapter) => adapter.adapterIndex === this.selectedAdapterIndex()) ?? this.gpuAdapters()[0]);

    constructor() {
        effect(() => {
            const adapters = this.gpuAdapters();
            if (adapters.length > 0 && !adapters.some((adapter) => adapter.adapterIndex === this.selectedAdapterIndex())) {
                this.selectedAdapterIndex.set(adapters[0].adapterIndex);
            }
        });
    }

    metricValue(label: string): string {
        return this.state.metrics().find((metric) => metric.label === label)?.value ?? "0%";
    }

    gpuAdapters() {
        return this.state.gpuAdapters();
    }

    selectAdapter(adapter: BackendGpuAdapterUsage): void {
        this.selectedAdapterIndex.set(adapter.adapterIndex);
    }

    isSelected(adapter: BackendGpuAdapterUsage): boolean {
        return adapter.adapterIndex === this.selectedAdapter()?.adapterIndex;
    }

    usage(value: number): string {
        return `${Math.max(0, Math.min(100, value)).toFixed(0)}%`;
    }

    width(value: number): string {
        return `${Math.max(0, Math.min(100, value)).toFixed(1)}%`;
    }

    enginePath(adapterIndex: number, engineName: string): string {
        const history = this.state.gpuAdapterHistory();
        if (history.length === 0) {
            return "0,52 120,52";
        }

        return history.map((adapters, index) => {
            const x = history.length === 1 ? 120 : index * (120 / (history.length - 1));
            const adapter = adapters.find((item) => item.adapterIndex === adapterIndex);
            const value = adapter?.engines.find((engine) => engine.name === engineName)?.utilizationPercent ?? 0;
            return `${x.toFixed(1)},${(60 - Math.max(0, Math.min(100, value)) * 0.52).toFixed(1)}`;
        }).join(" ");
    }
}