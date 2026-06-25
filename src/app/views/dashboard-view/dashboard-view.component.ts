import { NgClass } from "@angular/common";
import { Component, HostListener, computed, inject, signal } from "@angular/core";
import { ProcessRow } from "../../app.models";
import { DetailsPaneComponent } from "../../components/details-pane/details-pane.component";
import { MetricBlockComponent } from "../../components/metric-block/metric-block.component";
import { ProcessGridComponent } from "../../components/process-grid/process-grid.component";
import { SearchBoxComponent } from "../../components/search-box/search-box.component";
import { WorkareaStateService } from "../../services/workarea-state.service";

@Component({
    selector: "mtx-dashboard-view",
    imports: [NgClass, DetailsPaneComponent, MetricBlockComponent, ProcessGridComponent, SearchBoxComponent],
    templateUrl: "./dashboard-view.component.html",
})
export class DashboardViewComponent {
    state = inject(WorkareaStateService);
    processFilter = signal("");
    detailsOpen = signal(true);
    viewOptionsOpen = signal(false);

    filteredRows = computed(() => this.filterRows(this.state.rows(), this.processFilter()));

    filterSearchClass = "flex h-7.5 flex-1 items-center gap-2 rounded-[5px] border border-(--border) bg-[rgba(15,28,40,0.84)] px-2.5 py-0 text-[12px] text-(--muted)";

    selectProcess(row: ProcessRow): void {
        this.detailsOpen.set(true);
        this.state.selectProcess(row);
    }

    closeDetails(): void {
        this.detailsOpen.set(false);
    }

    toggleViewOptions(): void {
        this.viewOptionsOpen.update((open) => !open);
    }

    closeViewOptions(): void {
        this.viewOptionsOpen.set(false);
    }

    @HostListener("document:click")
    closeViewOptionsFromDocument(): void {
        this.closeViewOptions();
    }

    private filterRows(rows: ProcessRow[], filterValue: string): ProcessRow[] {
        const filter = filterValue.trim().toLowerCase();
        if (!filter) {
            return rows;
        }

        return rows.filter((row) =>
            row.name.toLowerCase().includes(filter)
            || row.publisher.toLowerCase().includes(filter)
            || row.pid.toString().includes(filter)
            || row.status.toLowerCase().includes(filter)
            || row.user.toLowerCase().includes(filter),
        );
    }
}