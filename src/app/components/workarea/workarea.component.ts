import { NgClass, NgTemplateOutlet } from "@angular/common";
import { Component, computed, input, output, signal } from "@angular/core";
import { MetricCard, ProcessRow, ResourceBar, ViewId } from "../../app.models";
import { MetricBlockComponent } from "../metric-block/metric-block.component";
import { ProcessGridComponent } from "../process-grid/process-grid.component";
import { SearchBoxComponent } from "../search-box/search-box.component";

@Component({
    selector: "mtx-workarea",
    imports: [NgClass, NgTemplateOutlet, MetricBlockComponent, ProcessGridComponent, SearchBoxComponent],
    host: { class: "block h-full min-h-0 overflow-hidden" },
    templateUrl: "./workarea.component.html",
})
export class WorkareaComponent {
    activeView = input.required<ViewId>();
    totalProcesses = input.required<number>();
    metrics = input.required<MetricCard[]>();
    rows = input.required<ProcessRow[]>();
    selectedProcess = input.required<string>();
    bars = input.required<ResourceBar[]>();
    activeTitle = input.required<string>();
    processSelected = output<ProcessRow>();
    processFilter = signal("");
    detailsOpen = signal(true);

    filteredRows = computed(() => {
        const filter = this.processFilter().trim().toLowerCase();
        if (!filter) {
            return this.rows();
        }

        return this.rows().filter((row) =>
            row.name.toLowerCase().includes(filter)
            || row.publisher.toLowerCase().includes(filter)
            || row.pid.toString().includes(filter)
            || row.status.toLowerCase().includes(filter)
            || row.user.toLowerCase().includes(filter),
        );
    });

    filterSearchClass = "flex h-7.5 flex-1 items-center gap-2 rounded-[5px] border border-(--border) bg-[rgba(15,28,40,0.84)] px-2.5 py-0 text-[12px] text-(--muted)";
    wideFilterSearchClass = "flex h-7.5 basis-[238px] items-center gap-2 rounded-[5px] border border-(--border) bg-[rgba(15,28,40,0.84)] px-2.5 py-0 text-[12px] text-(--muted)";

    selectProcess(row: ProcessRow): void {
        this.detailsOpen.set(true);
        this.processSelected.emit(row);
    }

    closeDetails(): void {
        this.detailsOpen.set(false);
    }
}
