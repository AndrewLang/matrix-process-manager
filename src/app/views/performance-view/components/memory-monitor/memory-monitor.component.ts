import { Component } from "@angular/core";
import { ResourceDetailMonitorComponent } from "../resource-detail-monitor/resource-detail-monitor.component";

@Component({
    selector: "mtx-memory-monitor",
    imports: [ResourceDetailMonitorComponent],
    templateUrl: "./memory-monitor.component.html",
})
export class MemoryMonitorComponent { }