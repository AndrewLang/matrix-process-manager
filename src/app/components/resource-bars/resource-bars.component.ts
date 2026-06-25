import { NgClass } from "@angular/common";
import { Component, input } from "@angular/core";
import { ResourceBar } from "../../app.models";

@Component({
    selector: "mtx-resource-bars",
    imports: [NgClass],
    templateUrl: "./resource-bars.component.html",
})
export class ResourceBarsComponent {
    bars = input.required<ResourceBar[]>();
}