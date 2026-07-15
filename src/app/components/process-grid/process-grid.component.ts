import { NgClass } from "@angular/common";
import { Component, HostListener, computed, input, output, signal } from "@angular/core";
import { ProcessGroup, ProcessRow } from "../../app.models";
import { IconComponent } from "../icon/icon.component";

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
    processGroup: ProcessGroup;
}

interface ProcessSection {
    key: ProcessGroup;
    label: string;
    groups: ProcessNameGroup[];
    count: number;
}

@Component({
    selector: "mtx-process-grid",
    imports: [NgClass, IconComponent],
    templateUrl: "./process-grid.component.html",
})
export class ProcessGridComponent {
    private readonly collapsedGroupsKey = "prism.process-grid.collapsed-groups";
    private readonly legacyCollapsedGroupsKey = "matrix-process-manager.process-grid.collapsed-groups";
    rows = input.required<ProcessRow[]>();
    selectedProcess = input<ProcessRow | undefined>();
    processSelected = output<ProcessRow>();
    collapsedGroups = signal<ReadonlySet<string>>(this.loadCollapsedGroups());

    columns = signal<ProcessColumn[]>([
        { key: "select", label: "", width: 32, minWidth: 32, resizable: false },
        { key: "name", label: "Name", width: 230, minWidth: 150, resizable: true },
        { key: "type", label: "Type", width: 132, minWidth: 104, resizable: true },
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
        const groupsByName = new Map<string, ProcessRow[]>();

        for (const row of this.rows()) {
            const nameRows = groupsByName.get(row.name) ?? [];
            nameRows.push(row);
            groupsByName.set(row.name, nameRows);
        }

        const sections = new Map<ProcessGroup, ProcessNameGroup[]>([
            ["apps", []],
            ["background", []],
            ["windows", []],
        ]);

        for (const [name, rows] of groupsByName) {
            const processGroup = this.groupProcessType(rows);
            sections.get(processGroup)!.push({ name, rows, processGroup });
        }

        const processSections: ProcessSection[] = [
            { key: "apps", label: "Apps", groups: sections.get("apps")!, count: this.countRows(sections.get("apps")!) },
            { key: "background", label: "Background processes", groups: sections.get("background")!, count: this.countRows(sections.get("background")!) },
            { key: "windows", label: "Windows processes", groups: sections.get("windows")!, count: this.countRows(sections.get("windows")!) },
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
        return this.selectedProcess()?.pid === row.pid;
    }

    selectGroup(group: ProcessNameGroup): void {
        const row = group.rows[0];
        if (row) {
            this.processSelected.emit(row);
        }
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

    groupType(group: ProcessNameGroup): string {
        return this.processTypeLabel(group.processGroup);
    }

    rowType(row: ProcessRow): string {
        return this.processTypeLabel(row.processGroup ?? "apps");
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

            this.saveCollapsedGroups(nextGroups);
            return nextGroups;
        });
    }

    expandAllGroups(): void {
        const groups = new Set<string>();
        this.collapsedGroups.set(groups);
        this.saveCollapsedGroups(groups);
    }

    collapseAllGroups(): void {
        const groups = new Set(this.sections().flatMap((section) => section.groups.map((group) => this.groupKey(section, group))));
        this.collapsedGroups.set(groups);
        this.saveCollapsedGroups(groups);
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

    private groupProcessType(rows: ProcessRow[]): ProcessGroup {
        if (rows.some((row) => row.processGroup === "apps")) {
            return "apps";
        }

        if (rows.some((row) => row.processGroup === "background")) {
            return "background";
        }

        return "windows";
    }

    private countRows(groups: ProcessNameGroup[]): number {
        return groups.reduce((total, group) => total + group.rows.length, 0);
    }

    private groupKey(section: ProcessSection, group: ProcessNameGroup): string {
        return `${section.key}:${group.name}`;
    }

    private loadCollapsedGroups(): ReadonlySet<string> {
        try {
            const value = JSON.parse(localStorage.getItem(this.collapsedGroupsKey) ?? localStorage.getItem(this.legacyCollapsedGroupsKey) ?? "[]");
            return Array.isArray(value) ? new Set(value.filter((item): item is string => typeof item === "string")) : new Set();
        } catch {
            return new Set();
        }
    }

    private saveCollapsedGroups(groups: ReadonlySet<string>): void {
        try {
            localStorage.setItem(this.collapsedGroupsKey, JSON.stringify([...groups]));
        } catch {
            return;
        }
    }

    private processTypeLabel(group: ProcessGroup): string {
        if (group === "background") {
            return "Background process";
        }

        if (group === "windows") {
            return "Windows process";
        }

        return "App";
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