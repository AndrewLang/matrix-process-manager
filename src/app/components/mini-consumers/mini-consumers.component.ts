import { Component, input } from "@angular/core";
import { ProcessRow } from "../../app.models";
import { IconComponent } from "../icon/icon.component";

@Component({
    selector: "mtx-mini-consumers",
    imports: [IconComponent],
    templateUrl: "./mini-consumers.component.html",
})
export class MiniConsumersComponent {
    rows = input.required<ProcessRow[]>();
}