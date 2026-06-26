import { NgClass } from "@angular/common";
import { Component, HostListener, OnInit, computed, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { StartupApp } from "../../app.models";
import { SearchBoxComponent } from "../../components/search-box/search-box.component";

interface StartupColumn {
    key: string;
    label: string;
    width: number;
    minWidth: number;
    align?: "left" | "right";
    resizable: boolean;
}

@Component({
    selector: "mtx-startup-view",
    imports: [NgClass, SearchBoxComponent],
    templateUrl: "./startup-view.component.html",
})
export class StartupViewComponent implements OnInit {
    apps = signal<StartupApp[]>([]);
    selectedName = signal<string | undefined>(undefined);
    filter = signal("");
    sourceFilter = signal("All Sources");
    impactFilter = signal("All Impact");
    showDisabled = signal(true);
    loading = signal(false);
    columns = signal<StartupColumn[]>([
        { key: "name", label: "Name", width: 260, minWidth: 160, resizable: true },
        { key: "publisher", label: "Publisher", width: 160, minWidth: 112, resizable: true },
        { key: "status", label: "Status", width: 96, minWidth: 82, resizable: true },
        { key: "impact", label: "Impact", width: 96, minWidth: 82, resizable: true },
        { key: "startupType", label: "Startup Type", width: 132, minWidth: 104, resizable: true },
        { key: "source", label: "Source", width: 132, minWidth: 104, resizable: true },
        { key: "actions", label: "Actions", width: 92, minWidth: 76, align: "right", resizable: true },
    ]);

    selectedApp = computed<StartupApp | undefined>(() => this.apps().find((app) => app.name === this.selectedName()) ?? this.filteredApps()[0]);
    enabledCount = computed(() => this.apps().filter((app) => app.status === "Enabled").length);
    disabledCount = computed(() => this.apps().filter((app) => app.status !== "Enabled").length);
    highImpactCount = computed(() => this.apps().filter((app) => app.impact === "High").length);
    filteredApps = computed(() => this.filterApps());
    tableWidth = computed(() => this.columns().reduce((total, column) => total + column.width, 0));

    wideFilterSearchClass = "flex h-7.5 min-w-0 flex-1 items-center gap-2 rounded-[5px] border border-(--border) bg-[rgba(15,28,40,0.84)] px-2.5 py-0 text-[12px] text-(--muted)";

    private resizing?: { index: number; startX: number; startWidth: number };

    ngOnInit(): void {
        this.refresh();
    }

    refresh(): void {
        this.loading.set(true);
        invoke<StartupApp[]>("get_startup_apps")
            .then((apps) => {
                this.apps.set(apps);
                if (!this.selectedName() && apps.length > 0) {
                    this.selectedName.set(apps[0].name);
                }
            })
            .catch(() => this.apps.set([]))
            .finally(() => this.loading.set(false));
    }

    selectApp(app: StartupApp): void {
        this.selectedName.set(app.name);
    }

    openLocation(app: StartupApp | undefined): void {
        if (!app?.path) {
            return;
        }

        revealItemInDir(app.path).catch(() => undefined);
    }

    toggleDisabled(): void {
        this.showDisabled.update((value) => !value);
    }

    startResize(event: MouseEvent, index: number): void {
        event.preventDefault();
        event.stopPropagation();
        const column = this.columns()[index];
        if (!column.resizable) {
            return;
        }

        this.resizing = { index, startX: event.clientX, startWidth: column.width };
    }

    @HostListener("document:mousemove", ["$event"])
    resizeColumn(event: MouseEvent): void {
        if (!this.resizing) {
            return;
        }

        const { index, startX, startWidth } = this.resizing;
        this.columns.update((columns) =>
            columns.map((column, columnIndex) =>
                columnIndex === index
                    ? { ...column, width: Math.max(column.minWidth, startWidth + event.clientX - startX) }
                    : column,
            ),
        );
    }

    @HostListener("document:mouseup")
    stopResize(): void {
        this.resizing = undefined;
    }

    impactClass(impact: string): string {
        return impact === "High" ? "text-(--red) bg-[rgba(255,75,67,0.14)]" : impact === "Medium" ? "text-(--orange) bg-[rgba(255,154,46,0.14)]" : "text-(--green) bg-[rgba(34,206,116,0.14)]";
    }

    statusClass(status: string): string {
        return status === "Enabled" ? "text-(--green) bg-[rgba(34,206,116,0.14)]" : "text-(--muted) bg-white/6";
    }

    private filterApps(): StartupApp[] {
        const filter = this.filter().trim().toLowerCase();
        return this.apps().filter((app) => {
            const matchesText = !filter || app.name.toLowerCase().includes(filter) || app.publisher.toLowerCase().includes(filter) || app.command.toLowerCase().includes(filter);
            const matchesSource = this.sourceFilter() === "All Sources" || app.startupType === this.sourceFilter();
            const matchesImpact = this.impactFilter() === "All Impact" || app.impact === this.impactFilter();
            const matchesStatus = this.showDisabled() || app.status === "Enabled";
            return matchesText && matchesSource && matchesImpact && matchesStatus;
        }).sort((first, second) => {
            const firstEnabled = first.status === "Enabled" ? 0 : 1;
            const secondEnabled = second.status === "Enabled" ? 0 : 1;
            return firstEnabled - secondEnabled || first.name.localeCompare(second.name);
        });
    }
}