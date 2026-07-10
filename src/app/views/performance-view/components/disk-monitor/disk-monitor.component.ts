import { Component, computed, effect, inject, signal } from "@angular/core";
import { BackendDiskDriveUsage } from "../../../../app.models";
import { SelectComponent } from "../../../../components/select/select.component";
import { WorkareaStateService } from "../../../../services/workarea-state.service";

type DiskSortMode = "label" | "index";

@Component({
    selector: "mtx-disk-monitor",
    imports: [SelectComponent],
    templateUrl: "./disk-monitor.component.html",
})
export class DiskMonitorComponent {
    state = inject(WorkareaStateService);
    selectedDiskIndex = signal(0);
    sortMode = signal<DiskSortMode>("index");
    selectedDisk = computed(() => this.diskDrives().find((drive) => drive.diskIndex === this.selectedDiskIndex()) ?? this.diskDrives()[0]);
    sortedDiskDrives = computed(() => {
        const drives = [...this.diskDrives()];
        if (this.sortMode() === "label") {
            return drives.sort((left, right) => {
                const leftLabel = this.primaryLabel(left);
                const rightLabel = this.primaryLabel(right);
                if (!leftLabel && rightLabel) {
                    return 1;
                }

                if (leftLabel && !rightLabel) {
                    return -1;
                }

                return leftLabel.localeCompare(rightLabel) || left.diskIndex - right.diskIndex;
            });
        }

        return drives.sort((left, right) => left.diskIndex - right.diskIndex);
    });

    constructor() {
        effect(() => {
            const drives = this.diskDrives();
            if (drives.length > 0 && !drives.some((drive) => drive.diskIndex === this.selectedDiskIndex())) {
                this.selectedDiskIndex.set(drives[0].diskIndex);
            }
        });
    }

    diskDrives(): BackendDiskDriveUsage[] {
        return this.state.diskDrives();
    }

    setSortMode(value: string): void {
        this.sortMode.set(value === "label" ? "label" : "index");
    }

    selectDisk(drive: BackendDiskDriveUsage): void {
        this.selectedDiskIndex.set(drive.diskIndex);
    }

    isSelected(drive: BackendDiskDriveUsage): boolean {
        return drive.diskIndex === this.selectedDisk()?.diskIndex;
    }

    usage(value: number): string {
        return `${Math.max(0, Math.min(100, value)).toFixed(0)}%`;
    }

    milliseconds(value: number): string {
        return `${Math.max(0, value).toFixed(0)} ms`;
    }

    yesNo(value?: boolean): string {
        return value == null ? "Unavailable" : value ? "Yes" : "No";
    }

    formatBytes(bytes?: number): string {
        if (bytes == null) {
            return "Unavailable";
        }

        if (bytes >= 1024 * 1024 * 1024 * 1024) {
            return `${(bytes / 1024 / 1024 / 1024 / 1024).toFixed(1)} TB`;
        }

        if (bytes >= 1024 * 1024 * 1024) {
            return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
        }

        if (bytes >= 1024 * 1024) {
            return `${(bytes / 1024 / 1024).toFixed(0)} MB`;
        }

        if (bytes >= 1024) {
            return `${(bytes / 1024).toFixed(0)} KB`;
        }

        return `${bytes} B`;
    }

    speed(bytes: number): string {
        return `${this.formatBytes(bytes)}/s`;
    }

    diskPath(diskIndex: number): string {
        const history = this.state.diskDriveHistory();
        if (history.length === 0) {
            return "0,52 120,52";
        }

        return history.map((drives, index) => {
            const x = history.length === 1 ? 120 : index * (120 / (history.length - 1));
            const value = drives.find((drive) => drive.diskIndex === diskIndex)?.activeTimePercent ?? 0;
            return `${x.toFixed(1)},${(60 - Math.max(0, Math.min(100, value)) * 0.52).toFixed(1)}`;
        }).join(" ");
    }

    diskAreaPath(diskIndex: number): string {
        return `0,60 ${this.diskPath(diskIndex)} 120,60`;
    }

    private primaryLabel(drive: BackendDiskDriveUsage): string {
        return drive.labels[0] ?? "";
    }
}