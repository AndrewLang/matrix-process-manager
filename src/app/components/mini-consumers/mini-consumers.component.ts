import { NgClass } from "@angular/common";
import { Component, input } from "@angular/core";
import { ProcessRow } from "../../app.models";

@Component({
    selector: "mtx-mini-consumers",
    imports: [NgClass],
    templateUrl: "./mini-consumers.component.html",
})
export class MiniConsumersComponent {
    rows = input.required<ProcessRow[]>();
}