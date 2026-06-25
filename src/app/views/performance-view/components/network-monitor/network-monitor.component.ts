import { Component, computed, effect, inject, signal } from "@angular/core";
import { BackendNetworkAdapterUsage } from "../../../../app.models";
import { WorkareaStateService } from "../../../../services/workarea-state.service";

@Component({
    selector: "mtx-network-monitor",
    templateUrl: "./network-monitor.component.html",
})
export class NetworkMonitorComponent {
    state = inject(WorkareaStateService);
    selectedAdapterIndex = signal(0);
    selectedAdapter = computed(() => this.networkAdapters().find((adapter) => adapter.adapterIndex === this.selectedAdapterIndex()) ?? this.networkAdapters()[0]);

    constructor() {
        effect(() => {
            const adapters = this.networkAdapters();
            if (adapters.length > 0 && !adapters.some((adapter) => adapter.adapterIndex === this.selectedAdapterIndex())) {
                this.selectedAdapterIndex.set(adapters[0].adapterIndex);
            }
        });
    }

    networkAdapters(): BackendNetworkAdapterUsage[] {
        return this.state.networkAdapters();
    }

    selectAdapter(adapter: BackendNetworkAdapterUsage): void {
        this.selectedAdapterIndex.set(adapter.adapterIndex);
    }

    isSelected(adapter: BackendNetworkAdapterUsage): boolean {
        return adapter.adapterIndex === this.selectedAdapter()?.adapterIndex;
    }

    usage(value: number): string {
        return `${Math.max(0, Math.min(100, value)).toFixed(0)}%`;
    }

    speed(bytes: number): string {
        return `${this.formatBytes(bytes)}/s`;
    }

    traffic(adapter: BackendNetworkAdapterUsage): string {
        return this.speed(adapter.receiveBytesPerSec + adapter.sendBytesPerSec);
    }

    linkSpeed(bits?: number): string {
        if (bits == null) {
            return "Unavailable";
        }

        if (bits >= 1000 * 1000 * 1000) {
            return `${(bits / 1000 / 1000 / 1000).toFixed(1)} Gbps`;
        }

        if (bits >= 1000 * 1000) {
            return `${(bits / 1000 / 1000).toFixed(0)} Mbps`;
        }

        if (bits >= 1000) {
            return `${(bits / 1000).toFixed(0)} Kbps`;
        }

        return `${bits} bps`;
    }

    adapterPath(adapterIndex: number): string {
        const history = this.state.networkAdapterHistory();
        if (history.length === 0) {
            return "0,52 120,52";
        }

        const maxTraffic = Math.max(...history.map((adapters) => {
            const adapter = adapters.find((adapter) => adapter.adapterIndex === adapterIndex);
            return (adapter?.receiveBytesPerSec ?? 0) + (adapter?.sendBytesPerSec ?? 0);
        }), 1);

        return history.map((adapters, index) => {
            const x = history.length === 1 ? 120 : index * (120 / (history.length - 1));
            const adapter = adapters.find((adapter) => adapter.adapterIndex === adapterIndex);
            const value = (adapter?.receiveBytesPerSec ?? 0) + (adapter?.sendBytesPerSec ?? 0);
            return `${x.toFixed(1)},${(60 - Math.max(0, Math.min(1, value / maxTraffic)) * 52).toFixed(1)}`;
        }).join(" ");
    }

    adapterAreaPath(adapterIndex: number): string {
        return `0,60 ${this.adapterPath(adapterIndex)} 120,60`;
    }

    private formatBytes(bytes: number): string {
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