import { NgClass } from "@angular/common";
import { Component, HostListener, computed, input, output, signal } from "@angular/core";
import { ProcessGroup, ProcessRow } from "../../app.models";

interface ProcessColumn {
    key: string;
    label: string;
    width: number;
    minWidth: number;
    resizable: boolean;
}

interface ProcessNameGroup {
    name: string;
    rows: ProcessRow[];
}

interface ProcessSection {
    key: ProcessGroup;
    label: string;
    groups: ProcessNameGroup[];
    count: number;
}

@Component({
    selector: "mtx-process-grid",
    imports: [NgClass],
    templateUrl: "./process-grid.component.html",
})
export class ProcessGridComponent {
    rows = input.required<ProcessRow[]>();
    selectedProcess = input.required<string>();
    processSelected = output<ProcessRow>();
    collapsedGroups = signal<ReadonlySet<string>>(new Set());

    columns = signal<ProcessColumn[]>([
        { key: "select", label: "", width: 32, minWidth: 32, resizable: false },
        { key: "name", label: "Name", width: 230, minWidth: 150, resizable: true },
        { key: "cpu", label: "CPU", width: 72, minWidth: 60, resizable: true },
        { key: "gpu", label: "GPU", width: 72, minWidth: 60, resizable: true },
        { key: "memory", label: "Memory", width: 104, minWidth: 82, resizable: true },
        { key: "disk", label: "Disk", width: 102, minWidth: 82, resizable: true },
        { key: "network", label: "Network", width: 104, minWidth: 82, resizable: true },
        { key: "pid", label: "PID", width: 72, minWidth: 56, resizable: true },
        { key: "user", label: "User", width: 92, minWidth: 72, resizable: true },
        { key: "menu", label: "", width: 36, minWidth: 36, resizable: false },
    ]);

    sections = computed<ProcessSection[]>(() => {
        const sections = new Map<ProcessGroup, Map<string, ProcessRow[]>>([
            ["apps", new Map<string, ProcessRow[]>()],
            ["background", new Map<string, ProcessRow[]>()],
            ["windows", new Map<string, ProcessRow[]>()],
        ]);

        for (const row of this.rows()) {
            const section = sections.get(row.processGroup ?? "apps")!;
            const nameRows = section.get(row.name) ?? [];
            nameRows.push(row);
            section.set(row.name, nameRows);
        }

        const processSections: ProcessSection[] = [
            { key: "apps", label: "Apps", groups: this.toNameGroups(sections.get("apps")!), count: this.countRows(sections.get("apps")!) },
            { key: "background", label: "Background processes", groups: this.toNameGroups(sections.get("background")!), count: this.countRows(sections.get("background")!) },
            { key: "windows", label: "Windows processes", groups: this.toNameGroups(sections.get("windows")!), count: this.countRows(sections.get("windows")!) },
        ];

        return processSections.filter((section) => section.count > 0);
    });

    private resizing?: { index: number; startX: number; startWidth: number };

    startResize(event: MouseEvent, index: number): void {
        event.preventDefault();
        event.stopPropagation();
        const column = this.columns()[index];
        if (!column.resizable) {
            return;
        }

        this.resizing = { index, startX: event.clientX, startWidth: column.width };
    }

    isSelected(row: ProcessRow): boolean {
        return row.selected || this.selectedProcess() === row.name;
    }

    groupCpu(group: ProcessNameGroup): string {
        return `${group.rows.reduce((total, row) => total + Number.parseFloat(row.cpu), 0).toFixed(1)}%`;
    }

    groupGpu(group: ProcessNameGroup): string {
        return `${group.rows.reduce((total, row) => total + Number.parseFloat(row.gpu), 0).toFixed(1)}%`;
    }

    groupPublisher(group: ProcessNameGroup): string {
        return group.rows[0]?.publisher ?? "Unknown publisher";
    }

    groupIcon(group: ProcessNameGroup): string {
        return group.rows[0]?.iconClass ?? "bi-window";
    }

    groupIconDataUrl(group: ProcessNameGroup): string | undefined {
        return group.rows.find((row) => row.iconDataUrl)?.iconDataUrl;
    }

    groupMemory(group: ProcessNameGroup): string {
        return this.formatBytes(group.rows.reduce((total, row) => total + this.parseBytes(row.memory), 0));
    }

    isGroupExpanded(section: ProcessSection, group: ProcessNameGroup): boolean {
        return !this.collapsedGroups().has(this.groupKey(section, group));
    }

    toggleGroup(section: ProcessSection, group: ProcessNameGroup): void {
        const key = this.groupKey(section, group);
        this.collapsedGroups.update((groups) => {
            const nextGroups = new Set(groups);
            if (nextGroups.has(key)) {
                nextGroups.delete(key);
            } else {
                nextGroups.add(key);
            }

            return nextGroups;
        });
    }

    expandAllGroups(): void {
        this.collapsedGroups.set(new Set());
    }

    collapseAllGroups(): void {
        this.collapsedGroups.set(new Set(this.sections().flatMap((section) => section.groups.map((group) => this.groupKey(section, group)))));
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

    private toNameGroups(groups: Map<string, ProcessRow[]>): ProcessNameGroup[] {
        return Array.from(groups.entries()).map(([name, rows]) => ({ name, rows }));
    }

    private countRows(groups: Map<string, ProcessRow[]>): number {
        return Array.from(groups.values()).reduce((total, rows) => total + rows.length, 0);
    }

    private groupKey(section: ProcessSection, group: ProcessNameGroup): string {
        return `${section.key}:${group.name}`;
    }

    private parseBytes(value: string): number {
        const amount = Number.parseFloat(value) || 0;
        if (value.includes("GB")) {
            return amount * 1024 * 1024 * 1024;
        }

        if (value.includes("MB")) {
            return amount * 1024 * 1024;
        }

        if (value.includes("KB")) {
            return amount * 1024;
        }

        return amount;
    }

    private formatBytes(bytes: number): string {
        if (bytes >= 1024 * 1024 * 1024) {
            return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
        }

        if (bytes >= 1024 * 1024) {
            return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
        }

        if (bytes >= 1024) {
            return `${(bytes / 1024).toFixed(1)} KB`;
        }

        return `${bytes} B`;
    }
}
