import { NgClass } from "@angular/common";
import { Component, HostListener, input, output, signal } from "@angular/core";
import { ProcessRow } from "../../app.models";

interface ProcessColumn {
    key: string;
    label: string;
    width: number;
    minWidth: number;
    resizable: boolean;
}

@Component({
    selector: "app-process-grid",
    imports: [NgClass],
    templateUrl: "./process-grid.component.html",
})
export class ProcessGridComponent {
    rows = input.required<ProcessRow[]>();
    selectedProcess = input.required<string>();
    processSelected = output<ProcessRow>();

    columns = signal<ProcessColumn[]>([
        { key: "select", label: "", width: 32, minWidth: 32, resizable: false },
        { key: "name", label: "Name", width: 230, minWidth: 150, resizable: true },
        { key: "pid", label: "PID", width: 72, minWidth: 56, resizable: true },
        { key: "status", label: "Status", width: 96, minWidth: 74, resizable: true },
        { key: "cpu", label: "CPU", width: 72, minWidth: 60, resizable: true },
        { key: "memory", label: "Memory", width: 104, minWidth: 82, resizable: true },
        { key: "disk", label: "Disk", width: 102, minWidth: 82, resizable: true },
        { key: "network", label: "Network", width: 104, minWidth: 82, resizable: true },
        { key: "user", label: "User", width: 92, minWidth: 72, resizable: true },
        { key: "menu", label: "", width: 36, minWidth: 36, resizable: false },
    ]);

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
}
