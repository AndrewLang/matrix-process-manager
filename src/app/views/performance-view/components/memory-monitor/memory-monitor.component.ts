import { Component, inject } from "@angular/core";
import { BackendMemoryInfo } from "../../../../app.models";
import { WorkareaStateService } from "../../../../services/workarea-state.service";
import { ResourceDetailMonitorComponent } from "../resource-detail-monitor/resource-detail-monitor.component";

@Component({
    selector: "mtx-memory-monitor",
    imports: [ResourceDetailMonitorComponent],
    templateUrl: "./memory-monitor.component.html",
})
export class MemoryMonitorComponent {
    state = inject(WorkareaStateService);

    formatBytes(bytes?: number): string {
        if (bytes == null || bytes <= 0) {
            return bytes === 0 ? "0 MB" : "Unavailable";
        }

        const units = ["B", "KB", "MB", "GB", "TB"];
        let value = bytes;
        let unitIndex = 0;

        while (value >= 1024 && unitIndex < units.length - 1) {
            value /= 1024;
            unitIndex += 1;
        }

        return `${value.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
    }

    inUseLabel(): string {
        const info = this.state.memoryInfo();
        if (!info) {
            return "Unavailable";
        }

        const compressed = info.compressedBytes != null ? ` (${this.formatBytes(info.compressedBytes)})` : "";
        return `${this.formatBytes(info.inUseBytes)}${compressed}`;
    }

    committedLabel(): string {
        const info = this.state.memoryInfo();
        if (!info) {
            return "Unavailable";
        }

        return `${this.formatBytes(info.committedBytes)}/${this.formatBytes(info.commitLimitBytes)}`;
    }

    slotsLabel(): string {
        const info = this.state.memoryInfo();
        if (!info?.slotsUsed || !info.slotsTotal) {
            return "Unavailable";
        }

        return `${info.slotsUsed} of ${info.slotsTotal}`;
    }

    speedLabel(): string {
        const speed = this.state.memoryInfo()?.speedMhz;
        return speed ? `${speed} MT/s` : "Unavailable";
    }

    value(label: keyof BackendMemoryInfo, formatter: (value: number) => string = (value) => this.formatBytes(value)): string {
        const info = this.state.memoryInfo();
        const value = info?.[label];
        return typeof value === "number" ? formatter(value) : "Unavailable";
    }
}