import { NgClass } from "@angular/common";
import { Component, HostListener, computed, inject, signal } from "@angular/core";
import { MetricCard, ProcessRow, ResourceSample } from "../../app.models";
import { DetailsPaneComponent } from "../../components/details-pane/details-pane.component";
import { ProcessGridComponent } from "../../components/process-grid/process-grid.component";
import { SearchBoxComponent } from "../../components/search-box/search-box.component";
import { WorkareaStateService } from "../../services/workarea-state.service";

@Component({
    selector: "mtx-processes-view",
    imports: [NgClass, DetailsPaneComponent, ProcessGridComponent, SearchBoxComponent],
    templateUrl: "./processes-view.component.html",
})
export class ProcessesViewComponent {
    state = inject(WorkareaStateService);
    processFilter = signal("");
    detailsOpen = signal(true);
    viewOptionsOpen = signal(false);

    summaryMetrics = computed(() => this.metricOrder().map((label) => this.metric(label)).filter((metric): metric is MetricCard => Boolean(metric)));
    filteredRows = computed(() => this.filterRows(this.state.rows(), this.processFilter()));

    wideFilterSearchClass = "flex h-7.5 basis-[238px] items-center gap-2 rounded-[5px] border border-(--border) bg-[rgba(15,28,40,0.84)] px-2.5 py-0 text-[12px] text-(--muted)";

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

    metricOrder(): string[] {
        return ["CPU", "Memory", "Disk", "Network"];
    }

    metric(label: string): MetricCard | undefined {
        return this.state.metrics().find((metric) => metric.label === label);
    }

    metricKey(label: string): keyof ResourceSample {
        return label.toLowerCase() as keyof ResourceSample;
    }

    accentClass(accent: string): string {
        return `text-(--${accent})`;
    }

    chartPath(metric: keyof ResourceSample, width = 160, height = 48): string {
        const history = this.state.resourceHistory();
        if (history.length === 0) {
            return `0,${height - 8} ${width},${height - 8}`;
        }

        return history.map((sample, index) => {
            const x = history.length === 1 ? width : index * (width / (history.length - 1));
            const y = height - 8 - Math.max(0, Math.min(100, sample[metric])) / 100 * (height - 16);
            return `${x.toFixed(1)},${y.toFixed(1)}`;
        }).join(" ");
    }

    chartAreaPath(metric: keyof ResourceSample, width = 160, height = 48): string {
        return `0,${height - 8} ${this.chartPath(metric, width, height)} ${width},${height - 8}`;
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