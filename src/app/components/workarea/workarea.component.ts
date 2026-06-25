import { Component, effect, input } from "@angular/core";
import { RouterOutlet } from "@angular/router";
import { MetricCard, ProcessRow, ResourceBar, ViewId } from "../../app.models";
import { WorkareaStateService } from "../../services/workarea-state.service";

@Component({
    selector: "mtx-workarea",
    imports: [RouterOutlet],
    host: { class: "block h-full min-h-0 overflow-hidden" },
    templateUrl: "./workarea.component.html",
})
export class WorkareaComponent {
    activeView = input<ViewId>("dashboard");
    totalProcesses = input(0);
    metrics = input<MetricCard[]>([]);
    rows = input<ProcessRow[]>([]);
    selectedProcess = input("");
    bars = input<ResourceBar[]>([]);
    activeTitle = input("Dashboard");

    constructor(private workareaState: WorkareaStateService) {
        effect(() => {
            this.workareaState.setState({
                activeView: this.activeView(),
                totalProcesses: this.totalProcesses(),
                metrics: this.metrics(),
                rows: this.rows(),
                selectedProcess: this.selectedProcess(),
                bars: this.bars(),
                activeTitle: this.activeTitle(),
            });
        });
    }
}
