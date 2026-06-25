import { NgClass } from "@angular/common";
import { Component, input } from "@angular/core";
import { MetricCard } from "../../app.models";

@Component({
    selector: "mtx-metric-block",
    imports: [NgClass],
    templateUrl: "./metric-block.component.html",
})
export class MetricBlockComponent {
    metric = input.required<MetricCard>();
}
