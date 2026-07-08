import { NgClass } from "@angular/common";
import { Component, HostListener, computed, effect, input, output, signal } from "@angular/core";

export type DataGridSortDirection = "asc" | "desc";

export interface DataGridColumn<T = unknown> {
    key: string;
    label: string;
    width: number;
    minWidth: number;
    align?: "left" | "right";
    sortable?: boolean;
    resizable?: boolean;
    value: (row: T) => string | number | undefined;
    cellClass?: (row: T) => string;
}

@Component({
    selector: "mtx-data-grid",
    imports: [NgClass],
    templateUrl: "./data-grid.component.html",
    host: { class: "block h-full min-h-0 min-w-0 overflow-hidden" },
})
export class DataGridComponent<T = unknown> {
    columns = input.required<DataGridColumn<T>[]>();
    rows = input.required<T[]>();
    rowKey = input.required<(row: T) => string>();
    selectedKey = input("");
    loading = input(false);
    emptyText = input("No rows found.");
    loadingText = input("Loading...");
    rowSelected = output<T>();

    visibleColumns = signal<DataGridColumn<T>[]>([]);
    sortKey = signal<string | undefined>(undefined);
    sortDirection = signal<DataGridSortDirection>("asc");
    tableWidth = computed(() => this.visibleColumns().reduce((total, column) => total + column.width, 0));
    sortedRows = computed(() => this.sortRows(this.rows()));

    private resizing?: { index: number; startX: number; startWidth: number };

    constructor() {
        effect(() => {
            const incoming = this.columns();
            this.visibleColumns.update((current) => {
                if (current.length === incoming.length && current.every((column, index) => column.key === incoming[index].key)) {
                    return current.map((column, index) => ({ ...incoming[index], width: column.width }));
                }

                return incoming.map((column) => ({ ...column }));
            });

            if (!this.sortKey()) {
                const firstSortable = incoming.find((column) => column.sortable !== false);
                this.sortKey.set(firstSortable?.key);
            }
        });
    }

    selectRow(row: T): void {
        this.rowSelected.emit(row);
    }

    isSelected(row: T): boolean {
        return this.rowKey()(row) === this.selectedKey();
    }

    sortBy(column: DataGridColumn<T>): void {
        if (column.sortable === false || this.resizing) {
            return;
        }

        if (this.sortKey() === column.key) {
            this.sortDirection.update((direction) => direction === "asc" ? "desc" : "asc");
            return;
        }

        this.sortKey.set(column.key);
        this.sortDirection.set("asc");
    }

    sortIcon(column: DataGridColumn<T>): string {
        if (column.sortable === false) {
            return "";
        }

        if (this.sortKey() !== column.key) {
            return "bi-chevron-expand";
        }

        return this.sortDirection() === "asc" ? "bi-chevron-up" : "bi-chevron-down";
    }

    cellText(row: T, column: DataGridColumn<T>): string {
        const value = column.value(row);
        return value == null || value === "" ? "-" : value.toString();
    }

    cellClass(row: T, column: DataGridColumn<T>): string {
        return column.cellClass?.(row) ?? "";
    }

    startResize(event: MouseEvent, index: number): void {
        event.preventDefault();
        event.stopPropagation();
        const column = this.visibleColumns()[index];
        if (column.resizable === false) {
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
        this.visibleColumns.update((columns) =>
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

    private sortRows(rows: T[]): T[] {
        const sortKey = this.sortKey();
        const column = this.visibleColumns().find((item) => item.key === sortKey);
        if (!column || column.sortable === false) {
            return rows;
        }

        const direction = this.sortDirection() === "asc" ? 1 : -1;
        return [...rows].sort((left, right) => this.compareValues(column.value(left), column.value(right)) * direction);
    }

    private compareValues(left: string | number | undefined, right: string | number | undefined): number {
        if (typeof left === "number" && typeof right === "number") {
            return left - right;
        }

        return (left?.toString() ?? "").localeCompare(right?.toString() ?? "", undefined, { numeric: true, sensitivity: "base" });
    }
}
