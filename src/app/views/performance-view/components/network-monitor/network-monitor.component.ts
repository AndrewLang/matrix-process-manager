import { Component } from "@angular/core";
import { ResourceDetailMonitorComponent } from "../resource-detail-monitor/resource-detail-monitor.component";

@Component({
    selector: "mtx-network-monitor",
    imports: [ResourceDetailMonitorComponent],
    templateUrl: "./network-monitor.component.html",
})
export class NetworkMonitorComponent { }