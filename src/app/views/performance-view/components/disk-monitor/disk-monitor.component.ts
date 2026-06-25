import { Component } from "@angular/core";
import { ResourceDetailMonitorComponent } from "../resource-detail-monitor/resource-detail-monitor.component";

@Component({
    selector: "mtx-disk-monitor",
    imports: [ResourceDetailMonitorComponent],
    templateUrl: "./disk-monitor.component.html",
})
export class DiskMonitorComponent { }