import { Component, inject } from "@angular/core";
import { IconComponent } from "../../components/icon/icon.component";
import { WorkareaStateService } from "../../services/workarea-state.service";

@Component({
    selector: "mtx-system-info-view",
    imports: [IconComponent],
    templateUrl: "./system-info-view.component.html",
})
export class SystemInfoViewComponent {
    state = inject(WorkareaStateService);

    hardwareCards() {
        return [
            { icon: "bi-cpu", label: "Processor", value: this.info("CPU model"), detail: this.info("CPU current speed") },
            { icon: "bi-memory", label: "Installed RAM", value: this.installedRam(), detail: this.memorySpeed() },
            { icon: "bi-gpu-card", label: "Graphics card", value: this.graphicsMemory(), detail: this.graphicsSummary() },
            { icon: "bi-device-hdd", label: "Storage", value: this.storageTotal(), detail: this.storageUsed() },
        ];
    }

    deviceInfo() {
        return [
            { label: "Device name", value: this.info("Device name") },
            { label: "Processor", value: `${this.info("CPU model")} (${this.info("CPU current speed")})` },
            { label: "Installed RAM", value: this.installedRamDetail() },
            { label: "Graphics card", value: this.graphicsList() },
            { label: "Storage", value: this.storageUsed() },
            { label: "Device ID", value: this.info("Device ID") },
            { label: "Product ID", value: this.info("Product ID") },
            { label: "System type", value: this.info("System type") },
            { label: "Input support", value: "Standard input devices" },
        ];
    }

    operatingSystemInfo() {
        return [
            { label: "Edition", value: this.info("OS edition") },
            { label: "Version", value: this.info("OS version") },
            { label: "Installed on", value: this.info("Installed on") },
            { label: "OS build", value: this.info("OS build") },
            { label: "System firmware", value: this.info("System firmware") },
        ];
    }

    info(label: string): string {
        return this.state.systemInfo().find((item) => item.label === label)?.value ?? "Unavailable";
    }

    private installedRam(): string {
        const installed = this.state.memoryInfo()?.installedBytes;
        return installed ? this.formatBytes(installed) : this.info("Memory");
    }

    private installedRamDetail(): string {
        const installed = this.installedRam();
        const usable = this.info("Memory");
        return usable === "Unavailable" ? installed : `${installed} (${usable} usable)`;
    }

    private memorySpeed(): string {
        const speed = this.state.memoryInfo()?.speedMhz;
        return speed ? `Speed: ${speed} MT/s` : "Speed unavailable";
    }

    private graphicsMemory(): string {
        const adapters = this.state.gpuAdapters();
        return adapters.length > 1 ? `${adapters.length} GPUs` : adapters[0]?.name ?? "Unavailable";
    }

    private graphicsSummary(): string {
        const adapters = this.state.gpuAdapters();
        if (adapters.length === 0) {
            return "No GPU adapters detected";
        }

        return adapters.length > 1 ? "Multiple GPUs installed" : adapters[0].name;
    }

    private graphicsList(): string {
        const adapters = this.state.gpuAdapters();
        return adapters.length > 0 ? adapters.map((adapter) => adapter.name).join("\n") : "Unavailable";
    }

    private storageTotal(): string {
        const total = this.state.diskDrives().reduce((sum, drive) => sum + (drive.capacityBytes ?? 0), 0);
        return total > 0 ? this.formatBytes(total) : "Unavailable";
    }

    private storageUsed(): string {
        const drives = this.state.diskDrives();
        const total = drives.reduce((sum, drive) => sum + (drive.capacityBytes ?? 0), 0);
        const formatted = drives.reduce((sum, drive) => sum + (drive.formattedBytes ?? 0), 0);
        if (total === 0) {
            return "Unavailable";
        }

        return formatted > 0 ? `${this.formatBytes(formatted)} of ${this.formatBytes(total)} used` : this.formatBytes(total);
    }

    private formatBytes(bytes: number): string {
        if (bytes >= 1024 * 1024 * 1024 * 1024) {
            return `${(bytes / 1024 / 1024 / 1024 / 1024).toFixed(2)} TB`;
        }

        if (bytes >= 1024 * 1024 * 1024) {
            return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
        }

        if (bytes >= 1024 * 1024) {
            return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
        }

        return `${bytes} B`;
    }
}