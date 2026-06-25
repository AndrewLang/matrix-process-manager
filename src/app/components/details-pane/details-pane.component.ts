import { Component, input, output } from "@angular/core";
import { ResourceBar } from "../../app.models";
import { ResourceBarsComponent } from "../resource-bars/resource-bars.component";

@Component({
    selector: "mtx-details-pane",
    imports: [ResourceBarsComponent],
    templateUrl: "./details-pane.component.html",
})
export class DetailsPaneComponent {
    bars = input.required<ResourceBar[]>();
    closeDetails = output<void>();
}