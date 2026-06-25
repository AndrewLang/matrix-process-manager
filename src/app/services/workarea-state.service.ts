import { Injectable, computed, signal } from "@angular/core";
import { MetricCard, ProcessRow, ResourceBar, ViewId } from "../app.models";

@Injectable({ providedIn: "root" })
export class WorkareaStateService {
    activeView = signal<ViewId>("dashboard");
    totalProcesses = signal(0);
    metrics = signal<MetricCard[]>([]);
    rows = signal<ProcessRow[]>([]);
    selectedProcess = signal("");
    selectedPid = signal<number | undefined>(undefined);
    bars = signal<ResourceBar[]>([]);
    activeTitle = signal("Dashboard");

    selectedRow = computed(() => {
        const rows = this.rows();
        const selectedPid = this.selectedPid();
        return rows.find((row) => row.pid === selectedPid)
            ?? rows.find((row) => row.name === this.selectedProcess())
            ?? rows.find((row) => row.selected)
            ?? rows[0];
    });

    setState(state: {
        activeView: ViewId;
        totalProcesses: number;
        metrics: MetricCard[];
        rows: ProcessRow[];
        selectedProcess: string;
        bars: ResourceBar[];
        activeTitle: string;
    }): void {
        this.activeView.set(state.activeView);
        this.totalProcesses.set(state.totalProcesses);
        this.metrics.set(state.metrics);
        this.rows.set(state.rows);
        this.selectedProcess.set(state.selectedProcess);
        this.bars.set(state.bars);
        this.activeTitle.set(state.activeTitle);
    }

    selectProcess(row: ProcessRow): void {
        this.selectedProcess.set(row.name);
        this.selectedPid.set(row.pid);
    }
}