import { NgClass } from "@angular/common";
import { Component, input } from "@angular/core";
import { ProcessRow } from "../../app.models";

@Component({
    selector: "mtx-top-resource-consumers",
    imports: [NgClass],
    templateUrl: "./top-resource-consumers.component.html",
})
export class TopResourceConsumersComponent {
    rows = input.required<ProcessRow[]>();
}