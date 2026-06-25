import { NgClass, NgTemplateOutlet } from "@angular/common";
import { Component, input, output } from "@angular/core";
import { MetricCard, ProcessRow, ResourceBar, ViewId } from "../../app.models";
import { MetricBlockComponent } from "../metric-block/metric-block.component";
import { ProcessGridComponent } from "../process-grid/process-grid.component";
import { SearchBoxComponent } from "../search-box/search-box.component";

@Component({
    selector: "app-workarea",
    imports: [NgClass, NgTemplateOutlet, MetricBlockComponent, ProcessGridComponent, SearchBoxComponent],
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

    filterSearchClass = "flex h-[30px] flex-1 items-center gap-2 rounded-[5px] border border-[var(--border)] bg-[rgba(15,28,40,0.84)] px-2.5 py-0 text-[var(--muted)]";
    wideFilterSearchClass = "flex h-[30px] basis-[238px] items-center gap-2 rounded-[5px] border border-[var(--border)] bg-[rgba(15,28,40,0.84)] px-2.5 py-0 text-[var(--muted)]";
}
