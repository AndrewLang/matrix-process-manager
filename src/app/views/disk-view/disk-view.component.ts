import { Component, OnInit, computed, inject, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { BackendDiskDriveUsage, DiskCleanupResult, DiskCleanupScan, DiskCleanupTarget, DiskVolumeUsage } from "../../app.models";
import { WorkareaStateService } from "../../services/workarea-state.service";

@Component({
    selector: "mtx-disk-view",
    templateUrl: "./disk-view.component.html",
})
export class DiskViewComponent implements OnInit {
    state = inject(WorkareaStateService);
    scan = signal<DiskCleanupScan | undefined>(undefined);
    selectedTargetIds = signal<string[]>([]);
    loading = signal(false);
    cleaning = signal(false);
    error = signal("");
    lastReleasedBytes = signal<number | undefined>(undefined);

    systemVolume = computed(() => this.scan()?.volumes.find((volume) => volume.systemDrive) ?? this.scan()?.volumes[0]);
    systemDrive = computed(() => this.state.diskDrives().find((drive) => drive.systemDisk) ?? this.state.diskDrives()[0]);
    systemDriveName = computed(() => this.systemDrive()?.name || this.systemVolume()?.name || "No system drive detected");
    cleanupTargets = computed(() => [...(this.scan()?.targets ?? [])].sort((left, right) => right.bytes - left.bytes));
    visibleCleanupTargets = computed(() => this.cleanupTargets().filter((target) => target.exists || target.bytes > 0));
    selectedTargets = computed(() => this.cleanupTargets().filter((target) => this.selectedTargetIds().includes(target.id)));
    selectedBytes = computed(() => this.selectedTargets().reduce((total, target) => total + target.bytes, 0));
    cleanableBytes = computed(() => this.cleanupTargets().reduce((total, target) => total + target.bytes, 0));
    hasSelection = computed(() => this.selectedTargetIds().length > 0);

    ngOnInit(): void {
        this.refresh();
    }

    refresh(): void {
        this.loading.set(true);
        this.error.set("");
        invoke<DiskCleanupScan>("get_disk_cleanup_scan")
            .then((scan) => {
                this.scan.set(scan);
                this.selectedTargetIds.set(scan.targets.filter((target) => target.exists && target.bytes > 0).map((target) => target.id));
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Disk scan failed."))
            .finally(() => this.loading.set(false));
    }

    toggleTarget(target: DiskCleanupTarget): void {
        this.selectedTargetIds.update((targetIds) => targetIds.includes(target.id) ? targetIds.filter((targetId) => targetId !== target.id) : [...targetIds, target.id]);
    }

    cleanSelected(): void {
        const targetIds = this.selectedTargetIds();
        if (targetIds.length === 0 || this.cleaning()) {
            return;
        }

        this.cleaning.set(true);
        this.error.set("");
        invoke<DiskCleanupResult>("clean_disk", { request: { targetIds } })
            .then((result) => {
                this.lastReleasedBytes.set(result.releasedBytes);
                this.refresh();
            })
            .catch((error: unknown) => this.error.set(error instanceof Error ? error.message : "Disk clean failed."))
            .finally(() => this.cleaning.set(false));
    }

    usagePercent(volume: DiskVolumeUsage | undefined): number {
        if (!volume?.totalBytes) {
            return 0;
        }

        return Math.max(0, Math.min(100, (volume.totalBytes - volume.freeBytes) / volume.totalBytes * 100));
    }

    freePercent(volume: DiskVolumeUsage | undefined): number {
        return Math.max(0, 100 - this.usagePercent(volume));
    }

    driveUsage(drive: BackendDiskDriveUsage | undefined): string {
        return drive ? `${Math.max(0, Math.min(100, drive.activeTimePercent)).toFixed(0)}%` : "0%";
    }

    targetSelected(target: DiskCleanupTarget): boolean {
        return this.selectedTargetIds().includes(target.id);
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
}